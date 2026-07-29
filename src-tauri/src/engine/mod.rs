//! Real-time tag engine, polling scheduler, alarm evaluation.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use chrono::{DateTime, Utc};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::{watch, Mutex as AsyncMutex};
use tokio::task::JoinHandle;

use crate::audit::AuditLog;
use crate::modbus::{self, codec, ConnectionConfig};
use crate::project::{
    credentials, AlarmDefinition, AlarmPriority, BitWriteMode, DeviceConfig, ModbusTable, Role,
    ScadaProject, TagDataType, TagDefinition, UserAccount, UserSummary, LEGACY_SALT,
};

const STALE_AFTER_MS: u64 = 3_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccountInput {
    pub id: Option<String>,
    pub username: String,
    pub display_name: String,
    pub password: Option<String>,
    pub pin: Option<String>,
    pub security_level: u32,
    pub enabled: bool,
}

pub fn security_level_to_role(level: u32) -> Role {
    if level >= 1000 {
        Role::Administrator
    } else if level >= 500 {
        Role::Engineer
    } else if level >= 100 {
        Role::Operator
    } else {
        Role::Viewer
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Good,
    Uncertain,
    Bad,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagValue {
    pub tag_id: String,
    pub value: f64,
    pub bool_value: bool,
    pub quality: Quality,
    pub ts: DateTime<Utc>,
    pub age_ms: u64,
    pub raw: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteReceipt {
    pub tag_id: String,
    pub requested_value: f64,
    pub observed_value: f64,
    pub raw_readback: u16,
    pub protocol: String,
    pub verify_readback: bool,
    pub matches: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlarmState {
    Inactive,
    ActiveUnacked,
    ActiveAcked,
    ClearedUnacked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmInstance {
    pub def_id: String,
    pub name: String,
    pub message: String,
    pub priority: AlarmPriority,
    pub group_id: String,
    pub state: AlarmState,
    pub source_active: bool,
    pub latched: bool,
    pub active_since: Option<DateTime<Utc>>,
    pub last_change: DateTime<Utc>,
    /// True when the source tag is not `Good`, so the displayed state is the
    /// last trustworthy one rather than a current evaluation.
    pub evaluation_suspended: bool,
    /// Why evaluation is suspended, for operator-facing display.
    pub suspended_reason: Option<String>,
    /// When evaluation stopped being trustworthy.
    pub suspended_since: Option<DateTime<Utc>>,
    #[serde(skip)]
    pending_active_since: Option<DateTime<Utc>>,
    #[serde(skip)]
    pending_clear_since: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct PersistedAlarmState {
    def_id: String,
    state: AlarmState,
    source_active: bool,
    active_since: Option<DateTime<Utc>>,
    last_change: DateTime<Utc>,
    pending_active_since: Option<DateTime<Utc>>,
    pending_clear_since: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlarmStateJournal {
    version: u32,
    project_id: String,
    definitions_hash: String,
    alarms: Vec<PersistedAlarmState>,
    content_hash: String,
}

#[derive(Default)]
struct AlarmStateStoreInner {
    path: Option<PathBuf>,
    last_error: Option<String>,
}

#[derive(Default)]
struct AlarmStateStore {
    inner: Mutex<AlarmStateStoreInner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserRealmJournal {
    version: u32,
    provisioning_closed: bool,
    users: Vec<UserAccount>,
    content_hash: String,
}

#[derive(Default)]
struct UserRealmStoreInner {
    path: Option<PathBuf>,
    last_error: Option<String>,
    restore_failed: bool,
    provisioning_closed: bool,
}

#[derive(Default)]
struct UserRealmStore {
    inner: Mutex<UserRealmStoreInner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSnapshot {
    pub connected: bool,
    pub device_id: Option<String>,
    pub last_error: Option<String>,
    pub poll_count: u64,
    pub last_poll_ms: u64,
    pub tags: Vec<TagValue>,
    pub alarms: Vec<AlarmInstance>,
    pub role: Role,
    pub actor: String,
    pub current_user: Option<UserSummary>,
    pub security_level: u32,
    pub project_name: Option<String>,
    pub mode: String,
    /// True when alarm evaluation is not running against live data.
    pub alarms_suspended: bool,
    /// True when the signed-in account must change its password before it can
    /// write to the process or administer users.
    pub password_change_required: bool,
    /// True when the loaded project has no accounts and the one-shot local
    /// Administrator provisioning command is available.
    pub requires_bootstrap: bool,
    pub audit_chain_ok: bool,
    pub audit_persisted: bool,
    pub audit_last_error: Option<String>,
    pub alarm_state_persisted: bool,
    pub alarm_state_last_error: Option<String>,
    pub user_realm_persisted: bool,
    pub user_realm_last_error: Option<String>,
}

struct LiveTag {
    def: TagDefinition,
    value: f64,
    bool_value: bool,
    quality: Quality,
    ts: DateTime<Utc>,
    raw: u16,
}

struct LoginThrottle {
    failures: u32,
    blocked_until: Option<DateTime<Utc>>,
}

struct EngineInner {
    project: Option<ScadaProject>,
    tags: HashMap<String, LiveTag>,
    alarms: HashMap<String, AlarmInstance>,
    connected: bool,
    device_id: Option<String>,
    last_error: Option<String>,
    poll_count: u64,
    last_poll_ms: u64,
    role: Role,
    actor: String,
    current_user: Option<UserSummary>,
    security_level: u32,
    last_activity_ts: DateTime<Utc>,
    /// Incremented whenever identity, privilege or operating mode changes.
    auth_epoch: u64,
    /// Incremented whenever a project is installed.
    project_epoch: u64,
    /// Generation token preventing an aborted poller from publishing late data.
    poll_epoch: u64,
    login_throttle: HashMap<String, LoginThrottle>,
    mode: String, // designer | runtime
    poll_handle: Option<JoinHandle<()>>,
    stop_tx: Option<watch::Sender<bool>>,
}

pub struct Engine {
    inner: Arc<RwLock<EngineInner>>,
    audit: Arc<AuditLog>,
    alarm_state_store: Arc<AlarmStateStore>,
    user_realm_store: Arc<UserRealmStore>,
    write_locks: RegisterWriteLocks,
    /// One reusable, serialized write connection per device. Without this every
    /// operator command would open — and leak — a TCP session on the PLC.
    write_sessions: WriteSessions,
    /// Dedicated Tokio runtime for Modbus I/O and the poll loop.
    /// Sync Tauri commands are not on a reactor — never use bare `tokio::spawn` here.
    rt: tokio::runtime::Runtime,
}

type RegisterWriteLocks = Mutex<HashMap<(String, u16), Arc<AsyncMutex<()>>>>;
type WriteSessions = Mutex<HashMap<String, Arc<AsyncMutex<Option<modbus::ModbusContext>>>>>;

impl AlarmStateStore {
    fn attach(&self, path: &Path, g: &mut EngineInner) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("alarm-state directory: {error}"))?;
        }
        {
            let mut store = self.inner.lock();
            store.path = Some(path.to_path_buf());
            store.last_error = None;
        }
        if path
            .symlink_metadata()
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            let error = "Refusing symbolic-link alarm-state journal".to_string();
            self.inner.lock().last_error = Some(error.clone());
            return Err(error);
        }
        if !path.exists() {
            return self.persist(g);
        }

        let result = (|| -> Result<(), String> {
            let bytes = std::fs::read(path)
                .map_err(|error| format!("read alarm-state journal: {error}"))?;
            let journal: AlarmStateJournal = serde_json::from_slice(&bytes)
                .map_err(|error| format!("alarm-state journal is corrupt: {error}"))?;
            validate_alarm_journal(&journal, g)?;
            for persisted in &journal.alarms {
                let alarm = g
                    .alarms
                    .get_mut(&persisted.def_id)
                    .ok_or_else(|| format!("Alarm {} no longer exists", persisted.def_id))?;
                alarm.state = persisted.state.clone();
                alarm.source_active = persisted.source_active;
                alarm.active_since = persisted.active_since;
                alarm.last_change = persisted.last_change;
                alarm.pending_active_since = persisted.pending_active_since;
                alarm.pending_clear_since = persisted.pending_clear_since;
                // Quality starts Bad after restart, so the restored lifecycle
                // is explicitly suspended until a fresh sample arrives.
                alarm.evaluation_suspended = true;
                alarm.suspended_reason = Some("Restored state awaiting live data".into());
                alarm.suspended_since = Some(Utc::now());
            }
            Ok(())
        })();
        let mut store = self.inner.lock();
        match result {
            Ok(()) => {
                store.last_error = None;
                Ok(())
            }
            Err(error) => {
                store.last_error = Some(error.clone());
                Err(error)
            }
        }
    }

    fn persist(&self, g: &EngineInner) -> Result<(), String> {
        self.persist_alarms(g, &g.alarms)
    }

    fn persist_if_attached(&self, g: &EngineInner) -> Result<(), String> {
        if self.is_attached() {
            self.persist(g)
        } else {
            Ok(())
        }
    }

    fn persist_alarms(
        &self,
        g: &EngineInner,
        alarms: &HashMap<String, AlarmInstance>,
    ) -> Result<(), String> {
        let path = self
            .inner
            .lock()
            .path
            .clone()
            .ok_or("Alarm-state journal is not attached")?;
        let journal = build_alarm_journal(g, alarms)?;
        let bytes = serde_json::to_vec_pretty(&journal)
            .map_err(|error| format!("encode alarm-state journal: {error}"))?;
        let result = atomic_write_protected_state(&path, &bytes, "alarm-state journal");
        let mut store = self.inner.lock();
        match result {
            Ok(()) => {
                store.last_error = None;
                Ok(())
            }
            Err(error) => {
                store.last_error = Some(error.clone());
                Err(error)
            }
        }
    }

    fn status(&self) -> (bool, Option<String>) {
        let store = self.inner.lock();
        (
            store.path.is_some() && store.last_error.is_none(),
            store.last_error.clone(),
        )
    }

    fn is_attached(&self) -> bool {
        self.inner.lock().path.is_some()
    }

    fn owns_path(&self, path: &Path) -> bool {
        self.inner
            .lock()
            .path
            .as_deref()
            .is_some_and(|owned| paths_refer_to_same_file(owned, path))
    }
}

impl UserRealmStore {
    fn attach(&self, path: &Path, g: &mut EngineInner) -> Result<(), String> {
        let project = g.project.as_ref().ok_or("No project loaded")?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("user-realm directory: {error}"))?;
        }
        {
            let mut store = self.inner.lock();
            store.path = Some(path.to_path_buf());
            store.last_error = None;
            store.restore_failed = false;
            // Fail closed while attachment/validation is still in progress.
            store.provisioning_closed = true;
        }

        let fail_restore = |error: String| {
            let mut store = self.inner.lock();
            store.last_error = Some(error.clone());
            store.restore_failed = true;
            store.provisioning_closed = true;
            Err(error)
        };

        if path
            .symlink_metadata()
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return fail_restore("Refusing symbolic-link user realm".into());
        }

        if !path.exists() {
            if !project.users.is_empty() {
                return fail_restore(
                    "Refusing to seed a new user realm from project-embedded accounts".into(),
                );
            }
            {
                let mut store = self.inner.lock();
                store.provisioning_closed = false;
            }
            return self.persist_users(&project.users, false);
        }

        let result = (|| -> Result<(Vec<UserAccount>, bool), String> {
            let bytes = std::fs::read(path).map_err(|error| format!("read user realm: {error}"))?;
            let journal: UserRealmJournal = serde_json::from_slice(&bytes)
                .map_err(|error| format!("user realm is corrupt: {error}"))?;
            validate_user_realm_journal(&journal, project)?;
            restrict_secret_file_permissions(path)?;
            Ok((journal.users, journal.provisioning_closed))
        })();

        match result {
            Ok((users, provisioning_closed)) => {
                let mut project = g.project.clone().ok_or("No project loaded")?;
                project.users = users;
                project.recompute_hash();
                g.project = Some(project);
                let mut store = self.inner.lock();
                store.last_error = None;
                store.restore_failed = false;
                store.provisioning_closed = provisioning_closed;
                Ok(())
            }
            Err(error) => fail_restore(error),
        }
    }

    fn persist_users(
        &self,
        users: &[UserAccount],
        provisioning_closed: bool,
    ) -> Result<(), String> {
        let path = {
            let store = self.inner.lock();
            if store.restore_failed {
                return Err(
                    "User realm restore failed; account mutation is blocked until recovery".into(),
                );
            }
            store.path.clone().ok_or("User realm is not attached")?
        };
        if !provisioning_closed && !users.is_empty() {
            return Err("An open user realm cannot contain accounts".into());
        }
        let journal = build_user_realm_journal(users, provisioning_closed)?;
        let bytes = serde_json::to_vec_pretty(&journal)
            .map_err(|error| format!("encode user realm: {error}"))?;
        let result = atomic_write_protected_state(&path, &bytes, "user realm");
        let mut store = self.inner.lock();
        match result {
            Ok(()) => {
                store.last_error = None;
                store.provisioning_closed = provisioning_closed;
                Ok(())
            }
            Err(error) => {
                store.last_error = Some(error.clone());
                Err(error)
            }
        }
    }

    fn status(&self) -> (bool, Option<String>) {
        let store = self.inner.lock();
        (
            store.path.is_some() && !store.restore_failed && store.last_error.is_none(),
            store.last_error.clone(),
        )
    }

    fn bootstrap_available(&self, users_are_empty: bool) -> bool {
        let store = self.inner.lock();
        store.path.is_some()
            && !store.restore_failed
            && store.last_error.is_none()
            && !store.provisioning_closed
            && users_are_empty
    }

    fn owns_path(&self, path: &Path) -> bool {
        self.inner
            .lock()
            .path
            .as_deref()
            .is_some_and(|owned| paths_refer_to_same_file(owned, path))
    }
}

impl Engine {
    pub fn new(audit: Arc<AuditLog>) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("proscada-modbus")
            .build()
            .expect("failed to create ProScada Tokio runtime");

        Self {
            // Fail-closed: the engine holds no privileges until `login` grants them.
            inner: Arc::new(RwLock::new(EngineInner {
                project: None,
                tags: HashMap::new(),
                alarms: HashMap::new(),
                connected: false,
                device_id: None,
                last_error: None,
                poll_count: 0,
                last_poll_ms: 0,
                role: Role::Viewer,
                actor: "guest".into(),
                current_user: None,
                security_level: 0,
                last_activity_ts: Utc::now(),
                auth_epoch: 0,
                project_epoch: 0,
                poll_epoch: 0,
                login_throttle: HashMap::new(),
                mode: "runtime".into(),
                poll_handle: None,
                stop_tx: None,
            })),
            audit,
            alarm_state_store: Arc::new(AlarmStateStore::default()),
            user_realm_store: Arc::new(UserRealmStore::default()),
            write_locks: Mutex::new(HashMap::new()),
            write_sessions: Mutex::new(HashMap::new()),
            rt,
        }
    }

    pub fn audit(&self) -> Arc<AuditLog> {
        self.audit.clone()
    }

    pub fn attach_alarm_state_store(&self, path: &Path) -> Result<(), String> {
        let mut g = self.inner.write();
        self.alarm_state_store.attach(path, &mut g)
    }

    pub fn attach_user_realm_store(&self, path: &Path) -> Result<(), String> {
        let mut g = self.inner.write();
        self.user_realm_store.attach(path, &mut g)
    }

    /// Handle to the dedicated Modbus/poll Tokio runtime.
    pub fn runtime(&self) -> tokio::runtime::Handle {
        self.rt.handle().clone()
    }

    /// Install a project without touching the session.
    ///
    /// `verify_integrity` is only false for the built-in template, which is
    /// constructed in memory and therefore has no stored hash to compare.
    fn install_project(&self, project: ScadaProject, verify_integrity: bool) -> Result<(), String> {
        self.install_project_inner(project, verify_integrity, false)
    }

    fn install_project_inner(
        &self,
        project: ScadaProject,
        verify_integrity: bool,
        preserve_alarm_state: bool,
    ) -> Result<(), String> {
        let mut proj = project;
        // Verify *before* mutating, otherwise the check compares the content
        // against a hash we just recomputed and can never fail.
        if verify_integrity && proj.content_hash.trim().is_empty() {
            return Err("External project is missing its content hash".into());
        }
        if verify_integrity && !proj.verify_hash() {
            return Err("Project content hash verification failed".into());
        }
        proj.harden_seeded_accounts();
        proj.validate()?;
        proj.recompute_hash();

        self.stop_polling_internal()?;
        self.close_write_sessions();
        let mut g = self.inner.write();
        let previous_project = g.project.clone();
        let previous_alarms = if preserve_alarm_state {
            g.alarms.clone()
        } else {
            HashMap::new()
        };
        let mut tags = HashMap::new();
        for def in &proj.tags {
            let is_internal = def.binding.table == ModbusTable::Memory
                || def.binding.table == ModbusTable::System;
            let initial_value = if is_internal {
                normalize_internal_initial(def).unwrap_or(0.0)
            } else {
                0.0
            };
            let initial_bool = if is_internal {
                if let Some(ref s) = def.initial_value {
                    s == "true" || s == "1" || initial_value != 0.0
                } else {
                    false
                }
            } else {
                false
            };
            tags.insert(
                def.id.clone(),
                LiveTag {
                    def: def.clone(),
                    value: initial_value,
                    bool_value: initial_bool,
                    quality: if is_internal {
                        Quality::Good
                    } else {
                        Quality::Bad
                    },
                    ts: Utc::now(),
                    raw: initial_value as u16,
                },
            );
        }
        let mut alarms = HashMap::new();
        for def in &proj.alarms {
            let preserved = previous_project
                .as_ref()
                .and_then(|old_project| {
                    old_project
                        .alarms
                        .iter()
                        .find(|old| alarm_logic_compatible(old, def))
                })
                .and_then(|_| previous_alarms.get(&def.id))
                .cloned();
            let mut instance = preserved.unwrap_or_else(|| new_alarm_instance(def));
            instance.name = def.name.clone();
            instance.message = def.message.clone();
            instance.priority = def.priority.clone();
            instance.group_id = def.group_id.clone();
            instance.latched = def.latching;
            alarms.insert(def.id.clone(), instance);
        }
        let name = proj.name.clone();
        g.project = Some(proj);
        g.tags = tags;
        g.alarms = alarms;
        g.connected = false;
        g.device_id = None;
        g.last_error = None;
        g.poll_count = 0;
        g.project_epoch = g.project_epoch.wrapping_add(1);
        let alarm_state_result = if self.alarm_state_store.is_attached() {
            self.alarm_state_store.persist(&g)
        } else {
            Ok(())
        };
        let actor = g.actor.clone();
        let role = g.role.clone();
        drop(g);
        self.audit
            .append(&actor, role_str(&role), "project.load", &name);
        alarm_state_result?;
        Ok(())
    }

    /// Import a project while preserving the current authentication realm.
    ///
    /// User records in an imported file are never adopted: otherwise an
    /// Engineer could import a correctly hashed project containing their own
    /// Administrator account and escalate privileges.
    pub fn load_project(&self, mut project: ScadaProject) -> Result<(), String> {
        if project.content_hash.trim().is_empty() {
            return Err("External project is missing its content hash".into());
        }
        if !project.verify_hash() {
            return Err("Project content hash verification failed".into());
        }
        project.users = self
            .inner
            .read()
            .project
            .as_ref()
            .map(|current| current.users.clone())
            .unwrap_or_default();
        project.recompute_hash();
        self.install_project(project, false)?;
        self.reset_session("project.load");
        Ok(())
    }

    /// Load the built-in template during start-up, before any user exists.
    pub fn load_builtin(&self, project: ScadaProject) -> Result<(), String> {
        self.install_project(project, false)?;
        self.reset_session("project.load_builtin");
        Ok(())
    }

    /// User-triggered built-in load follows the same realm-preserving rule as
    /// external import. Only process/design content is replaced.
    pub fn load_builtin_preserving_users(&self, mut project: ScadaProject) -> Result<(), String> {
        project.users = self
            .inner
            .read()
            .project
            .as_ref()
            .map(|current| current.users.clone())
            .unwrap_or_default();
        project.recompute_hash();
        self.install_project(project, false)?;
        self.reset_session("project.load_builtin");
        Ok(())
    }

    fn reset_session(&self, reason: &str) {
        let mut g = self.inner.write();
        if g.current_user.is_none() && g.security_level == 0 {
            g.mode = "runtime".into();
            return;
        }
        let prev = g.actor.clone();
        g.current_user = None;
        g.security_level = 0;
        g.role = Role::Viewer;
        g.actor = "guest".into();
        g.last_activity_ts = Utc::now();
        g.auth_epoch = g.auth_epoch.wrapping_add(1);
        g.mode = "runtime".into();
        drop(g);
        self.audit
            .append(&prev, "viewer", "auth.session_reset", reason);
    }

    #[cfg(test)]
    pub fn get_project(&self) -> Option<ScadaProject> {
        self.inner.read().project.clone()
    }

    /// Viewer-safe project representation used by the command boundary.
    /// Account records never leave the trusted Rust side.
    pub fn get_project_redacted(&self) -> Option<ScadaProject> {
        self.inner.read().project.clone().map(|mut project| {
            project.users.clear();
            project.recompute_hash();
            project
        })
    }

    /// Persist the canonical backend project with a durable same-directory
    /// temporary file and one recoverable backup of the previous version.
    /// This administrative export remains a complete backend project and
    /// therefore includes credential hashes; viewer-facing `get_project`
    /// continues to return a redacted copy.
    pub fn save_project_file(&self, path: &str) -> Result<(), String> {
        self.expire_idle_session();
        let (project, actor, role) = {
            let g = self.inner.read();
            require_engineer(&g, "save a project file")?;
            (
                g.project.clone().ok_or("No project loaded")?,
                g.actor.clone(),
                g.role.clone(),
            )
        };
        let target = validate_project_file_path(Path::new(path))?;
        if self.alarm_state_store.owns_path(&target) {
            return Err("Project save path collides with the alarm-state journal".into());
        }
        if self.user_realm_store.owns_path(&target) {
            return Err("Project save path collides with the user realm".into());
        }
        let bytes = serde_json::to_vec_pretty(&project)
            .map_err(|error| format!("Project serialization failed: {error}"))?;
        self.audit.append_required(
            &actor,
            role_str(&role),
            "project.save_file_requested",
            &target.display().to_string(),
        )?;
        atomic_write_project(&target, &bytes)?;
        self.audit.append(
            &actor,
            role_str(&role),
            "project.save_file",
            &target.display().to_string(),
        );
        Ok(())
    }

    /// Persist designer edits.
    ///
    /// The user database is deliberately *not* taken from the incoming project:
    /// accounts change only through `save_user`/`delete_user`, which require
    /// administrator level. Otherwise an Engineer could grant themselves
    /// administrator by editing the project payload.
    pub fn set_project_mut(&self, project: ScadaProject) -> Result<(), String> {
        self.expire_idle_session();
        let existing_users = {
            let g = self.inner.read();
            if !g.role.can_edit_project() {
                return Err(
                    "Engineer or Administrator role is required to edit the project".into(),
                );
            }
            if g.current_user
                .as_ref()
                .is_some_and(|u| u.password_change_required)
            {
                return Err("Change the default password before editing the project".into());
            }
            g.project.as_ref().map(|p| p.users.clone())
        };
        let mut p = project;
        if let Some(users) = existing_users {
            p.users = users;
        }
        p.recompute_hash();
        self.install_project_inner(p, false, true)
    }

    /// Switch between Designer and Runtime.
    ///
    /// Entering Designer is an engineering action and requires the matching
    /// role; entering Runtime is always allowed so a viewer can observe.
    pub fn set_mode(&self, mode: String) -> Result<(), String> {
        self.expire_idle_session();
        if mode != "designer" && mode != "runtime" {
            return Err(format!("Unknown mode: {mode}"));
        }
        let mut g = self.inner.write();
        if mode == "designer" && !g.role.can_edit_project() {
            return Err("Engineer or Administrator role is required to enter Designer".into());
        }
        g.mode = mode.clone();
        g.last_activity_ts = Utc::now();
        g.auth_epoch = g.auth_epoch.wrapping_add(1);
        let actor = g.actor.clone();
        let role = g.role.clone();
        drop(g);
        self.audit
            .append(&actor, role_str(&role), "mode.set", &mode);
        Ok(())
    }

    /// Provision the first Administrator without any public factory secret.
    ///
    /// This is deliberately a one-shot operation: after the first account
    /// exists, only an authenticated Administrator can manage users.
    pub fn bootstrap_admin(&self, password: &str) -> Result<UserSummary, String> {
        if password.chars().count() < 12 {
            return Err("Bootstrap password must be at least 12 characters long".into());
        }
        let mut g = self.inner.write();
        let mut project = g.project.clone().ok_or("No project loaded")?;
        if !self
            .user_realm_store
            .bootstrap_available(project.users.is_empty())
        {
            return Err(
                "Administrator bootstrap is unavailable because the user realm is closed, absent, or degraded"
                    .into(),
            );
        }
        let user = UserAccount {
            id: format!("usr_{}", uuid::Uuid::new_v4().simple()),
            username: "admin".into(),
            display_name: "Administrator".into(),
            password_hash: credentials::hash_secret(password)?,
            salt: LEGACY_SALT.into(),
            pin_hash: None,
            security_level: 1000,
            enabled: true,
            password_change_required: false,
        };
        let summary = user.to_summary();
        self.audit.append_required(
            "local-bootstrap",
            "unauthenticated",
            "auth.bootstrap_admin_requested",
            "Provision first Administrator",
        )?;
        project.users.push(user);
        project.validate()?;
        project.recompute_hash();
        self.user_realm_store.persist_users(&project.users, true)?;
        g.login_throttle.clear();
        g.project = Some(project);
        drop(g);
        self.audit.append(
            "local-bootstrap",
            "unauthenticated",
            "auth.bootstrap_admin",
            "First Administrator provisioned; bootstrap permanently closed",
        );
        Ok(summary)
    }

    /// Force-authenticate an Administrator session during local development (`tauri:dev`).
    ///
    /// This bypasses password checks, cleared locks, and invalid credentials in
    /// debug builds so developer iteration is never blocked by stale realm files.
    pub fn dev_force_admin_login(&self) -> Result<UserSummary, String> {
        let mut g = self.inner.write();
        let project = g.project.as_mut().ok_or("No project loaded")?;

        let admin_pos = project
            .users
            .iter()
            .position(|u| u.enabled && u.security_level >= 1000);

        let summary = if let Some(idx) = admin_pos {
            project.users[idx].to_summary()
        } else {
            let user = UserAccount {
                id: format!("usr_{}", uuid::Uuid::new_v4().simple()),
                username: "admin".into(),
                display_name: "Administrator".into(),
                password_hash: credentials::hash_secret("admin123")?,
                salt: LEGACY_SALT.into(),
                pin_hash: None,
                security_level: 1000,
                enabled: true,
                password_change_required: false,
            };
            let sum = user.to_summary();
            project.users.push(user);
            let _ = self.user_realm_store.persist_users(&project.users, true);
            sum
        };

        let level = summary.security_level;
        let username = summary.username.clone();

        g.current_user = Some(summary.clone());
        g.security_level = level;
        g.role = security_level_to_role(level);
        g.actor = username.clone();
        g.last_activity_ts = Utc::now();
        g.auth_epoch = g.auth_epoch.wrapping_add(1);
        g.login_throttle.clear();
        drop(g);

        self.audit.append(
            &username,
            &format!("L{level}"),
            "auth.dev_force_login",
            "Developer Administrator session authenticated",
        );
        Ok(summary)
    }

    /// Authenticate by username and password. PINs never establish a session;
    /// they are reserved for re-authenticating an already signed-in operator.
    ///
    /// Legacy SHA-256 records are accepted once and immediately re-hashed with
    /// Argon2id, so an old project upgrades itself on first successful login.
    pub fn login(&self, username: &str, password: Option<&str>) -> Result<UserSummary, String> {
        let mut g = self.inner.write();
        let term = username.trim();
        let throttle_key = term.to_ascii_lowercase();
        if term.is_empty() {
            drop(g);
            self.audit.append(
                "unknown",
                "unauthenticated",
                "auth.login_failed",
                "Empty credential",
            );
            return Err("Invalid username or password".into());
        }
        if login_is_throttled(&mut g, &throttle_key) {
            let actor = term.to_string();
            drop(g);
            self.audit.append(
                &actor,
                "unauthenticated",
                "auth.login_throttled",
                "Too many failed login attempts",
            );
            return Err("Too many failed login attempts; try again later".into());
        }

        let project = g.project.as_ref().ok_or("No project loaded")?;
        let mut matched: Option<(usize, LoginKind, credentials::Verification)> = None;
        for (index, user) in project.users.iter().enumerate() {
            if !user.enabled {
                continue;
            }
            if user.username.eq_ignore_ascii_case(term) {
                if let Some(pwd) = password.filter(|p| !p.is_empty()) {
                    let outcome = credentials::verify_secret(pwd, &user.password_hash, &user.salt);
                    if outcome.is_accepted() {
                        matched = Some((index, LoginKind::Password, outcome));
                        break;
                    }
                }
            }
        }

        let Some((index, kind, outcome)) = matched else {
            record_login_failure(&mut g, &throttle_key);
            drop(g);
            self.audit.append(
                term,
                "unauthenticated",
                "auth.login_failed",
                "Invalid credentials",
            );
            return Err("Invalid username or password".into());
        };
        g.login_throttle.remove(&throttle_key);

        let mut rehashed = false;
        if outcome == credentials::Verification::AcceptedNeedsRehash {
            let secret = match kind {
                LoginKind::Password => password.map(str::to_string),
            };
            if let Some(secret) = secret {
                if let Ok(upgraded) = credentials::hash_secret(&secret) {
                    if self
                        .audit
                        .append_required(
                            term,
                            "unauthenticated",
                            "auth.credential_rehash_requested",
                            "Upgrade legacy password hash to Argon2id",
                        )
                        .is_ok()
                    {
                        let mut project = g.project.clone().expect("project present");
                        match kind {
                            LoginKind::Password => project.users[index].password_hash = upgraded,
                        }
                        project.recompute_hash();
                        if self
                            .user_realm_store
                            .persist_users(&project.users, true)
                            .is_ok()
                        {
                            g.project = Some(project);
                            rehashed = true;
                        }
                    }
                }
            }
        }

        let user = &g.project.as_ref().expect("project present").users[index];
        let summary = user.to_summary();
        let level = user.security_level;
        let username = user.username.clone();

        g.current_user = Some(summary.clone());
        g.security_level = level;
        g.role = security_level_to_role(level);
        g.actor = username.clone();
        g.last_activity_ts = Utc::now();
        g.auth_epoch = g.auth_epoch.wrapping_add(1);
        drop(g);

        let role_desc = format!("L{level}");
        self.audit.append(
            &username,
            &role_desc,
            "auth.login_success",
            if rehashed {
                "User authenticated; legacy credential upgraded to Argon2id"
            } else {
                "User authenticated"
            },
        );
        Ok(summary)
    }

    /// Replace the signed-in user's own password and clear the forced-change flag.
    pub fn change_password(
        &self,
        current_password: &str,
        new_password: &str,
    ) -> Result<UserSummary, String> {
        self.expire_idle_session();
        if new_password.chars().count() < 12 {
            return Err("New password must be at least 12 characters long".into());
        }
        if new_password == current_password {
            return Err("New password must differ from the current one".into());
        }

        let mut g = self.inner.write();
        let user_id = g
            .current_user
            .as_ref()
            .map(|u| u.id.clone())
            .ok_or("No user is signed in")?;
        let actor = g.actor.clone();
        let level = g.security_level;
        let mut project = g.project.clone().ok_or("No project loaded")?;
        let index = project
            .users
            .iter()
            .position(|u| u.id == user_id)
            .ok_or("Signed-in user no longer exists in this project")?;
        if !credentials::verify_secret(
            current_password,
            &project.users[index].password_hash,
            &project.users[index].salt,
        )
        .is_accepted()
        {
            drop(g);
            self.audit.append(
                &user_id,
                "unauthenticated",
                "auth.password_change_failed",
                "Current password rejected",
            );
            return Err("Current password is incorrect".into());
        }

        self.audit.append_required(
            &actor,
            &format!("L{level}"),
            "auth.password_change_requested",
            "Replace signed-in user's password",
        )?;
        let was_forced_change = project.users[index].password_change_required;
        project.users[index].password_hash = credentials::hash_secret(new_password)?;
        project.users[index].salt = LEGACY_SALT.into();
        project.users[index].password_change_required = false;
        if was_forced_change {
            // A PIN provisioned alongside a known/default password cannot be
            // trusted as an independently established second secret.
            project.users[index].pin_hash = None;
        }
        let summary = project.users[index].to_summary();
        project.recompute_hash();
        self.user_realm_store.persist_users(&project.users, true)?;
        g.project = Some(project);
        g.current_user = Some(summary.clone());
        g.auth_epoch = g.auth_epoch.wrapping_add(1);
        drop(g);

        self.audit.append(
            &actor,
            &format!("L{level}"),
            "auth.password_changed",
            "Password replaced by its owner",
        );
        Ok(summary)
    }

    pub fn logout(&self) -> Result<(), String> {
        let mut g = self.inner.write();
        let prev_actor = g.actor.clone();
        let prev_level = g.security_level;
        g.current_user = None;
        g.security_level = 0;
        g.role = Role::Viewer;
        g.actor = "guest".into();
        g.last_activity_ts = Utc::now();
        g.auth_epoch = g.auth_epoch.wrapping_add(1);
        g.mode = "runtime".into();
        drop(g);
        self.audit.append(
            &prev_actor,
            &format!("L{prev_level}"),
            "auth.logout",
            "User logged out",
        );
        Ok(())
    }

    pub fn list_users(&self) -> Result<Vec<UserSummary>, String> {
        self.expire_idle_session();
        let g = self.inner.read();
        require_administrator(&g)?;
        let project = g.project.as_ref().ok_or("No project loaded")?;
        Ok(project.users.iter().map(|u| u.to_summary()).collect())
    }

    pub fn save_user(&self, input: UserAccountInput) -> Result<UserSummary, String> {
        self.expire_idle_session();
        let mut g = self.inner.write();
        require_administrator(&g)?;
        if input.username.trim().is_empty() {
            return Err("Username must not be empty".into());
        }
        if input.security_level > 1000 {
            return Err("Security level must be 0..=1000".into());
        }
        if let Some(pwd) = input.password.as_deref().map(str::trim) {
            if !pwd.is_empty() && pwd.chars().count() < 12 {
                return Err("Password must be at least 12 characters long".into());
            }
        }
        if let Some(pin) = input.pin.as_deref().map(str::trim) {
            if !pin.is_empty()
                && (pin.chars().count() < 6 || !pin.chars().all(|c| c.is_ascii_digit()))
            {
                return Err("PIN must be at least 6 digits".into());
            }
        }
        let mut project = g.project.clone().ok_or("No project loaded")?;
        let user_idx = match input.id.as_deref() {
            Some(id) => Some(
                project
                    .users
                    .iter()
                    .position(|u| u.id == id)
                    .ok_or("User not found")?,
            ),
            None => None,
        };
        if project.users.iter().enumerate().any(|(idx, user)| {
            Some(idx) != user_idx && user.username.eq_ignore_ascii_case(input.username.trim())
        }) {
            return Err("Username is already in use".into());
        }
        if g.current_user
            .as_ref()
            .is_some_and(|current| Some(current.id.as_str()) == input.id.as_deref())
            && !input.enabled
        {
            return Err("You cannot disable the account you are signed in with".into());
        }
        if let Some(pin) = input
            .pin
            .as_deref()
            .map(str::trim)
            .filter(|pin| !pin.is_empty())
        {
            if project.users.iter().enumerate().any(|(idx, user)| {
                Some(idx) != user_idx
                    && user.pin_hash.as_ref().is_some_and(|hash| {
                        credentials::verify_secret(pin, hash, &user.salt).is_accepted()
                    })
            }) {
                return Err("PIN is already assigned to another account".into());
            }
        }

        let summary = if let Some(idx) = user_idx {
            let user = &mut project.users[idx];
            user.username = input.username.trim().to_string();
            user.display_name = input.display_name.trim().to_string();
            user.security_level = input.security_level;
            user.enabled = input.enabled;
            if let Some(pwd) = input.password.as_deref().map(str::trim) {
                if !pwd.is_empty() {
                    user.password_hash = credentials::hash_secret(pwd)?;
                    user.password_change_required = false;
                }
            }
            if let Some(pin) = input.pin.as_deref().map(str::trim) {
                user.pin_hash = if pin.is_empty() {
                    None
                } else {
                    Some(credentials::hash_secret(pin)?)
                };
            }
            user.to_summary()
        } else {
            let pwd = input
                .password
                .as_deref()
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .ok_or("A password is required when creating an account")?;
            let pin_hash = match input
                .pin
                .as_deref()
                .map(str::trim)
                .filter(|p| !p.is_empty())
            {
                Some(pin) => Some(credentials::hash_secret(pin)?),
                None => None,
            };
            let new_user = UserAccount {
                id: format!("usr_{}", uuid::Uuid::new_v4().simple()),
                username: input.username.trim().to_string(),
                display_name: input.display_name.trim().to_string(),
                password_hash: credentials::hash_secret(pwd)?,
                salt: LEGACY_SALT.into(),
                pin_hash,
                security_level: input.security_level,
                enabled: input.enabled,
                password_change_required: false,
            };
            let sum = new_user.to_summary();
            project.users.push(new_user);
            sum
        };

        project.validate()?;
        project.recompute_hash();
        self.audit.append_required(
            &g.actor,
            role_str(&g.role),
            "user.save_requested",
            &format!(
                "User {} (Level {})",
                summary.username, summary.security_level
            ),
        )?;
        self.user_realm_store.persist_users(&project.users, true)?;
        g.project = Some(project);
        if g.current_user
            .as_ref()
            .is_some_and(|current| current.id == summary.id)
        {
            g.current_user = Some(summary.clone());
            g.security_level = summary.security_level;
            g.role = security_level_to_role(summary.security_level);
            g.actor = summary.username.clone();
            g.auth_epoch = g.auth_epoch.wrapping_add(1);
        }
        let actor = g.actor.clone();
        let level = g.security_level;
        drop(g);

        self.audit.append(
            &actor,
            &format!("L{level}"),
            "user.save",
            &format!(
                "User {} saved (Level {})",
                summary.username, summary.security_level
            ),
        );

        Ok(summary)
    }

    pub fn delete_user(&self, user_id: &str) -> Result<(), String> {
        self.expire_idle_session();
        let mut g = self.inner.write();
        require_administrator(&g)?;
        if g.current_user.as_ref().is_some_and(|u| u.id == user_id) {
            return Err("You cannot delete the account you are signed in with".into());
        }
        let actor = g.actor.clone();
        let role = g.role.clone();
        let mut project = g.project.clone().ok_or("No project loaded")?;

        let admin_count = project
            .users
            .iter()
            .filter(|u| u.enabled && u.security_level >= 1000)
            .count();
        let target = project.users.iter().find(|u| u.id == user_id);

        if let Some(target_u) = target {
            if target_u.security_level >= 1000 && admin_count <= 1 {
                return Err("Cannot delete the last remaining active Administrator account".into());
            }
        } else {
            return Err("User not found".into());
        }

        self.audit.append_required(
            &actor,
            role_str(&role),
            "user.delete_requested",
            &format!("User ID {user_id}"),
        )?;
        project.users.retain(|u| u.id != user_id);
        project.validate()?;
        project.recompute_hash();
        self.user_realm_store.persist_users(&project.users, true)?;
        g.project = Some(project);

        let actor = g.actor.clone();
        let level = g.security_level;
        drop(g);

        self.audit.append(
            &actor,
            &format!("L{level}"),
            "user.delete",
            &format!("User ID {user_id} deleted"),
        );

        Ok(())
    }

    /// Expire an idle session at a command boundary in every operating mode.
    pub fn expire_idle_session(&self) {
        let expired_identity = {
            let mut g = self.inner.write();
            let timeout_mins = g
                .project
                .as_ref()
                .map(|p| p.session_config.auto_logout_minutes)
                .unwrap_or(15);
            let expired = g.security_level > 0
                && timeout_mins > 0
                && (Utc::now() - g.last_activity_ts).num_seconds() >= i64::from(timeout_mins) * 60;
            if !expired {
                None
            } else {
                let identity = (g.actor.clone(), g.security_level);
                g.current_user = None;
                g.security_level = 0;
                g.role = Role::Viewer;
                g.actor = "guest".into();
                g.last_activity_ts = Utc::now();
                g.auth_epoch = g.auth_epoch.wrapping_add(1);
                g.mode = "runtime".into();
                Some(identity)
            }
        };
        if let Some((actor, level)) = expired_identity {
            self.audit.append(
                &actor,
                &format!("L{level}"),
                "auth.session_expired",
                "Idle timeout reached",
            );
        }
    }

    pub fn snapshot(&self) -> EngineSnapshot {
        let mut g = self.inner.write();
        let mut alarm_changed = refresh_stale_quality(&mut g);
        if refresh_system_tags(&mut g) {
            let alarms = g
                .project
                .as_ref()
                .map(|project| project.alarms.clone())
                .unwrap_or_default();
            alarm_changed |= evaluate_alarms(&mut g, &alarms);
        }
        if alarm_changed {
            let _ = self.alarm_state_store.persist_if_attached(&g);
        }
        let audit_status = self.audit.status_redacted();
        let (alarm_state_persisted, alarm_state_last_error) = self.alarm_state_store.status();
        let (user_realm_persisted, user_realm_last_error) = self.user_realm_store.status();
        let now = Utc::now();

        let tags: Vec<TagValue> = g
            .tags
            .values()
            .map(|t| {
                let age = (now - t.ts).num_milliseconds().max(0) as u64;
                TagValue {
                    tag_id: t.def.id.clone(),
                    value: t.value,
                    bool_value: t.bool_value,
                    quality: t.quality,
                    ts: t.ts,
                    age_ms: age,
                    raw: t.raw,
                }
            })
            .collect();
        let alarms: Vec<AlarmInstance> = g.alarms.values().cloned().collect();
        let alarms_suspended = alarms.iter().any(|a| a.evaluation_suspended);
        EngineSnapshot {
            connected: g.connected,
            device_id: g.device_id.clone(),
            last_error: g.last_error.clone(),
            poll_count: g.poll_count,
            last_poll_ms: g.last_poll_ms,
            tags,
            alarms,
            role: g.role.clone(),
            actor: g.actor.clone(),
            current_user: g.current_user.clone(),
            security_level: g.security_level,
            project_name: g.project.as_ref().map(|p| p.name.clone()),
            mode: g.mode.clone(),
            alarms_suspended,
            password_change_required: g
                .current_user
                .as_ref()
                .is_some_and(|u| u.password_change_required),
            requires_bootstrap: self.user_realm_store.bootstrap_available(
                g.project
                    .as_ref()
                    .is_some_and(|project| project.users.is_empty()),
            ),
            audit_chain_ok: audit_status.chain_ok,
            audit_persisted: audit_status.persisted,
            audit_last_error: audit_status.last_error,
            alarm_state_persisted,
            alarm_state_last_error,
            user_realm_persisted,
            user_realm_last_error,
        }
    }

    /// Drop every cached write connection, e.g. when the project changes.
    fn close_write_sessions(&self) {
        let sessions: Vec<_> = {
            let mut guard = self.write_sessions.lock();
            guard.drain().map(|(_, v)| v).collect()
        };
        if sessions.is_empty() {
            return;
        }
        self.rt.spawn(async move {
            for session in sessions {
                let mut guard = session.lock().await;
                if let Some(ctx) = guard.take() {
                    modbus::close(ctx).await;
                }
            }
        });
    }

    fn stop_polling_internal(&self) -> Result<(), String> {
        let mut g = self.inner.write();
        g.poll_epoch = g.poll_epoch.wrapping_add(1);
        if let Some(tx) = g.stop_tx.take() {
            let _ = tx.send(true);
        }
        if let Some(h) = g.poll_handle.take() {
            h.abort();
        }
        g.connected = false;
        g.device_id = None;
        mark_all_bad(&mut g);
        refresh_system_tags(&mut g);
        let alarms = g
            .project
            .as_ref()
            .map(|project| project.alarms.clone())
            .unwrap_or_default();
        let alarm_changed = evaluate_alarms(&mut g, &alarms);
        if alarm_changed {
            self.alarm_state_store.persist_if_attached(&g)?;
        }
        Ok(())
    }

    pub fn stop_polling(&self) -> Result<(), String> {
        self.expire_idle_session();
        {
            let g = self.inner.read();
            require_runtime_operator(&g, "stop polling")?;
        }
        self.stop_polling_internal()?;
        let g = self.inner.read();
        self.audit.append(
            &g.actor,
            role_str(&g.role),
            "poll.stop",
            "Polling stopped; PLC tag quality set to Bad",
        );
        Ok(())
    }

    pub fn start_polling(&self, device_id: Option<String>) -> Result<(), String> {
        self.expire_idle_session();
        {
            let g = self.inner.read();
            require_runtime_operator(&g, "start polling")?;
        }
        self.stop_polling_internal()?;
        let (project, device) = {
            let g = self.inner.read();
            let project = g
                .project
                .clone()
                .ok_or_else(|| "No project loaded".to_string())?;
            let device = if let Some(ref id) = device_id {
                project
                    .devices
                    .iter()
                    .find(|d| d.id == *id)
                    .cloned()
                    .ok_or_else(|| format!("Device not found: {id}"))?
            } else {
                project
                    .devices
                    .iter()
                    .find(|d| d.enabled)
                    .cloned()
                    .ok_or_else(|| "No enabled device in project".to_string())?
            };
            if !device.enabled {
                return Err(format!("Device is disabled: {}", device.id));
            }
            (project, device)
        };

        let (stop_tx, stop_rx) = watch::channel(false);
        let inner = self.inner.clone();
        let audit = self.audit.clone();
        let alarm_state_store = self.alarm_state_store.clone();
        let dev_id = device.id.clone();
        let poll_epoch;

        {
            let mut g = inner.write();
            poll_epoch = g.poll_epoch;
            g.device_id = Some(dev_id.clone());
            g.stop_tx = Some(stop_tx);
            g.last_error = None;
        }

        let handle = self.rt.spawn(async move {
            poll_loop(
                inner,
                audit,
                alarm_state_store,
                project,
                device,
                stop_rx,
                poll_epoch,
            )
            .await;
        });

        self.inner.write().poll_handle = Some(handle);
        self.audit.append(
            &self.inner.read().actor,
            role_str(&self.inner.read().role),
            "poll.start",
            &dev_id,
        );
        Ok(())
    }

    /// Obtain the serialized write session for a device, reconnecting if needed.
    async fn write_session(
        &self,
        device_id: &str,
    ) -> Arc<AsyncMutex<Option<modbus::ModbusContext>>> {
        let mut guard = self.write_sessions.lock();
        guard
            .entry(device_id.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(None)))
            .clone()
    }

    #[cfg(test)]
    pub async fn write_tag(&self, tag_id: &str, value: f64) -> Result<WriteReceipt, String> {
        self.write_tag_with_pin(tag_id, value, None).await
    }

    /// Authorize, optionally challenge, execute and observe one process write.
    ///
    /// The PIN is checked here, against the same signed-in identity and project
    /// epoch used for the write. No separate PIN-verification oracle exists.
    pub async fn write_tag_with_pin(
        &self,
        tag_id: &str,
        value: f64,
        pin: Option<&str>,
    ) -> Result<WriteReceipt, String> {
        self.expire_idle_session();
        if !value.is_finite() {
            let error = "Write value must be finite".to_string();
            self.audit_write_denied(tag_id, value, &error);
            return Err(error);
        }

        let authorization = {
            let mut g = self.inner.write();
            if refresh_stale_quality(&mut g) {
                if let Err(error) = self.alarm_state_store.persist_if_attached(&g) {
                    drop(g);
                    self.audit_write_denied(tag_id, value, &error);
                    return Err(error);
                }
            }
            match authorize_write_locked(&g, tag_id, pin) {
                Ok(authorization) => authorization,
                Err(error) => {
                    drop(g);
                    self.audit_write_denied(tag_id, value, &error);
                    return Err(error);
                }
            }
        };

        let addr = authorization.def.binding.address;
        let lock = {
            let mut locks = self.write_locks.lock();
            locks
                .entry((authorization.def.device_id.clone(), addr))
                .or_insert_with(|| Arc::new(AsyncMutex::new(())))
                .clone()
        };
        let _write_guard = lock.lock().await;

        // A queued command may have waited behind another write. Re-check all
        // gates, including timeout, mode, project epoch, quality and PIN.
        self.expire_idle_session();
        {
            let mut g = self.inner.write();
            if refresh_stale_quality(&mut g) {
                if let Err(error) = self.alarm_state_store.persist_if_attached(&g) {
                    drop(g);
                    self.audit_write_denied(tag_id, value, &error);
                    return Err(error);
                }
            }
            if let Err(error) = revalidate_write_locked(&g, &authorization, tag_id, pin) {
                drop(g);
                self.audit_write_denied(tag_id, value, &error);
                return Err(error);
            }
        }
        if let Err(error) = self.audit.append_required(
            &authorization.actor,
            role_str(&authorization.role),
            "tag.write_requested",
            &format!("{tag_id}={value}"),
        ) {
            let reason = format!("Durable audit unavailable; write blocked: {error}");
            self.audit_write_denied(tag_id, value, &reason);
            return Err(reason);
        }
        self.expire_idle_session();
        {
            let mut g = self.inner.write();
            if refresh_stale_quality(&mut g) {
                if let Err(error) = self.alarm_state_store.persist_if_attached(&g) {
                    drop(g);
                    self.audit_write_denied(tag_id, value, &error);
                    return Err(error);
                }
            }
            if let Err(error) = revalidate_write_locked(&g, &authorization, tag_id, pin) {
                drop(g);
                self.audit_write_denied(tag_id, value, &error);
                return Err(error);
            }
        }

        let def = authorization.def.clone();
        if def.binding.table == ModbusTable::Memory {
            let outcome = match normalize_internal_write(&def, value) {
                Ok(outcome) => outcome,
                Err(error) => {
                    self.audit_write_denied(tag_id, value, &error);
                    return Err(error);
                }
            };
            let alarm_persist_result = {
                let mut g = self.inner.write();
                if let Err(error) = revalidate_write_locked(&g, &authorization, tag_id, pin) {
                    drop(g);
                    self.audit_write_denied(tag_id, value, &error);
                    return Err(error);
                }
                let live = g.tags.get_mut(tag_id).ok_or("Live tag not found")?;
                live.value = outcome.engineering_value;
                live.bool_value = outcome.bool_value;
                live.quality = Quality::Good;
                live.ts = Utc::now();
                live.raw = outcome.raw;
                let project_alarms = g
                    .project
                    .as_ref()
                    .map(|p| p.alarms.clone())
                    .unwrap_or_default();
                let alarm_changed = evaluate_alarms(&mut g, &project_alarms);
                g.last_activity_ts = Utc::now();
                if alarm_changed {
                    self.alarm_state_store.persist_if_attached(&g)
                } else {
                    Ok(())
                }
            };
            if let Err(error) = alarm_persist_result {
                self.audit.append(
                    &authorization.actor,
                    role_str(&authorization.role),
                    "alarm.state_persist_failed",
                    &format!("{tag_id}: {error}"),
                );
                return Err(format!(
                    "Memory write was observed as {}, but alarm state persistence failed: {error}",
                    outcome.engineering_value
                ));
            }
            self.audit.append(
                &authorization.actor,
                role_str(&authorization.role),
                "tag.write",
                &format!(
                    "{tag_id}={value} (observed={}, protocol=memory)",
                    outcome.engineering_value
                ),
            );
            let matches = values_match(&def, value, outcome.engineering_value);
            if def.binding.verify_readback && !matches {
                return Err("Internal write verification mismatch".into());
            }
            return Ok(WriteReceipt {
                tag_id: tag_id.to_string(),
                requested_value: value,
                observed_value: outcome.engineering_value,
                raw_readback: outcome.raw,
                protocol: outcome.protocol.into(),
                verify_readback: def.binding.verify_readback,
                matches,
            });
        }

        let device = authorization.device.clone().ok_or("Device missing")?;
        let cfg = ConnectionConfig {
            host: device.host,
            port: device.port,
            unit_id: device.unit_id,
            timeout_ms: device.timeout_ms,
        };
        let session = self.write_session(&def.device_id).await;
        let outcome = self
            .rt
            .spawn({
                let def = def.clone();
                async move {
                    let mut ctx_guard = session.lock().await;
                    if ctx_guard.is_none() {
                        *ctx_guard = Some(modbus::connect(&cfg).await.map_err(|e| e.to_string())?);
                    }
                    let ctx = ctx_guard.as_mut().expect("connection established");
                    let result = perform_write(ctx, &cfg, &def, value).await;
                    if result.is_err() {
                        // A failed exchange may have desynchronized the socket.
                        if let Some(dead) = ctx_guard.take() {
                            modbus::close(dead).await;
                        }
                    }
                    result
                }
            })
            .await
            .map_err(|e| format!("write task join: {e}"))?;

        let WriteOutcome {
            raw,
            bool_value,
            engineering_value,
            protocol,
        } = match outcome {
            Ok(outcome) => outcome,
            Err(error) => {
                self.audit.append(
                    &authorization.actor,
                    role_str(&authorization.role),
                    "tag.write_failed",
                    &format!("{tag_id}={value}: {error}"),
                );
                return Err(error);
            }
        };

        self.expire_idle_session();
        let alarm_persist_result = {
            let mut g = self.inner.write();
            let mut alarm_changed = refresh_stale_quality(&mut g);
            if let Err(error) = revalidate_write_locked(&g, &authorization, tag_id, pin) {
                drop(g);
                self.audit.append(
                    &authorization.actor,
                    role_str(&authorization.role),
                    "tag.write_observed_after_auth_change",
                    &format!(
                        "{tag_id}={value} observed={engineering_value}, raw_readback={raw}: {error}"
                    ),
                );
                return Err(format!(
                    "PLC write was observed as {engineering_value}, but local authorization changed before commit: {error}"
                ));
            }
            let live = g.tags.get_mut(tag_id).ok_or("Live tag not found")?;
            live.raw = raw;
            live.value = engineering_value;
            live.bool_value = bool_value;
            live.quality = Quality::Good;
            live.ts = Utc::now();
            let project_alarms = g
                .project
                .as_ref()
                .map(|project| project.alarms.clone())
                .unwrap_or_default();
            alarm_changed |= evaluate_alarms(&mut g, &project_alarms);
            g.last_activity_ts = Utc::now();
            if alarm_changed {
                self.alarm_state_store.persist_if_attached(&g)
            } else {
                Ok(())
            }
        };
        if let Err(error) = alarm_persist_result {
            self.audit.append(
                &authorization.actor,
                role_str(&authorization.role),
                "alarm.state_persist_failed",
                &format!("{tag_id}: {error}"),
            );
            return Err(format!(
                "PLC write was observed as {engineering_value}, but alarm state persistence failed: {error}"
            ));
        }

        self.audit.append(
            &authorization.actor,
            role_str(&authorization.role),
            "tag.write",
            &format!(
                "{tag_id}={value} (observed={engineering_value}, raw_readback={raw}, protocol={protocol}, bit={:?}, verify_readback={})",
                def.binding.bit, def.binding.verify_readback
            ),
        );
        let matches = values_match(&def, value, engineering_value);
        if def.binding.verify_readback && !matches {
            return Err(format!(
                "Write read-back mismatch: requested {value}, observed {engineering_value}"
            ));
        }
        Ok(WriteReceipt {
            tag_id: tag_id.to_string(),
            requested_value: value,
            observed_value: engineering_value,
            raw_readback: raw,
            protocol: protocol.into(),
            verify_readback: def.binding.verify_readback,
            matches,
        })
    }

    fn audit_write_denied(&self, tag_id: &str, value: f64, reason: &str) {
        let g = self.inner.read();
        self.audit.append(
            &g.actor,
            role_str(&g.role),
            "tag.write_denied",
            &format!("{tag_id}={value}: {reason}"),
        );
    }

    pub fn ack_alarm(&self, def_id: &str) -> Result<(), String> {
        self.expire_idle_session();
        let mut g = self.inner.write();
        if !g.role.can_write() {
            return Err("Role cannot acknowledge alarms".into());
        }
        // The operator interacted with the plant, so the session is alive even
        // when the acknowledgement itself turns out to be a no-op.
        g.last_activity_ts = Utc::now();
        let actor = g.actor.clone();
        let role = g.role.clone();
        let mut candidate = g.alarms.get(def_id).cloned().ok_or("Alarm not found")?;
        let reset = matches!(candidate.state, AlarmState::ActiveAcked)
            && candidate.latched
            && !candidate.source_active;
        self.audit
            .append_required(&actor, role_str(&role), "alarm.ack_requested", def_id)?;
        match candidate.state {
            AlarmState::ActiveUnacked => candidate.state = AlarmState::ActiveAcked,
            AlarmState::ClearedUnacked => candidate.state = AlarmState::Inactive,
            AlarmState::ActiveAcked if candidate.latched && !candidate.source_active => {
                candidate.state = AlarmState::Inactive
            }
            _ => return Ok(()),
        };
        candidate.last_change = Utc::now();
        let mut persisted_alarms = g.alarms.clone();
        persisted_alarms.insert(def_id.to_string(), candidate.clone());
        self.alarm_state_store
            .persist_alarms(&g, &persisted_alarms)?;
        g.alarms.insert(def_id.to_string(), candidate);
        drop(g);
        let audit_action = if reset { "alarm.reset" } else { "alarm.ack" };
        self.audit
            .append(&actor, role_str(&role), audit_action, def_id);
        Ok(())
    }

    pub fn authorize_project_load(&self) -> Result<(), String> {
        self.expire_idle_session();
        let g = self.inner.read();
        require_engineer(&g, "load a project")
    }

    pub fn device_connection_config_for_test(
        &self,
        device_id: &str,
    ) -> Result<ConnectionConfig, String> {
        self.expire_idle_session();
        let g = self.inner.read();
        require_engineer(&g, "test a device connection")?;
        let project = g.project.as_ref().ok_or("No project loaded")?;
        let device = project
            .devices
            .iter()
            .find(|device| device.id == device_id)
            .ok_or_else(|| format!("Device not found: {device_id}"))?;
        Ok(ConnectionConfig {
            host: device.host.clone(),
            port: device.port,
            unit_id: device.unit_id,
            timeout_ms: device.timeout_ms,
        })
    }

    pub fn authorize_audit_read(&self) -> Result<(), String> {
        self.expire_idle_session();
        let g = self.inner.read();
        require_engineer(&g, "read or verify the audit trail")
    }

    pub fn authorize_audit_status(&self) -> Result<(), String> {
        self.expire_idle_session();
        let g = self.inner.read();
        if !g.role.can_write() {
            return Err(
                "Operator, Engineer or Administrator role is required for audit health".into(),
            );
        }
        Ok(())
    }
}

#[derive(Clone)]
struct WriteAuthorization {
    actor: String,
    role: Role,
    user_id: String,
    auth_epoch: u64,
    project_epoch: u64,
    poll_epoch: u64,
    def: TagDefinition,
    device: Option<DeviceConfig>,
}

fn authorize_write_locked(
    g: &EngineInner,
    tag_id: &str,
    pin: Option<&str>,
) -> Result<WriteAuthorization, String> {
    if !g.role.can_write() {
        return Err("Role cannot write process values".into());
    }
    if g.mode != "runtime" {
        return Err("Process writes are blocked outside Runtime mode".into());
    }
    let current = g.current_user.as_ref().ok_or("No user is signed in")?;
    if current.password_change_required {
        return Err("Change the default password before writing to the process".into());
    }
    let project = g.project.as_ref().ok_or("No project")?;
    let tag = project
        .tags
        .iter()
        .find(|tag| tag.id == tag_id)
        .ok_or("Tag not found")?
        .clone();
    if !tag.binding.writable {
        return Err("Tag is not writable".into());
    }
    if g.security_level < tag.binding.min_security_level {
        return Err(format!(
            "Security level {} required to write this tag (current: {})",
            tag.binding.min_security_level, g.security_level
        ));
    }
    let live = g.tags.get(tag_id).ok_or("Live tag not found")?;
    if live.quality != Quality::Good {
        return Err(format!(
            "Tag quality must be Good before write (current: {:?})",
            live.quality
        ));
    }
    if project.session_config.pin_challenge_on_write {
        let supplied = pin
            .map(str::trim)
            .filter(|pin| !pin.is_empty())
            .ok_or("This project requires a PIN in the same write request")?;
        let account = project
            .users
            .iter()
            .find(|user| user.enabled && user.id == current.id)
            .ok_or("Signed-in account is no longer active")?;
        let pin_hash = account
            .pin_hash
            .as_ref()
            .ok_or("The signed-in account has no PIN configured")?;
        if !credentials::verify_secret(supplied, pin_hash, &account.salt).is_accepted() {
            return Err("PIN challenge failed".into());
        }
    }
    let device = if tag.binding.table == ModbusTable::Memory {
        None
    } else {
        let device = project
            .devices
            .iter()
            .find(|device| device.id == tag.device_id)
            .ok_or("Device missing")?;
        if !device.enabled {
            return Err(format!("Device is disabled: {}", device.id));
        }
        Some(device.clone())
    };
    Ok(WriteAuthorization {
        actor: g.actor.clone(),
        role: g.role.clone(),
        user_id: current.id.clone(),
        auth_epoch: g.auth_epoch,
        project_epoch: g.project_epoch,
        poll_epoch: g.poll_epoch,
        def: tag,
        device,
    })
}

fn revalidate_write_locked(
    g: &EngineInner,
    original: &WriteAuthorization,
    tag_id: &str,
    pin: Option<&str>,
) -> Result<(), String> {
    if g.auth_epoch != original.auth_epoch {
        return Err("Session identity, privilege or mode changed".into());
    }
    if g.project_epoch != original.project_epoch {
        return Err("Project changed".into());
    }
    if g.poll_epoch != original.poll_epoch {
        return Err("Polling was stopped or switched".into());
    }
    let current = authorize_write_locked(g, tag_id, pin)?;
    if current.user_id != original.user_id {
        return Err("Signed-in user changed".into());
    }
    Ok(())
}

/// Which credential satisfied a login attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoginKind {
    Password,
}

fn login_is_throttled(g: &mut EngineInner, key: &str) -> bool {
    #[cfg(all(debug_assertions, not(test)))]
    {
        let _ = (g, key);
        false
    }
    #[cfg(not(all(debug_assertions, not(test))))]
    {
        let now = Utc::now();
        g.login_throttle.retain(|_, state| {
            state.blocked_until.is_some_and(|until| until > now) || state.failures > 0
        });
        g.login_throttle
            .get(key)
            .and_then(|state| state.blocked_until)
            .is_some_and(|until| until > now)
    }
}

fn record_login_failure(g: &mut EngineInner, key: &str) {
    const MAX_TRACKED_IDENTITIES: usize = 1_024;
    if !g.login_throttle.contains_key(key) && g.login_throttle.len() >= MAX_TRACKED_IDENTITIES {
        let is_configured_user = g.project.as_ref().is_some_and(|project| {
            project
                .users
                .iter()
                .any(|user| user.username.eq_ignore_ascii_case(key))
        });
        if !is_configured_user {
            // Unknown names are cheap to reject and must not evict throttles
            // for real accounts from the bounded map.
            return;
        }
        let unconfigured_victim = g.login_throttle.keys().find(|candidate| {
            !g.project.as_ref().is_some_and(|project| {
                project
                    .users
                    .iter()
                    .any(|user| user.username.eq_ignore_ascii_case(candidate))
            })
        });
        if let Some(victim) = unconfigured_victim.cloned() {
            g.login_throttle.remove(&victim);
        } else if let Some(victim) = g.login_throttle.keys().next().cloned() {
            g.login_throttle.remove(&victim);
        }
    }
    let state = g
        .login_throttle
        .entry(key.to_string())
        .or_insert(LoginThrottle {
            failures: 0,
            blocked_until: None,
        });
    state.failures = state.failures.saturating_add(1);
    if state.failures >= 5 {
        let exponent = state.failures.saturating_sub(5).min(5);
        let delay_seconds = 30_i64.saturating_mul(1_i64 << exponent);
        state.blocked_until = Some(Utc::now() + chrono::Duration::seconds(delay_seconds));
    }
}

fn require_administrator(g: &EngineInner) -> Result<(), String> {
    if g.security_level < 1000 {
        return Err(
            "Administrator permission (Security Level 1000) is required to manage users".into(),
        );
    }
    if g.current_user
        .as_ref()
        .is_some_and(|u| u.password_change_required)
    {
        return Err("Change the default password before administering users".into());
    }
    Ok(())
}

fn require_engineer(g: &EngineInner, action: &str) -> Result<(), String> {
    if !g.role.can_edit_project() {
        return Err(format!(
            "Engineer or Administrator role is required to {action}"
        ));
    }
    if g.current_user
        .as_ref()
        .is_some_and(|user| user.password_change_required)
    {
        return Err("Change the default password before this action".into());
    }
    Ok(())
}

fn require_runtime_operator(g: &EngineInner, action: &str) -> Result<(), String> {
    if !g.role.can_write() {
        return Err(format!(
            "Operator, Engineer or Administrator role is required to {action}"
        ));
    }
    if g.mode != "runtime" {
        return Err(format!("{action} is blocked outside Runtime mode"));
    }
    if g.current_user
        .as_ref()
        .is_some_and(|user| user.password_change_required)
    {
        return Err("Change the default password before controlling polling".into());
    }
    Ok(())
}

fn validate_project_file_path(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err("Project save path must be absolute".into());
    }
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Project save path has no valid filename")?;
    let lower = filename.to_ascii_lowercase();
    if !lower.ends_with(".proscada.json") && !lower.ends_with(".json") {
        return Err("Project filename must end with .proscada.json or .json".into());
    }
    if path
        .symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err("Refusing to replace a symbolic-link project path".into());
    }
    let parent = path
        .parent()
        .ok_or("Project save path has no parent directory")?;
    if !parent.is_dir() {
        return Err(format!(
            "Project save directory does not exist: {}",
            parent.display()
        ));
    }
    Ok(path.to_path_buf())
}

fn paths_refer_to_same_file(left: &Path, right: &Path) -> bool {
    left == right
        || left
            .canonicalize()
            .ok()
            .zip(right.canonicalize().ok())
            .is_some_and(|(left, right)| left == right)
}

fn persisted_alarm_states(alarms: &HashMap<String, AlarmInstance>) -> Vec<PersistedAlarmState> {
    let mut states: Vec<_> = alarms
        .values()
        .map(|alarm| PersistedAlarmState {
            def_id: alarm.def_id.clone(),
            state: alarm.state.clone(),
            source_active: alarm.source_active,
            active_since: alarm.active_since,
            last_change: alarm.last_change,
            pending_active_since: alarm.pending_active_since,
            pending_clear_since: alarm.pending_clear_since,
        })
        .collect();
    states.sort_by(|left, right| left.def_id.cmp(&right.def_id));
    states
}

fn alarm_definitions_hash(project: &ScadaProject) -> Result<String, String> {
    let bytes = serde_json::to_vec(&project.alarms)
        .map_err(|error| format!("encode alarm definitions: {error}"))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn alarm_journal_hash(journal: &AlarmStateJournal) -> Result<String, String> {
    let mut canonical = journal.clone();
    canonical.content_hash.clear();
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|error| format!("encode alarm-state journal: {error}"))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn build_alarm_journal(
    g: &EngineInner,
    alarms: &HashMap<String, AlarmInstance>,
) -> Result<AlarmStateJournal, String> {
    let project = g.project.as_ref().ok_or("No project loaded")?;
    let mut journal = AlarmStateJournal {
        version: 1,
        project_id: project.id.clone(),
        definitions_hash: alarm_definitions_hash(project)?,
        alarms: persisted_alarm_states(alarms),
        content_hash: String::new(),
    };
    journal.content_hash = alarm_journal_hash(&journal)?;
    Ok(journal)
}

fn validate_alarm_journal(journal: &AlarmStateJournal, g: &EngineInner) -> Result<(), String> {
    if journal.version != 1 {
        return Err(format!(
            "Unsupported alarm-state journal version {}",
            journal.version
        ));
    }
    if journal.content_hash != alarm_journal_hash(journal)? {
        return Err("Alarm-state journal hash verification failed".into());
    }
    let project = g.project.as_ref().ok_or("No project loaded")?;
    if journal.project_id != project.id {
        return Err(format!(
            "Alarm-state journal belongs to project {}, current project is {}",
            journal.project_id, project.id
        ));
    }
    if journal.definitions_hash != alarm_definitions_hash(project)? {
        return Err("Alarm-state journal does not match current alarm definitions".into());
    }
    if journal.alarms.len() != g.alarms.len() {
        return Err("Alarm-state journal has a different alarm set".into());
    }
    let mut ids = std::collections::HashSet::new();
    let latest_allowed = Utc::now() + chrono::Duration::minutes(5);
    for alarm in &journal.alarms {
        if !ids.insert(alarm.def_id.as_str()) || !g.alarms.contains_key(&alarm.def_id) {
            return Err(format!(
                "Alarm-state journal contains unknown or duplicate alarm {}",
                alarm.def_id
            ));
        }
        if alarm.last_change > latest_allowed
            || alarm.active_since.is_some_and(|ts| ts > latest_allowed)
            || alarm
                .pending_active_since
                .is_some_and(|ts| ts > latest_allowed)
            || alarm
                .pending_clear_since
                .is_some_and(|ts| ts > latest_allowed)
        {
            return Err(format!(
                "Alarm-state journal contains a future timestamp for {}",
                alarm.def_id
            ));
        }
    }
    Ok(())
}

fn user_realm_hash(journal: &UserRealmJournal) -> Result<String, String> {
    let mut canonical = journal.clone();
    canonical.content_hash.clear();
    let bytes =
        serde_json::to_vec(&canonical).map_err(|error| format!("encode user realm: {error}"))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn build_user_realm_journal(
    users: &[UserAccount],
    provisioning_closed: bool,
) -> Result<UserRealmJournal, String> {
    let mut journal = UserRealmJournal {
        version: 1,
        provisioning_closed,
        users: users.to_vec(),
        content_hash: String::new(),
    };
    journal.content_hash = user_realm_hash(&journal)?;
    Ok(journal)
}

fn validate_user_realm_journal(
    journal: &UserRealmJournal,
    project: &ScadaProject,
) -> Result<(), String> {
    if journal.version != 1 {
        return Err(format!(
            "Unsupported user-realm version {}",
            journal.version
        ));
    }
    if journal.content_hash != user_realm_hash(journal)? {
        return Err("User-realm hash verification failed".into());
    }
    if !journal.provisioning_closed && !journal.users.is_empty() {
        return Err("Open user realm unexpectedly contains accounts".into());
    }
    if journal.users.iter().any(|user| {
        user.password_hash.trim().is_empty()
            || user.salt.trim().is_empty()
            || user
                .pin_hash
                .as_ref()
                .is_some_and(|pin_hash| pin_hash.trim().is_empty())
    }) {
        return Err("User realm contains an empty credential record".into());
    }
    let mut candidate = project.clone();
    candidate.users = journal.users.clone();
    candidate.validate()?;
    Ok(())
}

fn atomic_write_protected_state(target: &Path, bytes: &[u8], label: &str) -> Result<(), String> {
    let parent = target
        .parent()
        .ok_or_else(|| format!("{label} path has no parent"))?;
    let filename = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{label} path has no filename"))?;
    if target
        .symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err(format!("Refusing symbolic-link {label}"));
    }
    let temporary = parent.join(format!(".{filename}.{}.tmp", uuid::Uuid::new_v4().simple()));
    let result = (|| -> Result<(), String> {
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = options
            .open(&temporary)
            .map_err(|error| format!("create {label} temporary file: {error}"))?;
        file.write_all(bytes)
            .map_err(|error| format!("write {label}: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("synchronize {label}: {error}"))?;
        drop(file);
        if target
            .symlink_metadata()
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err(format!("Refusing symbolic-link {label}"));
        }
        replace_target(&temporary, target, target)?;
        restrict_secret_file_permissions(target)?;
        sync_directory(parent)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

fn restrict_secret_file_permissions(path: &Path) -> Result<(), String> {
    let metadata = path
        .symlink_metadata()
        .map_err(|error| format!("inspect protected state file: {error}"))?;
    if metadata.file_type().is_symlink() {
        return Err("Refusing symbolic-link protected state file".into());
    }
    if !metadata.is_file() {
        return Err("Protected state path is not a regular file".into());
    }
    #[cfg(unix)]
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|error| format!("restrict protected state permissions: {error}"))?;
    Ok(())
}

fn atomic_write_project(target: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = target.parent().ok_or("Project path has no parent")?;
    let filename = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Project path has no filename")?;
    let nonce = uuid::Uuid::new_v4().simple();
    let temporary = parent.join(format!(".{filename}.{nonce}.tmp"));
    let backup = PathBuf::from(format!("{}.bak", target.display()));

    let write_result = (|| -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|error| format!("Create temporary project file: {error}"))?;
        file.write_all(bytes)
            .map_err(|error| format!("Write temporary project file: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("Synchronize temporary project file: {error}"))?;
        drop(file);

        if target.exists() {
            let backup_temp = parent.join(format!(".{filename}.{nonce}.bak.tmp"));
            let mut source =
                File::open(target).map_err(|error| format!("Open previous project: {error}"))?;
            let mut destination = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&backup_temp)
                .map_err(|error| format!("Create temporary project backup: {error}"))?;
            std::io::copy(&mut source, &mut destination)
                .map_err(|error| format!("Copy previous project to backup: {error}"))?;
            destination
                .sync_all()
                .map_err(|error| format!("Synchronize project backup: {error}"))?;
            drop(destination);
            replace_backup(&backup_temp, &backup)?;
        }

        replace_target(&temporary, target, &backup)?;
        sync_directory(parent)?;
        Ok(())
    })();

    if write_result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    write_result
}

fn replace_backup(source: &Path, backup: &Path) -> Result<(), String> {
    if !backup.exists() {
        return std::fs::rename(source, backup)
            .map_err(|error| format!("Install project backup: {error}"));
    }
    let displaced = backup.with_extension(format!(
        "{}.old",
        backup
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("bak")
    ));
    if displaced.exists() {
        std::fs::remove_file(&displaced)
            .map_err(|error| format!("Remove stale backup staging file: {error}"))?;
    }
    std::fs::rename(backup, &displaced)
        .map_err(|error| format!("Stage previous project backup: {error}"))?;
    if let Err(error) = std::fs::rename(source, backup) {
        let _ = std::fs::rename(&displaced, backup);
        return Err(format!("Install project backup: {error}"));
    }
    let _ = std::fs::remove_file(displaced);
    Ok(())
}

#[cfg(not(windows))]
fn replace_target(source: &Path, target: &Path, _backup: &Path) -> Result<(), String> {
    std::fs::rename(source, target).map_err(|error| format!("Atomically replace file: {error}"))
}

#[cfg(windows)]
fn replace_target(source: &Path, target: &Path, _backup: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let source_wide: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let target_wide: Vec<u16> = target.as_os_str().encode_wide().chain(Some(0)).collect();
    // MoveFileExW with REPLACE_EXISTING avoids the remove+rename data-loss
    // window, while WRITE_THROUGH waits for the replacement to reach storage.
    let replaced = unsafe {
        MoveFileExW(
            source_wide.as_ptr(),
            target_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if replaced == 0 {
        return Err(format!(
            "Atomically replace file: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("Synchronize project directory: {error}"))
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn refresh_stale_quality(g: &mut EngineInner) -> bool {
    let now = Utc::now();
    let mut changed = false;
    for tag in g.tags.values_mut() {
        let internal = matches!(
            tag.def.binding.table,
            ModbusTable::Memory | ModbusTable::System
        );
        let age_ms = (now - tag.ts).num_milliseconds().max(0) as u64;
        if !internal && tag.quality == Quality::Good && age_ms > STALE_AFTER_MS {
            tag.quality = Quality::Uncertain;
            changed = true;
        }
    }
    if changed {
        let alarms = g
            .project
            .as_ref()
            .map(|project| project.alarms.clone())
            .unwrap_or_default();
        evaluate_alarms(g, &alarms)
    } else {
        false
    }
}

fn refresh_system_tags(g: &mut EngineInner) -> bool {
    let now = Utc::now();
    let connected = g.connected;
    let poll_count = g.poll_count as f64;
    let last_poll_ms = g.last_poll_ms as f64;
    let security_level = f64::from(g.security_level);
    let runtime_mode = g.mode == "runtime";
    let mut changed = false;
    for tag in g.tags.values_mut() {
        let value = match tag.def.id.as_str() {
            "system.connected" => f64::from(u8::from(connected)),
            "system.poll_count" => poll_count,
            "system.last_poll_ms" => last_poll_ms,
            "system.security_level" => security_level,
            "system.mode" => f64::from(u8::from(runtime_mode)),
            _ => continue,
        };
        let bool_value = value != 0.0;
        changed |=
            tag.value != value || tag.bool_value != bool_value || tag.quality != Quality::Good;
        tag.value = value;
        tag.bool_value = bool_value;
        tag.quality = Quality::Good;
        tag.ts = now;
        tag.raw = value as u16;
    }
    changed
}

fn normalize_internal_initial(def: &TagDefinition) -> Result<f64, String> {
    let Some(initial) = def.initial_value.as_deref() else {
        return Ok(0.0);
    };
    let value = if def.data_type == TagDataType::Bool {
        match initial.trim().to_ascii_lowercase().as_str() {
            "true" | "1" => 1.0,
            "false" | "0" => 0.0,
            _ => return Err("Invalid bool initial value".into()),
        }
    } else {
        initial
            .trim()
            .parse::<f64>()
            .map_err(|_| "Invalid numeric initial value".to_string())?
    };
    Ok(normalize_internal_write(def, value)?.engineering_value)
}

fn normalize_internal_write(def: &TagDefinition, value: f64) -> Result<WriteOutcome, String> {
    if def.scale == 0.0 {
        return Err("Tag scale cannot be zero".into());
    }
    let scaled = (value - def.offset) / def.scale;
    let words = codec::encode(def.data_type, def.binding.word_order, scaled)?;
    let decoded = codec::decode(def.data_type, def.binding.word_order, &words)
        .ok_or("Internal value cannot be decoded")?;
    let engineering_value = decoded * def.scale + def.offset;
    Ok(WriteOutcome {
        raw: words.first().copied().unwrap_or(0),
        bool_value: engineering_value != 0.0,
        engineering_value,
        protocol: "memory",
    })
}

fn values_match(def: &TagDefinition, requested: f64, observed: f64) -> bool {
    if def.data_type == TagDataType::Bool {
        return (requested != 0.0) == (observed != 0.0);
    }
    let expected = normalize_internal_write(def, requested)
        .map(|outcome| outcome.engineering_value)
        .unwrap_or(requested);
    let tolerance = expected.abs().max(observed.abs()).max(1.0) * f64::EPSILON * 8.0;
    (expected - observed).abs() <= tolerance
}

fn alarm_logic_compatible(old: &AlarmDefinition, new: &AlarmDefinition) -> bool {
    old.id == new.id
        && old.tag_id == new.tag_id
        && old.when_true == new.when_true
        && old.hi_limit == new.hi_limit
        && old.lo_limit == new.lo_limit
        && old.deadband == new.deadband
        && old.on_delay_ms == new.on_delay_ms
        && old.off_delay_ms == new.off_delay_ms
        && old.latching == new.latching
}

fn new_alarm_instance(def: &AlarmDefinition) -> AlarmInstance {
    AlarmInstance {
        def_id: def.id.clone(),
        name: def.name.clone(),
        message: def.message.clone(),
        priority: def.priority.clone(),
        group_id: def.group_id.clone(),
        state: AlarmState::Inactive,
        source_active: false,
        latched: def.latching,
        active_since: None,
        last_change: Utc::now(),
        evaluation_suspended: true,
        suspended_reason: Some("No live data yet".into()),
        suspended_since: Some(Utc::now()),
        pending_active_since: None,
        pending_clear_since: None,
    }
}

struct WriteOutcome {
    raw: u16,
    bool_value: bool,
    engineering_value: f64,
    protocol: &'static str,
}

/// Execute one write on an established connection.
///
/// Every branch reads the target back, so the reported value is what the PLC
/// actually holds, never what the UI asked for.
async fn perform_write(
    ctx: &mut modbus::ModbusContext,
    cfg: &ConnectionConfig,
    def: &TagDefinition,
    value: f64,
) -> Result<WriteOutcome, String> {
    let addr = def.binding.address;
    let verify_readback = def.binding.verify_readback;
    let timeout_ms = cfg.timeout_ms;

    match def.binding.table {
        ModbusTable::Holding => {
            if let Some(bit) = def.binding.bit {
                if !matches!(def.data_type, TagDataType::Bool) {
                    return Err("Bit binding requires bool data_type".into());
                }
                if bit > 15 {
                    return Err(format!("Holding-register bit must be 0..15, got {bit}"));
                }
                let requested = value != 0.0;
                let readback = match def.binding.bit_write_mode {
                    BitWriteMode::MaskWrite => modbus::write_holding_bit_masked(
                        ctx,
                        addr,
                        bit,
                        requested,
                        timeout_ms,
                        verify_readback,
                    )
                    .await
                    .map_err(|e| e.to_string())?,
                    BitWriteMode::ReadModifyWrite => {
                        if !def.binding.single_writer {
                            return Err(
                                "Read-modify-write requires binding.single_writer=true; use FC22 or a dedicated PLC coil"
                                    .into(),
                            );
                        }
                        modbus::write_holding_bit_rmw(
                            ctx,
                            addr,
                            bit,
                            requested,
                            timeout_ms,
                            verify_readback,
                        )
                        .await
                        .map_err(|e| e.to_string())?
                    }
                };
                let actual = ((readback >> bit) & 1) == 1;
                return Ok(WriteOutcome {
                    raw: readback,
                    bool_value: actual,
                    engineering_value: if actual { 1.0 } else { 0.0 },
                    protocol: match def.binding.bit_write_mode {
                        BitWriteMode::MaskWrite => "FC22+FC03",
                        BitWriteMode::ReadModifyWrite => "FC03+FC06+FC03(single-writer)",
                    },
                });
            }

            if def.scale == 0.0 {
                return Err("Tag scale cannot be zero".into());
            }
            // Out-of-range values are rejected, never clamped: a silently
            // clamped setpoint is indistinguishable from a successful write.
            let scaled = (value - def.offset) / def.scale;
            let words = codec::encode(def.data_type, def.binding.word_order, scaled)?;
            let (readback, protocol) = if words.len() == 1 {
                (
                    vec![
                        modbus::write_holding(ctx, addr, words[0], timeout_ms, verify_readback)
                            .await
                            .map_err(|e| e.to_string())?,
                    ],
                    "FC06+FC03",
                )
            } else {
                (
                    modbus::write_holding_block(ctx, addr, &words, timeout_ms, verify_readback)
                        .await
                        .map_err(|e| e.to_string())?,
                    "FC16+FC03",
                )
            };
            let decoded = codec::decode(def.data_type, def.binding.word_order, &readback)
                .ok_or("Read-back is too short to decode")?;
            let engineering_value = decoded * def.scale + def.offset;
            Ok(WriteOutcome {
                raw: readback[0],
                bool_value: engineering_value != 0.0,
                engineering_value,
                protocol,
            })
        }
        ModbusTable::Coil => {
            if def.binding.bit.is_some() {
                return Err("Coil bindings must not define a register bit index".into());
            }
            if !matches!(def.data_type, TagDataType::Bool) {
                return Err("Coil binding requires bool data_type".into());
            }
            let requested = value != 0.0;
            let readback = modbus::write_coil(ctx, addr, requested, timeout_ms, verify_readback)
                .await
                .map_err(|e| e.to_string())?;
            Ok(WriteOutcome {
                raw: u16::from(readback),
                bool_value: readback,
                engineering_value: if readback { 1.0 } else { 0.0 },
                protocol: "FC05+FC01",
            })
        }
        ModbusTable::Input | ModbusTable::Discrete | ModbusTable::System => {
            Err("Input registers, discrete inputs, and system tags are read-only".into())
        }
        ModbusTable::Memory => {
            Err("Memory tags are stored in-memory in application runtime".into())
        }
    }
}

fn role_str(r: &Role) -> &'static str {
    match r {
        Role::Viewer => "viewer",
        Role::Operator => "operator",
        Role::Engineer => "engineer",
        Role::Administrator => "administrator",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReadBlock {
    table: ModbusTable,
    start: u16,
    quantity: u16,
}

fn build_read_plan(project: &ScadaProject, device_id: &str) -> Vec<ReadBlock> {
    let mut plan = Vec::new();
    for table in [
        ModbusTable::Holding,
        ModbusTable::Input,
        ModbusTable::Coil,
        ModbusTable::Discrete,
    ] {
        // Each tag claims every register it actually occupies, so a 32/64-bit
        // value is never read one register short.
        let mut addresses: Vec<u16> = project
            .tags
            .iter()
            .filter(|tag| tag.device_id == device_id && tag.binding.table == table)
            .flat_map(|tag| {
                let span = match table {
                    ModbusTable::Holding | ModbusTable::Input => {
                        tag.data_type.register_count().unwrap_or(1)
                    }
                    _ => 1,
                };
                let start = tag.binding.address;
                (0..span).filter_map(move |offset| start.checked_add(offset))
            })
            .collect();
        addresses.sort_unstable();
        addresses.dedup();
        let Some(&first) = addresses.first() else {
            continue;
        };
        let max_quantity = match table {
            ModbusTable::Holding | ModbusTable::Input => 120_u16,
            ModbusTable::Coil | ModbusTable::Discrete => 1000_u16,
            ModbusTable::Memory | ModbusTable::System => continue,
        };
        let mut start = first;
        let mut last = first;
        for address in addresses.into_iter().skip(1) {
            let candidate_quantity = address.saturating_sub(start).saturating_add(1);
            if address == last.saturating_add(1) && candidate_quantity <= max_quantity {
                last = address;
                continue;
            }
            plan.push(ReadBlock {
                table,
                start,
                quantity: last - start + 1,
            });
            start = address;
            last = address;
        }
        plan.push(ReadBlock {
            table,
            start,
            quantity: last - start + 1,
        });
    }
    plan
}

async fn poll_loop(
    inner: Arc<RwLock<EngineInner>>,
    _audit: Arc<AuditLog>,
    alarm_state_store: Arc<AlarmStateStore>,
    project: ScadaProject,
    device: DeviceConfig,
    mut stop_rx: watch::Receiver<bool>,
    poll_epoch: u64,
) {
    let cfg = ConnectionConfig {
        host: device.host.clone(),
        port: device.port,
        unit_id: device.unit_id,
        timeout_ms: device.timeout_ms,
    };

    let poll_ms = device.poll_ms.max(100);
    let read_plan = build_read_plan(&project, &device.id);
    let mut ctx = match modbus::connect(&cfg).await {
        Ok(c) => {
            let mut g = inner.write();
            if g.poll_epoch != poll_epoch {
                return;
            }
            g.connected = true;
            g.last_error = None;
            Some(c)
        }
        Err(e) => {
            let mut g = inner.write();
            if g.poll_epoch != poll_epoch {
                return;
            }
            g.connected = false;
            g.last_error = Some(e.to_string());
            mark_all_bad(&mut g);
            None
        }
    };

    loop {
        if *stop_rx.borrow() || inner.read().poll_epoch != poll_epoch {
            break;
        }

        let started = Instant::now();

        if ctx.is_none() {
            match modbus::connect(&cfg).await {
                Ok(c) => {
                    ctx = Some(c);
                    let mut g = inner.write();
                    if g.poll_epoch != poll_epoch {
                        break;
                    }
                    g.connected = true;
                    g.last_error = None;
                }
                Err(e) => {
                    let mut g = inner.write();
                    if g.poll_epoch != poll_epoch {
                        break;
                    }
                    g.connected = false;
                    g.last_error = Some(e.to_string());
                    mark_all_bad(&mut g);
                }
            }
        }

        if let Some(ref mut c) = ctx {
            let mut cycle_error = None;
            for block in &read_plan {
                let result = match block.table {
                    ModbusTable::Holding => {
                        match modbus::read_holding(
                            c,
                            block.start,
                            block.quantity,
                            device.timeout_ms,
                        )
                        .await
                        {
                            Ok(values) => {
                                let mut g = inner.write();
                                if g.poll_epoch != poll_epoch {
                                    return;
                                }
                                apply_register_values(&mut g, &project, &device.id, block, &values);
                                Ok(())
                            }
                            Err(error) => Err(error),
                        }
                    }
                    ModbusTable::Input => {
                        match modbus::read_input(c, block.start, block.quantity, device.timeout_ms)
                            .await
                        {
                            Ok(values) => {
                                let mut g = inner.write();
                                if g.poll_epoch != poll_epoch {
                                    return;
                                }
                                apply_register_values(&mut g, &project, &device.id, block, &values);
                                Ok(())
                            }
                            Err(error) => Err(error),
                        }
                    }
                    ModbusTable::Coil => {
                        match modbus::read_coils(c, block.start, block.quantity, device.timeout_ms)
                            .await
                        {
                            Ok(values) => {
                                let mut g = inner.write();
                                if g.poll_epoch != poll_epoch {
                                    return;
                                }
                                apply_bit_values(&mut g, &project, &device.id, block, &values);
                                Ok(())
                            }
                            Err(error) => Err(error),
                        }
                    }
                    ModbusTable::Discrete => {
                        match modbus::read_discrete(
                            c,
                            block.start,
                            block.quantity,
                            device.timeout_ms,
                        )
                        .await
                        {
                            Ok(values) => {
                                let mut g = inner.write();
                                if g.poll_epoch != poll_epoch {
                                    return;
                                }
                                apply_bit_values(&mut g, &project, &device.id, block, &values);
                                Ok(())
                            }
                            Err(error) => Err(error),
                        }
                    }
                    ModbusTable::Memory | ModbusTable::System => Ok(()),
                };
                if let Err(error) = result {
                    cycle_error = Some(error);
                    break;
                }
            }
            if let Some(error) = cycle_error {
                ctx = None;
                let mut g = inner.write();
                if g.poll_epoch != poll_epoch {
                    break;
                }
                g.connected = false;
                g.last_error = Some(error.to_string());
                mark_all_bad(&mut g);
                refresh_system_tags(&mut g);
                // Alarms are still evaluated so that every instance is flagged
                // as suspended instead of silently freezing on its last state.
                if evaluate_alarms(&mut g, &project.alarms) {
                    let _ = alarm_state_store.persist_if_attached(&g);
                }
            } else {
                let mut g = inner.write();
                if g.poll_epoch != poll_epoch {
                    break;
                }
                g.connected = true;
                g.last_error = None;
                g.poll_count = g.poll_count.wrapping_add(1);
                g.last_poll_ms = started.elapsed().as_millis() as u64;
                refresh_system_tags(&mut g);
                if evaluate_alarms(&mut g, &project.alarms) {
                    let _ = alarm_state_store.persist_if_attached(&g);
                }
            }
        } else {
            let mut g = inner.write();
            if g.poll_epoch != poll_epoch {
                break;
            }
            refresh_system_tags(&mut g);
            if evaluate_alarms(&mut g, &project.alarms) {
                let _ = alarm_state_store.persist_if_attached(&g);
            }
        }

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(poll_ms)) => {}
            _ = stop_rx.changed() => {
                if *stop_rx.borrow() { break; }
            }
        }
    }

    let mut g = inner.write();
    if g.poll_epoch != poll_epoch {
        return;
    }
    g.connected = false;
    mark_all_bad(&mut g);
    refresh_system_tags(&mut g);
    if evaluate_alarms(&mut g, &project.alarms) {
        let _ = alarm_state_store.persist_if_attached(&g);
    }
}

fn mark_all_bad(g: &mut EngineInner) {
    for t in g.tags.values_mut() {
        if t.def.binding.table != ModbusTable::Memory && t.def.binding.table != ModbusTable::System
        {
            t.quality = Quality::Bad;
        }
    }
}

fn apply_register_values(
    g: &mut EngineInner,
    project: &ScadaProject,
    device_id: &str,
    block: &ReadBlock,
    values: &[u16],
) {
    let now = Utc::now();
    for def in &project.tags {
        if def.device_id != device_id || def.binding.table != block.table {
            continue;
        }
        let addr = def.binding.address;
        if addr < block.start {
            continue;
        }
        let idx = (addr - block.start) as usize;
        if idx >= values.len() {
            continue;
        }
        let raw = values[idx];

        if matches!(def.data_type, TagDataType::Bool) {
            let Some(live) = g.tags.get_mut(&def.id) else {
                continue;
            };
            let bit = u16::from(def.binding.bit.unwrap_or(0));
            let b = (raw >> bit) & 1 == 1;
            live.raw = raw;
            live.ts = now;
            live.quality = Quality::Good;
            live.bool_value = b;
            live.value = if b { 1.0 } else { 0.0 };
            continue;
        }

        // A wide value spanning past the end of this block is not decoded from
        // a truncated buffer; it keeps its previous quality until a complete
        // read arrives.
        let Some(decoded) = codec::decode(def.data_type, def.binding.word_order, &values[idx..])
        else {
            continue;
        };
        let Some(live) = g.tags.get_mut(&def.id) else {
            continue;
        };
        let v = decoded * def.scale + def.offset;
        live.raw = raw;
        live.ts = now;
        live.quality = Quality::Good;
        live.value = v;
        live.bool_value = v != 0.0;
    }
}

fn apply_bit_values(
    g: &mut EngineInner,
    project: &ScadaProject,
    device_id: &str,
    block: &ReadBlock,
    values: &[bool],
) {
    let now = Utc::now();
    for def in &project.tags {
        if def.device_id != device_id || def.binding.table != block.table {
            continue;
        }
        let address = def.binding.address;
        if address < block.start {
            continue;
        }
        let index = (address - block.start) as usize;
        let Some(&value) = values.get(index) else {
            continue;
        };
        let Some(live) = g.tags.get_mut(&def.id) else {
            continue;
        };
        live.raw = if value { 1 } else { 0 };
        live.value = if value { 1.0 } else { 0.0 };
        live.bool_value = value;
        live.quality = Quality::Good;
        live.ts = now;
    }
}

fn evaluate_alarms(g: &mut EngineInner, defs: &[AlarmDefinition]) -> bool {
    let before = persisted_alarm_states(&g.alarms);
    let now = Utc::now();
    for def in defs {
        // Resolve the source quality first: an alarm whose input is not `Good`
        // must be reported as suspended, never as a fresh evaluation.
        let source = g.tags.get(&def.tag_id).map(|tag| {
            (
                tag.quality,
                tag.bool_value,
                tag.value,
                matches!(tag.def.data_type, TagDataType::Bool),
            )
        });

        let suspension = match source {
            None => Some("Source tag is not present in this project".to_string()),
            Some((Quality::Good, _, _, _)) => None,
            Some((quality, _, _, _)) => Some(format!("Source tag quality is {quality:?}")),
        };

        if let Some(reason) = suspension {
            let Some(inst) = g.alarms.get_mut(&def.id) else {
                continue;
            };
            if !inst.evaluation_suspended {
                inst.evaluation_suspended = true;
                inst.suspended_since = Some(now);
                inst.last_change = now;
            }
            inst.suspended_reason = Some(reason);
            inst.pending_active_since = None;
            inst.pending_clear_since = None;
            continue;
        }

        let (_, bool_value, value, is_bool) = source.expect("good-quality source");
        let was_source_active = g
            .alarms
            .get(&def.id)
            .map(|alarm| alarm.source_active)
            .unwrap_or(false);
        let condition_active = if is_bool {
            bool_value == def.when_true
        } else if let Some(hi) = def.hi_limit {
            let threshold = if was_source_active {
                hi - def.deadband.max(0.0)
            } else {
                hi
            };
            value >= threshold
        } else if let Some(lo) = def.lo_limit {
            let threshold = if was_source_active {
                lo + def.deadband.max(0.0)
            } else {
                lo
            };
            value <= threshold
        } else {
            false
        };

        let Some(inst) = g.alarms.get_mut(&def.id) else {
            continue;
        };
        if inst.evaluation_suspended {
            inst.evaluation_suspended = false;
            inst.suspended_reason = None;
            inst.suspended_since = None;
            inst.last_change = now;
        }
        if condition_active == inst.source_active {
            inst.pending_active_since = None;
            inst.pending_clear_since = None;
        } else {
            let delay_ms = if condition_active {
                def.on_delay_ms
            } else {
                def.off_delay_ms
            };
            let pending_since = if condition_active {
                &mut inst.pending_active_since
            } else {
                &mut inst.pending_clear_since
            };
            let started = pending_since.get_or_insert(now);
            let elapsed_ms = (now - *started).num_milliseconds().max(0) as u64;
            if delay_ms == 0 || elapsed_ms >= delay_ms {
                inst.source_active = condition_active;
                inst.pending_active_since = None;
                inst.pending_clear_since = None;
            }
        }

        match (&inst.state, inst.source_active) {
            (AlarmState::Inactive, true) => {
                inst.state = AlarmState::ActiveUnacked;
                inst.active_since = Some(now);
                inst.last_change = now;
            }
            (AlarmState::ActiveUnacked | AlarmState::ActiveAcked, false) => {
                if !inst.latched {
                    inst.state = if matches!(inst.state, AlarmState::ActiveAcked) {
                        AlarmState::Inactive
                    } else {
                        AlarmState::ClearedUnacked
                    };
                    inst.last_change = now;
                }
            }
            (AlarmState::ClearedUnacked, true) => {
                inst.state = AlarmState::ActiveUnacked;
                inst.active_since = Some(now);
                inst.last_change = now;
            }
            _ => {}
        }
    }
    before != persisted_alarm_states(&g.alarms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{water_tank_project, BitWriteMode, SessionConfig, TagBinding, WordOrder};

    fn test_inner(tag: TagDefinition, alarm: &AlarmDefinition, value: f64) -> EngineInner {
        let bool_value = value != 0.0;
        EngineInner {
            project: None,
            tags: HashMap::from([(
                tag.id.clone(),
                LiveTag {
                    def: tag,
                    value,
                    bool_value,
                    quality: Quality::Good,
                    ts: Utc::now(),
                    raw: value as u16,
                },
            )]),
            alarms: HashMap::from([(alarm.id.clone(), new_alarm_instance(alarm))]),
            connected: true,
            device_id: Some("test-device".into()),
            last_error: None,
            poll_count: 0,
            last_poll_ms: 0,
            role: Role::Engineer,
            actor: "test".into(),
            current_user: None,
            security_level: 500,
            last_activity_ts: Utc::now(),
            auth_epoch: 0,
            project_epoch: 0,
            poll_epoch: 0,
            login_throttle: HashMap::new(),
            mode: "runtime".into(),
            poll_handle: None,
            stop_tx: None,
        }
    }

    fn bool_tag(id: &str) -> TagDefinition {
        TagDefinition {
            id: id.into(),
            name: id.into(),
            device_id: "test-device".into(),
            data_type: TagDataType::Bool,
            binding: TagBinding {
                address: 10,
                bit: Some(0),
                table: ModbusTable::Holding,
                writable: false,
                bit_write_mode: BitWriteMode::MaskWrite,
                single_writer: false,
                verify_readback: true,
                word_order: WordOrder::HighWordFirst,
                min_security_level: 0,
            },
            unit: String::new(),
            description: String::new(),
            scale: 1.0,
            offset: 0.0,
            decimals: 0,
            initial_value: None,
        }
    }

    #[test]
    fn latching_alarm_remains_active_after_source_clears() {
        let tag = bool_tag("fault");
        let alarm = AlarmDefinition {
            id: "fault-alarm".into(),
            name: "Fault".into(),
            tag_id: tag.id.clone(),
            group_id: "station".into(),
            priority: AlarmPriority::High,
            when_true: true,
            hi_limit: None,
            lo_limit: None,
            deadband: 0.0,
            on_delay_ms: 0,
            off_delay_ms: 0,
            latching: true,
            message: "Fault".into(),
        };
        let mut inner = test_inner(tag, &alarm, 1.0);
        evaluate_alarms(&mut inner, std::slice::from_ref(&alarm));
        assert_eq!(inner.alarms["fault-alarm"].state, AlarmState::ActiveUnacked);
        let live = inner.tags.get_mut("fault").expect("test tag");
        live.value = 0.0;
        live.bool_value = false;
        evaluate_alarms(&mut inner, std::slice::from_ref(&alarm));
        assert!(!inner.alarms["fault-alarm"].source_active);
        assert_eq!(inner.alarms["fault-alarm"].state, AlarmState::ActiveUnacked);
    }

    #[test]
    fn analog_alarm_uses_deadband_before_returning_to_normal() {
        let mut tag = bool_tag("level");
        tag.data_type = TagDataType::U16;
        tag.binding.bit = None;
        let alarm = AlarmDefinition {
            id: "level-high".into(),
            name: "Level High".into(),
            tag_id: tag.id.clone(),
            group_id: "station".into(),
            priority: AlarmPriority::High,
            when_true: false,
            hi_limit: Some(100.0),
            lo_limit: None,
            deadband: 10.0,
            on_delay_ms: 0,
            off_delay_ms: 0,
            latching: false,
            message: "High level".into(),
        };
        let mut inner = test_inner(tag, &alarm, 100.0);
        evaluate_alarms(&mut inner, std::slice::from_ref(&alarm));
        assert!(inner.alarms["level-high"].source_active);
        inner.tags.get_mut("level").expect("test tag").value = 95.0;
        evaluate_alarms(&mut inner, std::slice::from_ref(&alarm));
        assert!(inner.alarms["level-high"].source_active);
        inner.tags.get_mut("level").expect("test tag").value = 89.0;
        evaluate_alarms(&mut inner, std::slice::from_ref(&alarm));
        assert!(!inner.alarms["level-high"].source_active);
        assert_eq!(inner.alarms["level-high"].state, AlarmState::ClearedUnacked);
    }

    #[test]
    fn read_plan_covers_all_tables_and_non_water_tank_addresses() {
        let mut project = water_tank_project();
        let device_id = project.devices[0].id.clone();
        project.tags.clear();
        let mut holding_a = bool_tag("holding-a");
        holding_a.device_id = device_id.clone();
        holding_a.binding.address = 1000;
        let mut holding_b = bool_tag("holding-b");
        holding_b.device_id = device_id.clone();
        holding_b.binding.address = 1001;
        let mut holding_c = bool_tag("holding-c");
        holding_c.device_id = device_id.clone();
        holding_c.binding.address = 1016;
        let mut coil_a = bool_tag("coil-a");
        coil_a.device_id = device_id.clone();
        coil_a.binding.address = 5;
        coil_a.binding.bit = None;
        coil_a.binding.table = ModbusTable::Coil;
        let mut coil_b = coil_a.clone();
        coil_b.id = "coil-b".into();
        coil_b.binding.address = 6;
        project.tags = vec![holding_a, holding_b, holding_c, coil_a, coil_b];

        let plan = build_read_plan(&project, &device_id);
        assert!(plan.contains(&ReadBlock {
            table: ModbusTable::Holding,
            start: 1000,
            quantity: 2,
        }));
        assert!(plan.contains(&ReadBlock {
            table: ModbusTable::Holding,
            start: 1016,
            quantity: 1,
        }));
        assert!(plan.contains(&ReadBlock {
            table: ModbusTable::Coil,
            start: 5,
            quantity: 2,
        }));
    }

    #[test]
    fn coil_and_high_address_register_samples_become_good() {
        let mut register_tag = bool_tag("register-bit");
        register_tag.binding.address = 1000;
        let alarm = AlarmDefinition {
            id: "test-alarm".into(),
            name: "Test".into(),
            tag_id: register_tag.id.clone(),
            group_id: "test".into(),
            priority: AlarmPriority::Low,
            when_true: true,
            hi_limit: None,
            lo_limit: None,
            deadband: 0.0,
            on_delay_ms: 0,
            off_delay_ms: 0,
            latching: false,
            message: "Test".into(),
        };
        let mut inner = test_inner(register_tag, &alarm, 0.0);
        let register_definition = inner.tags["register-bit"].def.clone();
        let register_project = ScadaProject {
            schema_version: 3,
            id: "test".into(),
            name: "test".into(),
            description: String::new(),
            devices: Vec::new(),
            tags: vec![register_definition],
            forms: Vec::new(),
            alarms: Vec::new(),
            alarm_groups: Vec::new(),
            users: Vec::new(),
            session_config: SessionConfig::default(),
            design_system: None,
            component_templates: Vec::new(),
            tree: Vec::new(),
            content_hash: String::new(),
        };
        apply_register_values(
            &mut inner,
            &register_project,
            "test-device",
            &ReadBlock {
                table: ModbusTable::Holding,
                start: 1000,
                quantity: 1,
            },
            &[1],
        );
        assert_eq!(inner.tags["register-bit"].quality, Quality::Good);
        assert!(inner.tags["register-bit"].bool_value);

        let mut coil_tag = bool_tag("coil");
        coil_tag.binding.table = ModbusTable::Coil;
        coil_tag.binding.bit = None;
        coil_tag.binding.address = 7;
        inner.tags.insert(
            coil_tag.id.clone(),
            LiveTag {
                def: coil_tag.clone(),
                value: 0.0,
                bool_value: false,
                quality: Quality::Bad,
                ts: Utc::now(),
                raw: 0,
            },
        );
        let coil_project = ScadaProject {
            schema_version: 3,
            id: "test-coil".into(),
            name: "test".into(),
            description: String::new(),
            devices: Vec::new(),
            tags: vec![coil_tag],
            forms: Vec::new(),
            alarms: Vec::new(),
            alarm_groups: Vec::new(),
            users: Vec::new(),
            session_config: SessionConfig::default(),
            design_system: None,
            component_templates: Vec::new(),
            tree: Vec::new(),
            content_hash: String::new(),
        };
        apply_bit_values(
            &mut inner,
            &coil_project,
            "test-device",
            &ReadBlock {
                table: ModbusTable::Coil,
                start: 7,
                quantity: 1,
            },
            &[true],
        );
        assert_eq!(inner.tags["coil"].quality, Quality::Good);
        assert!(inner.tags["coil"].bool_value);
    }

    #[test]
    fn alarm_evaluation_is_flagged_as_suspended_when_the_source_is_not_good() {
        let tag = bool_tag("fault");
        let alarm = AlarmDefinition {
            id: "fault-alarm".into(),
            name: "Fault".into(),
            tag_id: tag.id.clone(),
            group_id: "station".into(),
            priority: AlarmPriority::High,
            when_true: true,
            hi_limit: None,
            lo_limit: None,
            deadband: 0.0,
            on_delay_ms: 0,
            off_delay_ms: 0,
            latching: false,
            message: "Fault".into(),
        };
        let mut inner = test_inner(tag, &alarm, 1.0);

        evaluate_alarms(&mut inner, std::slice::from_ref(&alarm));
        assert!(!inner.alarms["fault-alarm"].evaluation_suspended);
        assert_eq!(inner.alarms["fault-alarm"].state, AlarmState::ActiveUnacked);

        mark_all_bad(&mut inner);
        evaluate_alarms(&mut inner, std::slice::from_ref(&alarm));
        let instance = &inner.alarms["fault-alarm"];
        assert!(
            instance.evaluation_suspended,
            "a bad-quality source must not be presented as a live evaluation"
        );
        assert!(instance.suspended_reason.is_some());
        assert_eq!(
            instance.state,
            AlarmState::ActiveUnacked,
            "the last trustworthy state is retained, but explicitly marked stale"
        );

        let live = inner.tags.get_mut("fault").expect("test tag");
        live.quality = Quality::Good;
        evaluate_alarms(&mut inner, std::slice::from_ref(&alarm));
        assert!(!inner.alarms["fault-alarm"].evaluation_suspended);
    }

    // ---------------------------------------------------------------------
    // Authorization gates
    // ---------------------------------------------------------------------

    const ADMIN_PASSWORD: &str = "test-admin-password";
    const OPERATOR_PASSWORD: &str = "test-operator-password";
    const OPERATOR_PIN: &str = "111111";

    fn durable_test_audit() -> Arc<AuditLog> {
        let audit = Arc::new(AuditLog::new());
        let path = std::env::temp_dir()
            .join(format!("proscada-engine-test-{}", uuid::Uuid::new_v4()))
            .join("audit.jsonl");
        audit.attach_sink(&path).expect("durable test audit");
        audit
    }

    fn attach_new_test_user_realm(engine: &Engine) -> PathBuf {
        let path = std::env::temp_dir()
            .join(format!("proscada-user-realm-test-{}", uuid::Uuid::new_v4()))
            .join("user-realm.json");
        engine
            .attach_user_realm_store(&path)
            .expect("attach test user realm");
        path
    }

    fn engine_with_water_tank() -> Engine {
        let engine = Engine::new(durable_test_audit());
        engine
            .load_builtin(water_tank_project())
            .expect("builtin project loads");
        attach_new_test_user_realm(&engine);
        engine
            .bootstrap_admin(ADMIN_PASSWORD)
            .expect("bootstrap admin");
        engine
            .login("admin", Some(ADMIN_PASSWORD))
            .expect("bootstrap admin login");
        engine
            .save_user(UserAccountInput {
                id: None,
                username: "operator".into(),
                display_name: "Operator".into(),
                password: Some(OPERATOR_PASSWORD.into()),
                pin: Some(OPERATOR_PIN.into()),
                security_level: 100,
                enabled: true,
            })
            .expect("seed test operator");
        engine.logout().expect("test setup logout");
        engine
    }

    fn sign_in_admin(engine: &Engine) {
        engine
            .login("admin", Some(ADMIN_PASSWORD))
            .expect("admin login");
    }

    #[test]
    fn engine_starts_without_any_privilege() {
        let engine = engine_with_water_tank();
        let snap = engine.snapshot();
        assert_eq!(snap.role, Role::Viewer);
        assert_eq!(snap.security_level, 0);
        assert!(snap.current_user.is_none());
        assert_eq!(snap.actor, "guest");
    }

    #[test]
    fn entering_designer_requires_an_engineering_role() {
        let engine = engine_with_water_tank();
        assert!(engine.set_mode("designer".into()).is_err());
        assert!(engine.set_mode("runtime".into()).is_ok());
        assert!(engine.set_mode("sabotage".into()).is_err());

        sign_in_admin(&engine);
        assert!(engine.set_mode("designer".into()).is_ok());
    }

    #[test]
    fn designer_mode_does_not_bypass_user_administration() {
        let engine = engine_with_water_tank();
        engine
            .login("operator", Some(OPERATOR_PASSWORD))
            .expect("operator login");
        // The operator cannot even enter designer any more, but force the mode
        // to prove the guard no longer depends on it.
        engine.inner.write().mode = "designer".into();

        let attempt = engine.save_user(UserAccountInput {
            id: None,
            username: "mallory".into(),
            display_name: "Mallory".into(),
            password: Some("a-very-long-password".into()),
            pin: None,
            security_level: 1000,
            enabled: true,
        });
        assert!(
            attempt.is_err(),
            "designer mode must not grant admin rights"
        );
        assert!(engine.delete_user("usr_admin").is_err());
    }

    #[test]
    fn saving_a_project_never_rewrites_the_user_database() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        engine
            .change_password(ADMIN_PASSWORD, "a-much-longer-password")
            .expect("password change");

        let mut project = engine.get_project().expect("project");
        project.users = vec![UserAccount {
            id: "usr_mallory".into(),
            username: "mallory".into(),
            display_name: "Mallory".into(),
            password_hash: credentials::hash_secret("irrelevant").expect("hash"),
            salt: LEGACY_SALT.into(),
            pin_hash: None,
            security_level: 1000,
            enabled: true,
            password_change_required: false,
        }];
        engine.set_project_mut(project).expect("save project");

        let users = engine.list_users().expect("users");
        assert!(
            users.iter().all(|u| u.username != "mallory"),
            "an injected account must not survive a project save"
        );
        assert!(users.iter().any(|u| u.username == "admin"));
    }

    #[test]
    fn loading_a_project_drops_the_current_session() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        assert_eq!(engine.snapshot().security_level, 1000);

        engine
            .load_project(water_tank_project())
            .expect("load project");
        let snap = engine.snapshot();
        assert_eq!(snap.security_level, 0);
        assert_eq!(snap.role, Role::Viewer);
        assert!(snap.current_user.is_none());
    }

    #[test]
    fn engineer_import_cannot_replace_the_authentication_realm() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        engine
            .save_user(UserAccountInput {
                id: None,
                username: "engineer".into(),
                display_name: "Engineer".into(),
                password: Some("test-engineer-password".into()),
                pin: None,
                security_level: 500,
                enabled: true,
            })
            .expect("create engineer");
        engine.logout().expect("logout");
        engine
            .login("engineer", Some("test-engineer-password"))
            .expect("engineer login");

        let mut crafted = water_tank_project();
        crafted.users = vec![UserAccount {
            id: "attacker".into(),
            username: "mallory".into(),
            display_name: "Mallory".into(),
            password_hash: credentials::hash_secret("mallory-admin-password").expect("hash"),
            salt: LEGACY_SALT.into(),
            pin_hash: None,
            security_level: 1000,
            enabled: true,
            password_change_required: false,
        }];
        crafted.recompute_hash();
        engine
            .load_project(crafted)
            .expect("realm-preserving import");

        assert!(engine
            .login("mallory", Some("mallory-admin-password"))
            .is_err());
        assert!(engine.login("admin", Some(ADMIN_PASSWORD)).is_ok());
    }

    #[test]
    fn a_tampered_project_hash_is_rejected() {
        let engine = engine_with_water_tank();
        let mut project = water_tank_project();
        project.recompute_hash();
        project.name = "Tampered".into();
        assert!(
            engine.load_project(project).is_err(),
            "content hash must be checked before the project is normalized"
        );
    }

    #[test]
    fn editing_the_project_requires_an_engineering_role() {
        let engine = engine_with_water_tank();
        let project = engine.get_project().expect("project");
        assert!(engine.set_project_mut(project).is_err());
    }

    #[test]
    fn a_forced_change_account_cannot_write_before_changing_its_password() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        {
            let mut g = engine.inner.write();
            g.current_user
                .as_mut()
                .expect("signed in")
                .password_change_required = true;
            g.project
                .as_mut()
                .expect("project")
                .users
                .iter_mut()
                .find(|user| user.username == "admin")
                .expect("admin")
                .password_change_required = true;
        }
        engine.set_mode("runtime".into()).expect("runtime");
        assert!(engine.snapshot().password_change_required);

        let error = engine
            .rt
            .block_on(engine.write_tag("wt.sp_p1_on", 500.0))
            .expect_err("write must be refused");
        assert!(
            error.contains("default password"),
            "unexpected error: {error}"
        );

        engine
            .change_password(ADMIN_PASSWORD, "a-much-longer-password")
            .expect("password change");
        assert!(!engine.snapshot().password_change_required);
    }

    #[test]
    fn change_password_enforces_length_and_the_current_secret() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        assert!(engine.change_password(ADMIN_PASSWORD, "short").is_err());
        assert!(engine
            .change_password("wrong", "a-much-longer-password")
            .is_err());
        assert!(engine
            .change_password(ADMIN_PASSWORD, "a-much-longer-password")
            .is_ok());
        assert!(engine.login("admin", Some(ADMIN_PASSWORD)).is_err());
        assert!(engine
            .login("admin", Some("a-much-longer-password"))
            .is_ok());
    }

    #[test]
    fn legacy_sha256_credentials_are_upgraded_on_first_login() {
        let engine = engine_with_water_tank();
        {
            let mut g = engine.inner.write();
            let project = g.project.as_mut().expect("project");
            let user = project
                .users
                .iter_mut()
                .find(|u| u.username == "admin")
                .expect("admin");
            user.password_hash = crate::project::hash_password(ADMIN_PASSWORD, LEGACY_SALT);
            user.salt = LEGACY_SALT.into();
        }
        engine.login("admin", Some(ADMIN_PASSWORD)).expect("login");
        let stored = engine
            .get_project()
            .expect("project")
            .users
            .iter()
            .find(|u| u.username == "admin")
            .expect("admin")
            .password_hash
            .clone();
        assert!(
            stored.starts_with("$argon2"),
            "a legacy digest must be replaced on successful login"
        );
    }

    #[test]
    fn pin_never_establishes_a_login_session() {
        let engine = engine_with_water_tank();
        assert!(engine.login(OPERATOR_PIN, None).is_err());
        assert!(engine.snapshot().current_user.is_none());
    }

    #[test]
    fn writes_are_refused_outside_runtime_and_below_the_required_level() {
        let engine = engine_with_water_tank();

        // Viewer, designer mode.
        let error = engine
            .rt
            .block_on(engine.write_tag("wt.sp_p1_on", 500.0))
            .expect_err("viewer write");
        assert!(error.contains("Role cannot write"), "unexpected: {error}");

        engine
            .login("operator", Some(OPERATOR_PASSWORD))
            .expect("operator login");
        engine.inner.write().mode = "designer".into();
        let error = engine
            .rt
            .block_on(engine.write_tag("wt.sp_p1_on", 500.0))
            .expect_err("designer write");
        assert!(error.contains("Runtime mode"), "unexpected: {error}");
    }

    #[test]
    fn writes_are_refused_for_read_only_tags_and_bad_quality() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        engine
            .change_password(ADMIN_PASSWORD, "a-much-longer-password")
            .expect("password change");
        engine.set_mode("runtime".into()).expect("runtime");

        let error = engine
            .rt
            .block_on(engine.write_tag("wt.p1_run", 1.0))
            .expect_err("read-only tag");
        assert!(error.contains("not writable"), "unexpected: {error}");

        // Writable tag, but nothing has been polled yet, so quality is Bad.
        let error = engine
            .rt
            .block_on(engine.write_tag("wt.sp_p1_on", 500.0))
            .expect_err("bad quality");
        assert!(
            error.contains("quality must be Good"),
            "unexpected: {error}"
        );
    }

    #[test]
    fn a_tag_can_demand_a_higher_security_level_than_the_role_alone() {
        let engine = engine_with_water_tank();
        {
            let mut g = engine.inner.write();
            let project = g.project.as_mut().expect("project");
            let tag = project
                .tags
                .iter_mut()
                .find(|t| t.id == "wt.sp_p1_on")
                .expect("setpoint tag");
            tag.binding.min_security_level = 500;
        }
        engine
            .login("operator", Some(OPERATOR_PASSWORD))
            .expect("operator login");
        engine.inner.write().mode = "runtime".into();
        engine
            .inner
            .write()
            .current_user
            .as_mut()
            .expect("user")
            .password_change_required = false;

        let error = engine
            .rt
            .block_on(engine.write_tag("wt.sp_p1_on", 500.0))
            .expect_err("level gate");
        assert!(error.contains("Security level 500"), "unexpected: {error}");
    }

    #[test]
    fn user_administration_requires_administrator_level() {
        let engine = engine_with_water_tank();
        engine
            .login("operator", Some(OPERATOR_PASSWORD))
            .expect("operator login");
        assert!(engine
            .save_user(UserAccountInput {
                id: None,
                username: "mallory".into(),
                display_name: "Mallory".into(),
                password: Some("a-very-long-password".into()),
                pin: None,
                security_level: 1000,
                enabled: true,
            })
            .is_err());
    }

    #[test]
    fn new_accounts_require_a_password_and_reject_weak_secrets() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        engine
            .change_password(ADMIN_PASSWORD, "a-much-longer-password")
            .expect("password change");

        let missing_password = engine.save_user(UserAccountInput {
            id: None,
            username: "no-secret".into(),
            display_name: "No Secret".into(),
            password: None,
            pin: None,
            security_level: 100,
            enabled: true,
        });
        assert!(missing_password.is_err());

        let weak = engine.save_user(UserAccountInput {
            id: None,
            username: "weak".into(),
            display_name: "Weak".into(),
            password: Some("short".into()),
            pin: None,
            security_level: 100,
            enabled: true,
        });
        assert!(weak.is_err());
    }

    #[test]
    fn project_validation_rejects_content_the_engine_cannot_represent() {
        let mut project = water_tank_project();
        project.tags[0].data_type = TagDataType::String;
        assert!(project.validate().is_err(), "String has no Modbus width");

        let mut project = water_tank_project();
        project.tags[0].binding.bit = Some(16);
        project.tags[0].data_type = TagDataType::Bool;
        assert!(project.validate().is_err(), "bit 16 does not exist");

        let mut project = water_tank_project();
        project.tags[0].binding.table = ModbusTable::Input;
        project.tags[0].binding.writable = true;
        assert!(project.validate().is_err(), "input registers are read-only");

        let mut project = water_tank_project();
        project.tags[0].scale = 0.0;
        assert!(project.validate().is_err(), "zero scale is not invertible");
    }

    #[test]
    fn an_idle_runtime_session_expires_but_activity_keeps_it_alive() {
        let engine = engine_with_water_tank();
        engine
            .login("operator", Some(OPERATOR_PASSWORD))
            .expect("operator login");
        engine.inner.write().mode = "runtime".into();
        assert_eq!(engine.snapshot().security_level, 100);

        // Reading the snapshot must never expire the session by itself.
        engine.inner.write().last_activity_ts = Utc::now() - chrono::Duration::minutes(60);
        assert_eq!(
            engine.snapshot().security_level,
            100,
            "snapshot must not be a state-changing operation"
        );

        engine.expire_idle_session();
        let snap = engine.snapshot();
        assert_eq!(snap.security_level, 0);
        assert_eq!(snap.role, Role::Viewer);
    }

    #[test]
    fn expired_session_cannot_acknowledge_an_alarm() {
        let engine = engine_with_water_tank();
        engine
            .login("operator", Some(OPERATOR_PASSWORD))
            .expect("operator login");
        engine.inner.write().last_activity_ts = Utc::now() - chrono::Duration::minutes(60);
        let alarm_id = engine
            .snapshot()
            .alarms
            .first()
            .map(|a| a.def_id.clone())
            .expect("water tank defines alarms");

        assert!(engine.ack_alarm(&alarm_id).is_err());
        assert_eq!(engine.snapshot().security_level, 0);
        assert_eq!(engine.snapshot().mode, "runtime");
    }

    #[test]
    fn multi_register_tags_are_read_in_full() {
        let mut project = water_tank_project();
        let device_id = project.devices[0].id.clone();
        let mut wide = project.tags[0].clone();
        wide.id = "wide".into();
        wide.data_type = TagDataType::F32;
        wide.binding.bit = None;
        wide.binding.address = 2000;
        project.tags = vec![wide];

        let plan = build_read_plan(&project, &device_id);
        assert!(
            plan.contains(&ReadBlock {
                table: ModbusTable::Holding,
                start: 2000,
                quantity: 2,
            }),
            "a 32-bit tag must claim both of its registers: {plan:?}"
        );
    }

    #[test]
    fn memory_tags_can_be_written_in_runtime() {
        let mut project = water_tank_project();
        let mem_tag = TagDefinition {
            id: "mem.sp_calc".into(),
            name: "Internal Calculated SP".into(),
            device_id: "SYS_INTERNAL".into(),
            data_type: TagDataType::F32,
            unit: "bar".into(),
            description: "Internal Memory Tag".into(),
            scale: 1.0,
            offset: 0.0,
            decimals: 2,
            binding: TagBinding {
                table: ModbusTable::Memory,
                address: 0,
                bit: None,
                writable: true,
                bit_write_mode: BitWriteMode::MaskWrite,
                single_writer: false,
                verify_readback: true,
                word_order: WordOrder::HighWordFirst,
                min_security_level: 0,
            },
            initial_value: Some("4.5".into()),
        };
        project.tags.push(mem_tag);
        assert!(
            project.validate().is_ok(),
            "project validation should accept SYS_INTERNAL memory tag"
        );

        let audit = durable_test_audit();
        let engine = Engine::new(audit);
        engine
            .install_project(project, false)
            .expect("install project");
        attach_new_test_user_realm(&engine);

        let snap = engine.snapshot();
        let live = snap
            .tags
            .iter()
            .find(|t| t.tag_id == "mem.sp_calc")
            .expect("live tag");
        assert_eq!(live.quality, Quality::Good);
        assert_eq!(live.value, 4.5);

        engine
            .bootstrap_admin(ADMIN_PASSWORD)
            .expect("bootstrap memory-project admin");
        engine.login("admin", Some(ADMIN_PASSWORD)).expect("login");
        engine.inner.write().mode = "runtime".into();

        let receipt = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(engine.write_tag("mem.sp_calc", 12.8))
            .expect("write memory tag");
        assert!(receipt.matches);
        assert_eq!(receipt.protocol, "memory");
        assert!((receipt.observed_value - 12.8).abs() < 0.001);

        let snap_after = engine.snapshot();
        let live_after = snap_after
            .tags
            .iter()
            .find(|t| t.tag_id == "mem.sp_calc")
            .expect("live tag after write");
        assert!((live_after.value - 12.8).abs() < 0.001);
        assert_eq!(live_after.quality, Quality::Good);
    }

    #[test]
    fn mark_all_bad_preserves_internal_memory_and_system_tags() {
        let mut project = water_tank_project();
        let mem_tag = TagDefinition {
            id: "mem.status".into(),
            name: "Memory Status".into(),
            device_id: "SYS_INTERNAL".into(),
            data_type: TagDataType::Bool,
            unit: String::new(),
            description: "Internal Status".into(),
            scale: 1.0,
            offset: 0.0,
            decimals: 0,
            binding: TagBinding {
                table: ModbusTable::Memory,
                address: 0,
                bit: None,
                writable: true,
                bit_write_mode: BitWriteMode::MaskWrite,
                single_writer: false,
                verify_readback: true,
                word_order: WordOrder::HighWordFirst,
                min_security_level: 0,
            },
            initial_value: Some("true".into()),
        };
        project.tags.push(mem_tag);

        let audit = Arc::new(crate::audit::AuditLog::new());
        let engine = Engine::new(audit);
        engine.install_project(project, false).expect("install");

        {
            let mut g = engine.inner.write();
            mark_all_bad(&mut g);
        }

        let snap = engine.snapshot();
        let live_modbus = snap
            .tags
            .iter()
            .find(|t| t.tag_id == "wt.level_cm")
            .expect("plc tag");
        assert_eq!(live_modbus.quality, Quality::Bad);

        let live_mem = snap
            .tags
            .iter()
            .find(|t| t.tag_id == "mem.status")
            .expect("mem tag");
        assert_eq!(
            live_mem.quality,
            Quality::Good,
            "memory tags must not be degraded when modbus disconnects"
        );
    }

    #[test]
    fn memory_tags_do_not_age_out_to_uncertain_in_snapshot() {
        let mut project = water_tank_project();
        let mem_tag = TagDefinition {
            id: "mem.stale_check".into(),
            name: "Stale Check Tag".into(),
            device_id: "SYS_INTERNAL".into(),
            data_type: TagDataType::F32,
            unit: String::new(),
            description: "Age test".into(),
            scale: 1.0,
            offset: 0.0,
            decimals: 2,
            binding: TagBinding {
                table: ModbusTable::Memory,
                address: 0,
                bit: None,
                writable: true,
                bit_write_mode: BitWriteMode::MaskWrite,
                single_writer: false,
                verify_readback: true,
                word_order: WordOrder::HighWordFirst,
                min_security_level: 0,
            },
            initial_value: Some("100.0".into()),
        };
        project.tags.push(mem_tag);

        let audit = Arc::new(crate::audit::AuditLog::new());
        let engine = Engine::new(audit);
        engine.install_project(project, false).expect("install");

        // Manually set timestamp to 10 seconds ago
        {
            let mut g = engine.inner.write();
            if let Some(t) = g.tags.get_mut("mem.stale_check") {
                t.ts = Utc::now() - chrono::Duration::seconds(10);
            }
        }

        let snap = engine.snapshot();
        let live_mem = snap
            .tags
            .iter()
            .find(|t| t.tag_id == "mem.stale_check")
            .expect("memory tag");

        assert_eq!(
            live_mem.quality,
            Quality::Good,
            "internal memory tags must maintain Quality::Good even after 3 seconds without poll"
        );
    }

    #[test]
    fn external_project_without_a_hash_is_rejected() {
        let engine = engine_with_water_tank();
        let mut project = water_tank_project();
        project.content_hash.clear();
        assert!(engine
            .load_project(project)
            .expect_err("empty external hash")
            .contains("missing its content hash"));
    }

    #[test]
    fn bootstrap_is_one_shot_and_never_provisions_a_factory_pin() {
        let audit = durable_test_audit();
        let engine = Engine::new(audit);
        engine
            .load_builtin(water_tank_project())
            .expect("load builtin");
        let realm_path = attach_new_test_user_realm(&engine);
        assert!(engine.snapshot().requires_bootstrap);
        assert!(engine.bootstrap_admin("short").is_err());
        let admin = engine
            .bootstrap_admin(ADMIN_PASSWORD)
            .expect("first bootstrap");
        assert!(!admin.has_pin);
        assert!(!engine.snapshot().requires_bootstrap);
        assert!(engine.bootstrap_admin("another-long-password").is_err());
        #[cfg(unix)]
        assert_eq!(
            std::fs::metadata(&realm_path)
                .expect("realm metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    #[test]
    fn bootstrap_and_mutations_fail_closed_without_durable_audit() {
        let engine = Engine::new(Arc::new(AuditLog::new()));
        engine
            .load_builtin(water_tank_project())
            .expect("load builtin");
        attach_new_test_user_realm(&engine);
        let error = engine
            .bootstrap_admin(ADMIN_PASSWORD)
            .expect_err("no durable audit");
        assert!(error.contains("audit sink"), "unexpected: {error}");
        assert!(engine.snapshot().requires_bootstrap);
        assert!(!engine.snapshot().audit_persisted);
    }

    #[test]
    fn user_realm_survives_restart_is_project_independent_and_corruption_never_reopens_bootstrap() {
        let dir =
            std::env::temp_dir().join(format!("proscada-user-realm-{}", uuid::Uuid::new_v4()));
        let path = dir.join("user-realm.json");

        let first = Engine::new(durable_test_audit());
        first
            .load_builtin(water_tank_project())
            .expect("load first project");
        first
            .attach_user_realm_store(&path)
            .expect("initialize realm");
        first
            .bootstrap_admin(ADMIN_PASSWORD)
            .expect("bootstrap first admin");
        first
            .login("admin", Some(ADMIN_PASSWORD))
            .expect("login first admin");
        first
            .save_user(UserAccountInput {
                id: None,
                username: "operator".into(),
                display_name: "Operator".into(),
                password: Some(OPERATOR_PASSWORD.into()),
                pin: Some(OPERATOR_PIN.into()),
                security_level: 100,
                enabled: true,
            })
            .expect("persist second account");

        let second = Engine::new(durable_test_audit());
        let mut renamed_project = water_tank_project();
        renamed_project.id = "different-project-id".into();
        second
            .load_builtin(renamed_project)
            .expect("load different project");
        second
            .attach_user_realm_store(&path)
            .expect("restore installation realm");
        let snapshot = second.snapshot();
        assert!(snapshot.user_realm_persisted);
        assert!(!snapshot.requires_bootstrap);
        second
            .login("admin", Some(ADMIN_PASSWORD))
            .expect("restored admin login");
        assert_eq!(second.list_users().expect("restored users").len(), 2);

        let corrupt_path = dir.join("corrupt-user-realm.json");
        std::fs::write(&corrupt_path, b"{ definitely not json").expect("write corrupt realm");
        let corrupt = Engine::new(durable_test_audit());
        corrupt
            .load_builtin(water_tank_project())
            .expect("load project for corrupt realm");
        assert!(corrupt.attach_user_realm_store(&corrupt_path).is_err());
        let corrupt_snapshot = corrupt.snapshot();
        assert!(!corrupt_snapshot.user_realm_persisted);
        assert!(corrupt_snapshot.user_realm_last_error.is_some());
        assert!(
            !corrupt_snapshot.requires_bootstrap,
            "a corrupt existing realm must never reopen initial provisioning"
        );
        assert!(corrupt.bootstrap_admin(ADMIN_PASSWORD).is_err());

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn account_mutation_is_not_committed_in_memory_when_realm_persistence_fails() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        let before = engine.list_users().expect("users before failed save");
        let realm_path = engine
            .user_realm_store
            .inner
            .lock()
            .path
            .clone()
            .expect("test realm path");
        std::fs::remove_file(&realm_path).expect("remove realm to inject failure");
        std::fs::create_dir(&realm_path).expect("replace realm file with directory");

        let error = engine
            .save_user(UserAccountInput {
                id: None,
                username: "must-not-commit".into(),
                display_name: "Must Not Commit".into(),
                password: Some("another-long-password".into()),
                pin: None,
                security_level: 100,
                enabled: true,
            })
            .expect_err("realm persistence failure must reject mutation");
        assert!(
            error.contains("Atomically replace") || error.contains("protected state"),
            "unexpected error: {error}"
        );
        assert_eq!(
            engine.list_users().expect("users after failed save").len(),
            before.len()
        );
        assert!(!engine.snapshot().user_realm_persisted);

        let parent = realm_path.parent().expect("realm parent").to_path_buf();
        let _ = std::fs::remove_dir_all(parent);
    }

    #[test]
    fn stale_quality_mutates_authoritative_state_suspends_alarm_and_blocks_write() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        {
            let mut g = engine.inner.write();
            let live = g.tags.get_mut("wt.level_cm").expect("level tag");
            live.quality = Quality::Good;
            live.value = 900.0;
            live.ts = Utc::now() - chrono::Duration::seconds(10);
            let alarms = g.project.as_ref().expect("project").alarms.clone();
            evaluate_alarms(&mut g, &alarms);
        }

        let snapshot = engine.snapshot();
        assert_eq!(
            snapshot
                .tags
                .iter()
                .find(|tag| tag.tag_id == "wt.level_cm")
                .expect("level")
                .quality,
            Quality::Uncertain
        );
        assert!(engine.inner.read().tags["wt.level_cm"].quality == Quality::Uncertain);
        assert!(
            snapshot
                .alarms
                .iter()
                .find(|alarm| alarm.def_id == "alm_level_hi")
                .expect("alarm")
                .evaluation_suspended
        );
        let error = engine
            .rt
            .block_on(engine.write_tag("wt.level_cm", 500.0))
            .expect_err("stale write");
        assert!(error.contains("Uncertain"), "unexpected: {error}");
        assert!(engine
            .audit()
            .list(20)
            .iter()
            .any(|entry| entry.action == "tag.write_denied"));
    }

    #[test]
    fn stopping_polling_immediately_marks_plc_tags_bad_and_suspends_alarms() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        {
            let mut g = engine.inner.write();
            let live = g.tags.get_mut("wt.level_cm").expect("level");
            live.quality = Quality::Good;
            live.ts = Utc::now();
            let alarms = g.project.as_ref().expect("project").alarms.clone();
            evaluate_alarms(&mut g, &alarms);
        }
        engine.stop_polling().expect("authorized stop");
        let snapshot = engine.snapshot();
        assert_eq!(
            snapshot
                .tags
                .iter()
                .find(|tag| tag.tag_id == "wt.level_cm")
                .expect("level")
                .quality,
            Quality::Bad
        );
        assert!(snapshot
            .alarms
            .iter()
            .all(|alarm| alarm.evaluation_suspended));
    }

    #[test]
    fn explicit_missing_device_never_falls_back_to_another_device() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        let error = engine
            .start_polling(Some("missing-device".into()))
            .expect_err("missing device");
        assert_eq!(error, "Device not found: missing-device");
        assert!(!engine.snapshot().connected);
    }

    #[test]
    fn pin_challenge_is_bound_to_the_same_memory_write_request() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        let admin = engine.snapshot().current_user.expect("admin summary");
        engine
            .save_user(UserAccountInput {
                id: Some(admin.id),
                username: "admin".into(),
                display_name: "Administrator".into(),
                password: None,
                pin: Some("654321".into()),
                security_level: 1000,
                enabled: true,
            })
            .expect("set admin pin");

        let mut project = engine.get_project().expect("project");
        project.session_config.pin_challenge_on_write = true;
        let mut memory = project.tags[0].clone();
        memory.id = "memory.pin-gated".into();
        memory.name = "PIN gated memory".into();
        memory.device_id = "SYS_INTERNAL".into();
        memory.binding.table = ModbusTable::Memory;
        memory.binding.bit = None;
        memory.binding.writable = true;
        memory.data_type = TagDataType::U16;
        memory.initial_value = Some("1".into());
        project.tags.push(memory);
        engine.set_project_mut(project).expect("install memory tag");

        assert!(engine
            .rt
            .block_on(engine.write_tag_with_pin("memory.pin-gated", 2.0, None))
            .is_err());
        assert!(engine
            .rt
            .block_on(engine.write_tag_with_pin("memory.pin-gated", 2.0, Some("000000")))
            .is_err());
        let receipt = engine
            .rt
            .block_on(engine.write_tag_with_pin("memory.pin-gated", 2.0, Some("654321")))
            .expect("atomic pin write");
        assert!(receipt.matches);
        assert_eq!(receipt.observed_value, 2.0);
    }

    #[test]
    fn last_admin_duplicate_username_and_duplicate_pin_are_rejected() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        let admin = engine.snapshot().current_user.expect("admin");
        assert!(engine
            .save_user(UserAccountInput {
                id: Some(admin.id.clone()),
                username: "admin".into(),
                display_name: "Administrator".into(),
                password: None,
                pin: None,
                security_level: 500,
                enabled: true,
            })
            .expect_err("last admin demotion")
            .contains("Administrator"));
        assert!(engine
            .save_user(UserAccountInput {
                id: None,
                username: "operator".into(),
                display_name: "Duplicate".into(),
                password: Some("duplicate-user-password".into()),
                pin: None,
                security_level: 100,
                enabled: true,
            })
            .expect_err("duplicate username")
            .contains("already in use"));
        assert!(engine
            .save_user(UserAccountInput {
                id: Some(admin.id),
                username: "admin".into(),
                display_name: "Administrator".into(),
                password: None,
                pin: Some(OPERATOR_PIN.into()),
                security_level: 1000,
                enabled: true,
            })
            .expect_err("duplicate pin")
            .contains("already assigned"));
    }

    #[test]
    fn alarm_lifecycle_survives_unrelated_designer_save() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        let active_since = Utc::now() - chrono::Duration::minutes(1);
        {
            let mut g = engine.inner.write();
            let alarm = g.alarms.get_mut("alm_level_hi").expect("alarm");
            alarm.state = AlarmState::ActiveAcked;
            alarm.source_active = true;
            alarm.active_since = Some(active_since);
        }
        let mut project = engine.get_project().expect("project");
        project.description.push_str(" edited");
        engine.set_project_mut(project).expect("designer save");
        let alarm = engine
            .snapshot()
            .alarms
            .into_iter()
            .find(|alarm| alarm.def_id == "alm_level_hi")
            .expect("alarm");
        assert_eq!(alarm.state, AlarmState::ActiveAcked);
        assert_eq!(alarm.active_since, Some(active_since));
    }

    #[test]
    fn canonical_project_file_save_is_atomic_and_keeps_one_backup() {
        let engine = engine_with_water_tank();
        let dir = std::env::temp_dir().join(format!("proscada-save-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let target = dir.join("plant.proscada.json");
        std::fs::write(&target, b"previous-good-version").expect("old project");
        assert!(engine
            .save_project_file(target.to_str().expect("path"))
            .is_err());

        sign_in_admin(&engine);
        engine
            .save_project_file(target.to_str().expect("path"))
            .expect("atomic save");
        let saved: ScadaProject =
            serde_json::from_slice(&std::fs::read(&target).expect("saved project"))
                .expect("canonical JSON");
        assert!(
            !saved.users.is_empty(),
            "canonical backend save retains the protected user database"
        );
        let backup = PathBuf::from(format!("{}.bak", target.display()));
        assert_eq!(
            std::fs::read(backup).expect("backup"),
            b"previous-good-version"
        );
        assert!(engine
            .save_project_file(dir.join("plant.txt").to_str().expect("path"))
            .is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn system_tags_are_allowlisted_and_reflect_authoritative_engine_state() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        let mut project = engine.get_project().expect("project");
        for (id, data_type) in [
            ("system.connected", TagDataType::Bool),
            ("system.security_level", TagDataType::U16),
            ("system.mode", TagDataType::Bool),
        ] {
            let mut tag = project.tags[0].clone();
            tag.id = id.into();
            tag.name = id.into();
            tag.device_id = "SYS_INTERNAL".into();
            tag.binding.table = ModbusTable::System;
            tag.binding.writable = false;
            tag.binding.bit = None;
            tag.data_type = data_type;
            tag.initial_value = None;
            tag.scale = 1.0;
            tag.offset = 0.0;
            project.tags.push(tag);
        }
        engine.set_project_mut(project).expect("system tags");
        let snapshot = engine.snapshot();
        assert_eq!(
            snapshot
                .tags
                .iter()
                .find(|tag| tag.tag_id == "system.connected")
                .expect("connected")
                .value,
            0.0
        );
        assert_eq!(
            snapshot
                .tags
                .iter()
                .find(|tag| tag.tag_id == "system.security_level")
                .expect("security")
                .value,
            1000.0
        );
        assert_eq!(
            snapshot
                .tags
                .iter()
                .find(|tag| tag.tag_id == "system.mode")
                .expect("mode")
                .value,
            1.0
        );
    }

    #[test]
    fn repeated_login_failures_apply_bounded_backoff() {
        let engine = engine_with_water_tank();
        for _ in 0..5 {
            assert!(engine.login("admin", Some("definitely-wrong")).is_err());
        }
        let error = engine
            .login("admin", Some(ADMIN_PASSWORD))
            .expect_err("correct credential remains throttled");
        assert!(error.contains("Too many failed"), "unexpected: {error}");
    }

    #[test]
    fn alarm_ack_lifecycle_restores_after_restart_and_rejects_bad_journals() {
        let dir =
            std::env::temp_dir().join(format!("proscada-alarm-state-{}", uuid::Uuid::new_v4()));
        let path = dir.join("alarm-state.json");

        let first = engine_with_water_tank();
        first
            .attach_alarm_state_store(&path)
            .expect("attach first journal");
        sign_in_admin(&first);
        let active_since = Utc::now() - chrono::Duration::minutes(2);
        {
            let mut g = first.inner.write();
            let alarm = g.alarms.get_mut("alm_fault").expect("latching alarm");
            alarm.state = AlarmState::ActiveUnacked;
            alarm.source_active = true;
            alarm.active_since = Some(active_since);
            alarm.evaluation_suspended = false;
            first
                .alarm_state_store
                .persist(&g)
                .expect("persist active alarm");
        }
        first.ack_alarm("alm_fault").expect("durable ACK");
        assert_eq!(
            first.inner.read().alarms["alm_fault"].state,
            AlarmState::ActiveAcked
        );

        let second = Engine::new(durable_test_audit());
        second
            .load_builtin(water_tank_project())
            .expect("load same project");
        second
            .attach_alarm_state_store(&path)
            .expect("restore journal");
        let restored = second.inner.read().alarms["alm_fault"].clone();
        assert_eq!(restored.state, AlarmState::ActiveAcked);
        assert_eq!(restored.active_since, Some(active_since));
        assert!(restored.evaluation_suspended);

        let wrong_project = Engine::new(durable_test_audit());
        let mut other = water_tank_project();
        other.id = "another-project".into();
        wrong_project
            .install_project(other, false)
            .expect("install other project");
        assert!(wrong_project.attach_alarm_state_store(&path).is_err());
        assert_eq!(
            wrong_project.inner.read().alarms["alm_fault"].state,
            AlarmState::Inactive
        );

        let corrupt_path = dir.join("corrupt-alarm-state.json");
        std::fs::write(&corrupt_path, b"{ definitely not json").expect("corrupt journal");
        let corrupt = Engine::new(durable_test_audit());
        corrupt
            .load_builtin(water_tank_project())
            .expect("load project");
        assert!(corrupt.attach_alarm_state_store(&corrupt_path).is_err());
        let snapshot = corrupt.snapshot();
        assert!(!snapshot.alarm_state_persisted);
        assert!(snapshot.alarm_state_last_error.is_some());

        let _ = std::fs::remove_dir_all(dir);
    }
}
