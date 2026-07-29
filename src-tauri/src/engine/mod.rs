//! Real-time tag engine, polling scheduler, alarm evaluation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use tokio::sync::{watch, Mutex as AsyncMutex};
use tokio::task::JoinHandle;

use crate::audit::AuditLog;
use crate::modbus::{self, codec, ConnectionConfig};
use crate::project::{
    credentials, AlarmDefinition, AlarmPriority, BitWriteMode, DeviceConfig, ModbusTable, Role,
    ScadaProject, TagDataType, TagDefinition, UserAccount, UserSummary, LEGACY_SALT,
};

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
}

struct LiveTag {
    def: TagDefinition,
    value: f64,
    bool_value: bool,
    quality: Quality,
    ts: DateTime<Utc>,
    raw: u16,
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
    mode: String, // designer | runtime
    poll_handle: Option<JoinHandle<()>>,
    stop_tx: Option<watch::Sender<bool>>,
}

pub struct Engine {
    inner: Arc<RwLock<EngineInner>>,
    audit: Arc<AuditLog>,
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
                mode: "designer".into(),
                poll_handle: None,
                stop_tx: None,
            })),
            audit,
            write_locks: Mutex::new(HashMap::new()),
            write_sessions: Mutex::new(HashMap::new()),
            rt,
        }
    }

    pub fn audit(&self) -> Arc<AuditLog> {
        self.audit.clone()
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
        let mut proj = project;
        // Verify *before* mutating, otherwise the check compares the content
        // against a hash we just recomputed and can never fail.
        if verify_integrity && !proj.verify_hash() {
            return Err("Project content hash verification failed".into());
        }
        proj.validate()?;
        proj.ensure_default_users();
        proj.recompute_hash();

        self.stop_polling();
        self.close_write_sessions();
        let mut g = self.inner.write();
        let mut tags = HashMap::new();
        for def in &proj.tags {
            let is_internal = def.binding.table == ModbusTable::Memory
                || def.binding.table == ModbusTable::System;
            let initial_value = if is_internal {
                def.initial_value
                    .as_deref()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0)
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
            alarms.insert(def.id.clone(), new_alarm_instance(def));
        }
        let name = proj.name.clone();
        g.project = Some(proj);
        g.tags = tags;
        g.alarms = alarms;
        g.connected = false;
        g.device_id = None;
        g.last_error = None;
        g.poll_count = 0;
        let actor = g.actor.clone();
        let role = g.role.clone();
        drop(g);
        self.audit
            .append(&actor, role_str(&role), "project.load", &name);
        Ok(())
    }

    /// Load a project file. Loading always drops the current session, because
    /// the project *is* the user database: adopting a new one must never carry
    /// privileges granted by the previous one.
    pub fn load_project(&self, project: ScadaProject) -> Result<(), String> {
        self.install_project(project, true)?;
        self.reset_session("project.load");
        Ok(())
    }

    /// Load the built-in template during start-up, before any user exists.
    pub fn load_builtin(&self, project: ScadaProject) -> Result<(), String> {
        self.install_project(project, false)?;
        self.reset_session("project.load_builtin");
        Ok(())
    }

    fn reset_session(&self, reason: &str) {
        let mut g = self.inner.write();
        if g.current_user.is_none() && g.security_level == 0 {
            return;
        }
        let prev = g.actor.clone();
        g.current_user = None;
        g.security_level = 0;
        g.role = Role::Viewer;
        g.actor = "guest".into();
        g.last_activity_ts = Utc::now();
        drop(g);
        self.audit
            .append(&prev, "viewer", "auth.session_reset", reason);
    }

    pub fn get_project(&self) -> Option<ScadaProject> {
        self.inner.read().project.clone()
    }

    /// Persist designer edits.
    ///
    /// The user database is deliberately *not* taken from the incoming project:
    /// accounts change only through `save_user`/`delete_user`, which require
    /// administrator level. Otherwise an Engineer could grant themselves
    /// administrator by editing the project payload.
    pub fn set_project_mut(&self, project: ScadaProject) -> Result<(), String> {
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
        p.ensure_default_users();
        p.recompute_hash();
        self.install_project(p, false)
    }

    /// Switch between Designer and Runtime.
    ///
    /// Entering Designer is an engineering action and requires the matching
    /// role; entering Runtime is always allowed so a viewer can observe.
    pub fn set_mode(&self, mode: String) -> Result<(), String> {
        if mode != "designer" && mode != "runtime" {
            return Err(format!("Unknown mode: {mode}"));
        }
        let mut g = self.inner.write();
        if mode == "designer" && !g.role.can_edit_project() {
            return Err("Engineer or Administrator role is required to enter Designer".into());
        }
        g.mode = mode.clone();
        g.last_activity_ts = Utc::now();
        let actor = g.actor.clone();
        let role = g.role.clone();
        drop(g);
        self.audit
            .append(&actor, role_str(&role), "mode.set", &mode);
        Ok(())
    }

    /// Authenticate by username+password or by PIN alone.
    ///
    /// Legacy SHA-256 records are accepted once and immediately re-hashed with
    /// Argon2id, so an old project upgrades itself on first successful login.
    pub fn login(
        &self,
        username_or_pin: &str,
        password: Option<&str>,
    ) -> Result<UserSummary, String> {
        let mut g = self.inner.write();
        let term = username_or_pin.trim();
        if term.is_empty() {
            drop(g);
            self.audit.append(
                "unknown",
                "unauthenticated",
                "auth.login_failed",
                "Empty credential",
            );
            return Err("Invalid username, password or PIN".into());
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
            if let Some(pin_hash) = &user.pin_hash {
                let outcome = credentials::verify_secret(term, pin_hash, &user.salt);
                if outcome.is_accepted() {
                    matched = Some((index, LoginKind::Pin, outcome));
                    break;
                }
            }
        }

        let Some((index, kind, outcome)) = matched else {
            drop(g);
            self.audit.append(
                term,
                "unauthenticated",
                "auth.login_failed",
                "Invalid credentials",
            );
            return Err("Invalid username, password or PIN".into());
        };

        let mut rehashed = false;
        if outcome == credentials::Verification::AcceptedNeedsRehash {
            let secret = match kind {
                LoginKind::Password => password.map(str::to_string),
                LoginKind::Pin => Some(term.to_string()),
            };
            if let Some(secret) = secret {
                if let Ok(upgraded) = credentials::hash_secret(&secret) {
                    if let Some(project) = g.project.as_mut() {
                        match kind {
                            LoginKind::Password => project.users[index].password_hash = upgraded,
                            LoginKind::Pin => project.users[index].pin_hash = Some(upgraded),
                        }
                        project.recompute_hash();
                        rehashed = true;
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
        let project = g.project.as_mut().ok_or("No project loaded")?;
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

        project.users[index].password_hash = credentials::hash_secret(new_password)?;
        project.users[index].salt = LEGACY_SALT.into();
        project.users[index].password_change_required = false;
        let summary = project.users[index].to_summary();
        project.recompute_hash();
        g.current_user = Some(summary.clone());
        let actor = g.actor.clone();
        let level = g.security_level;
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
        drop(g);
        self.audit.append(
            &prev_actor,
            &format!("L{prev_level}"),
            "auth.logout",
            "User logged out",
        );
        Ok(())
    }

    /// Re-authenticate the *signed-in* operator by PIN.
    ///
    /// Accepting any sufficiently privileged account's PIN would let one
    /// operator confirm another operator's action, so only the current user's
    /// PIN is considered.
    pub fn verify_pin(&self, pin: &str) -> Result<bool, String> {
        let mut g = self.inner.write();
        let project = g.project.as_ref().ok_or("No project loaded")?;
        let Some(current) = g.current_user.as_ref() else {
            return Ok(false);
        };
        let Some(user) = project
            .users
            .iter()
            .find(|u| u.enabled && u.id == current.id)
        else {
            return Ok(false);
        };
        let Some(pin_hash) = &user.pin_hash else {
            return Ok(false);
        };
        let accepted = credentials::verify_secret(pin, pin_hash, &user.salt).is_accepted();
        if accepted {
            g.last_activity_ts = Utc::now();
        }
        let actor = g.actor.clone();
        let level = g.security_level;
        drop(g);
        if !accepted {
            self.audit.append(
                &actor,
                &format!("L{level}"),
                "auth.pin_rejected",
                "PIN challenge failed",
            );
        }
        Ok(accepted)
    }

    pub fn list_users(&self) -> Result<Vec<UserSummary>, String> {
        let g = self.inner.read();
        let project = g.project.as_ref().ok_or("No project loaded")?;
        Ok(project.users.iter().map(|u| u.to_summary()).collect())
    }

    pub fn save_user(&self, input: UserAccountInput) -> Result<UserSummary, String> {
        let mut g = self.inner.write();
        require_administrator(&g)?;
        if input.username.trim().is_empty() {
            return Err("Username must not be empty".into());
        }
        if let Some(pwd) = input.password.as_deref().map(str::trim) {
            if !pwd.is_empty() && pwd.chars().count() < 12 {
                return Err("Password must be at least 12 characters long".into());
            }
        }
        if let Some(pin) = input.pin.as_deref().map(str::trim) {
            if !pin.is_empty()
                && (pin.chars().count() < 4 || !pin.chars().all(|c| c.is_ascii_digit()))
            {
                return Err("PIN must be at least 4 digits".into());
            }
        }
        let project = g.project.as_mut().ok_or("No project loaded")?;

        let user_idx = if let Some(id) = &input.id {
            project.users.iter().position(|u| u.id == *id)
        } else {
            project
                .users
                .iter()
                .position(|u| u.username.eq_ignore_ascii_case(&input.username))
        };

        let summary = if let Some(idx) = user_idx {
            let u = &mut project.users[idx];
            u.username = input.username.clone();
            u.display_name = input.display_name.clone();
            u.security_level = input.security_level;
            u.enabled = input.enabled;
            if let Some(pwd) = input.password.as_deref().map(str::trim) {
                if !pwd.is_empty() {
                    u.password_hash = credentials::hash_secret(pwd)?;
                    u.password_change_required = false;
                }
            }
            if let Some(pin) = input.pin.as_deref().map(str::trim) {
                if !pin.is_empty() {
                    u.pin_hash = Some(credentials::hash_secret(pin)?);
                }
            }
            u.to_summary()
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
                id: input
                    .id
                    .unwrap_or_else(|| format!("usr_{}", uuid::Uuid::new_v4().simple())),
                username: input.username,
                display_name: input.display_name,
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

        project.recompute_hash();
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
        let mut g = self.inner.write();
        require_administrator(&g)?;
        if g.current_user.as_ref().is_some_and(|u| u.id == user_id) {
            return Err("You cannot delete the account you are signed in with".into());
        }
        let project = g.project.as_mut().ok_or("No project loaded")?;

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

        project.users.retain(|u| u.id != user_id);
        project.recompute_hash();

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

    /// Expire an idle Runtime session.
    ///
    /// Kept separate from [`Engine::snapshot`] so that reading the engine state
    /// is not a state-changing operation.
    pub fn expire_idle_session(&self) {
        let expired = {
            let g = self.inner.read();
            if g.mode != "runtime" || g.security_level == 0 {
                false
            } else {
                let timeout_mins = g
                    .project
                    .as_ref()
                    .map(|p| p.session_config.auto_logout_minutes)
                    .unwrap_or(15);
                timeout_mins > 0
                    && (Utc::now() - g.last_activity_ts).num_seconds()
                        >= i64::from(timeout_mins) * 60
            }
        };
        if expired {
            let _ = self.logout();
        }
    }

    pub fn snapshot(&self) -> EngineSnapshot {
        let g = self.inner.read();
        let now = Utc::now();

        let tags: Vec<TagValue> = g
            .tags
            .values()
            .map(|t| {
                let age = (now - t.ts).num_milliseconds().max(0) as u64;
                let is_internal = t.def.binding.table == ModbusTable::Memory
                    || t.def.binding.table == ModbusTable::System;
                TagValue {
                    tag_id: t.def.id.clone(),
                    value: t.value,
                    bool_value: t.bool_value,
                    quality: if !is_internal && age > 3000 && t.quality == Quality::Good {
                        Quality::Uncertain
                    } else {
                        t.quality
                    },
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

    pub fn stop_polling(&self) {
        let mut g = self.inner.write();
        if let Some(tx) = g.stop_tx.take() {
            let _ = tx.send(true);
        }
        if let Some(h) = g.poll_handle.take() {
            h.abort();
        }
        g.connected = false;
    }

    pub fn start_polling(&self, device_id: Option<String>) -> Result<(), String> {
        self.stop_polling();
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
                    .or_else(|| project.devices.iter().find(|d| d.enabled))
                    .or_else(|| project.devices.first())
                    .cloned()
                    .ok_or_else(|| format!("Device not found: {id}"))?
            } else {
                project
                    .devices
                    .iter()
                    .find(|d| d.enabled)
                    .or_else(|| project.devices.first())
                    .cloned()
                    .ok_or_else(|| "No enabled device in project".to_string())?
            };
            (project, device)
        };

        let (stop_tx, stop_rx) = watch::channel(false);
        let inner = self.inner.clone();
        let audit = self.audit.clone();
        let dev_id = device.id.clone();

        {
            let mut g = inner.write();
            g.device_id = Some(dev_id.clone());
            g.stop_tx = Some(stop_tx);
            g.last_error = None;
        }

        let handle = self.rt.spawn(async move {
            poll_loop(inner, audit, project, device, stop_rx).await;
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

    pub async fn write_tag(&self, tag_id: &str, value: f64) -> Result<(), String> {
        if !value.is_finite() {
            return Err("Write value must be finite".into());
        }
        let (role, actor, def) = {
            let g = self.inner.read();
            if !g.role.can_write() {
                return Err("Role cannot write process values".into());
            }
            if g.mode != "runtime" {
                return Err("Process writes are blocked outside Runtime mode".into());
            }
            if g.current_user
                .as_ref()
                .is_some_and(|u| u.password_change_required)
            {
                return Err("Change the default password before writing to the process".into());
            }
            let project = g.project.as_ref().ok_or("No project")?;
            let tag = project
                .tags
                .iter()
                .find(|t| t.id == tag_id)
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
            (g.role.clone(), g.actor.clone(), tag)
        };

        if def.binding.table == ModbusTable::Memory {
            let bool_val = value != 0.0;
            {
                let mut g = self.inner.write();
                if let Some(live) = g.tags.get_mut(tag_id) {
                    live.value = value;
                    live.bool_value = bool_val;
                    live.quality = Quality::Good;
                    live.ts = Utc::now();
                    live.raw = value as u16;
                }
                let project_alarms = g
                    .project
                    .as_ref()
                    .map(|p| p.alarms.clone())
                    .unwrap_or_default();
                evaluate_alarms(&mut g, &project_alarms);
            }
            self.audit.append(
                &actor,
                role_str(&role),
                "tag.write",
                &format!("{tag_id}={value}"),
            );
            return Ok(());
        }

        let device = {
            let g = self.inner.read();
            let project = g.project.as_ref().ok_or("No project")?;
            project
                .devices
                .iter()
                .find(|d| d.id == def.device_id)
                .ok_or("Device missing")?
                .clone()
        };

        let cfg = ConnectionConfig {
            host: device.host,
            port: device.port,
            unit_id: device.unit_id,
            timeout_ms: device.timeout_ms,
        };
        let addr = def.binding.address;
        let lock = {
            let mut locks = self.write_locks.lock();
            locks
                .entry((def.device_id.clone(), addr))
                .or_insert_with(|| Arc::new(AsyncMutex::new(())))
                .clone()
        };
        let _write_guard = lock.lock().await;
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
                    &actor,
                    role_str(&role),
                    "tag.write_failed",
                    &format!("{tag_id}={value}: {error}"),
                );
                return Err(error);
            }
        };

        {
            let mut g = self.inner.write();
            if let Some(t) = g.tags.get_mut(tag_id) {
                t.raw = raw;
                t.value = engineering_value;
                t.bool_value = bool_value;
                t.quality = Quality::Good;
                t.ts = Utc::now();
            }
            g.last_activity_ts = Utc::now();
        }

        self.audit.append(
            &actor,
            role_str(&role),
            "tag.write",
            &format!(
                "{tag_id}={value} (observed={engineering_value}, raw_readback={raw}, protocol={protocol}, bit={:?}, verify_readback={})",
                def.binding.bit, def.binding.verify_readback
            ),
        );
        Ok(())
    }

    pub fn ack_alarm(&self, def_id: &str) -> Result<(), String> {
        let mut g = self.inner.write();
        if !g.role.can_write() {
            return Err("Role cannot acknowledge alarms".into());
        }
        // The operator interacted with the plant, so the session is alive even
        // when the acknowledgement itself turns out to be a no-op.
        g.last_activity_ts = Utc::now();
        let inst = g.alarms.get_mut(def_id).ok_or("Alarm not found")?;
        let reset =
            matches!(inst.state, AlarmState::ActiveAcked) && inst.latched && !inst.source_active;
        match inst.state {
            AlarmState::ActiveUnacked => inst.state = AlarmState::ActiveAcked,
            AlarmState::ClearedUnacked => inst.state = AlarmState::Inactive,
            AlarmState::ActiveAcked if inst.latched && !inst.source_active => {
                inst.state = AlarmState::Inactive
            }
            _ => return Ok(()),
        };
        inst.last_change = Utc::now();
        let actor = g.actor.clone();
        let role = g.role.clone();
        drop(g);
        let audit_action = if reset { "alarm.reset" } else { "alarm.ack" };
        self.audit
            .append(&actor, role_str(&role), audit_action, def_id);
        Ok(())
    }
}

/// Which credential satisfied a login attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoginKind {
    Password,
    Pin,
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
    project: ScadaProject,
    device: DeviceConfig,
    mut stop_rx: watch::Receiver<bool>,
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
            inner.write().connected = true;
            inner.write().last_error = None;
            Some(c)
        }
        Err(e) => {
            let mut g = inner.write();
            g.connected = false;
            g.last_error = Some(e.to_string());
            mark_all_bad(&mut g);
            None
        }
    };

    loop {
        if *stop_rx.borrow() {
            break;
        }

        let started = Instant::now();

        if ctx.is_none() {
            match modbus::connect(&cfg).await {
                Ok(c) => {
                    ctx = Some(c);
                    let mut g = inner.write();
                    g.connected = true;
                    g.last_error = None;
                }
                Err(e) => {
                    let mut g = inner.write();
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
                                apply_register_values(
                                    &mut inner.write(),
                                    &project,
                                    &device.id,
                                    block,
                                    &values,
                                );
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
                                apply_register_values(
                                    &mut inner.write(),
                                    &project,
                                    &device.id,
                                    block,
                                    &values,
                                );
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
                                apply_bit_values(
                                    &mut inner.write(),
                                    &project,
                                    &device.id,
                                    block,
                                    &values,
                                );
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
                                apply_bit_values(
                                    &mut inner.write(),
                                    &project,
                                    &device.id,
                                    block,
                                    &values,
                                );
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
                g.connected = false;
                g.last_error = Some(error.to_string());
                mark_all_bad(&mut g);
                // Alarms are still evaluated so that every instance is flagged
                // as suspended instead of silently freezing on its last state.
                evaluate_alarms(&mut g, &project.alarms);
            } else {
                let mut g = inner.write();
                g.connected = true;
                g.last_error = None;
                g.poll_count = g.poll_count.wrapping_add(1);
                g.last_poll_ms = started.elapsed().as_millis() as u64;
                evaluate_alarms(&mut g, &project.alarms);
            }
        } else {
            let mut g = inner.write();
            evaluate_alarms(&mut g, &project.alarms);
        }

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(poll_ms)) => {}
            _ = stop_rx.changed() => {
                if *stop_rx.borrow() { break; }
            }
        }
    }

    let mut g = inner.write();
    g.connected = false;
    mark_all_bad(&mut g);
    evaluate_alarms(&mut g, &project.alarms);
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

fn evaluate_alarms(g: &mut EngineInner, defs: &[AlarmDefinition]) {
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

    fn engine_with_water_tank() -> Engine {
        let engine = Engine::new(Arc::new(AuditLog::new()));
        engine
            .load_builtin(water_tank_project())
            .expect("builtin project loads");
        engine
    }

    fn sign_in_admin(engine: &Engine) {
        engine
            .login("admin", Some("admin123"))
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
            .login("operator", Some("operator123"))
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
            .change_password("admin123", "a-much-longer-password")
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
    fn a_default_account_cannot_write_before_changing_its_password() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
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
            .change_password("admin123", "a-much-longer-password")
            .expect("password change");
        assert!(!engine.snapshot().password_change_required);
    }

    #[test]
    fn change_password_enforces_length_and_the_current_secret() {
        let engine = engine_with_water_tank();
        sign_in_admin(&engine);
        assert!(engine.change_password("admin123", "short").is_err());
        assert!(engine
            .change_password("wrong", "a-much-longer-password")
            .is_err());
        assert!(engine
            .change_password("admin123", "a-much-longer-password")
            .is_ok());
        assert!(engine.login("admin", Some("admin123")).is_err());
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
            user.password_hash = crate::project::hash_password("admin123", LEGACY_SALT);
            user.salt = LEGACY_SALT.into();
        }
        engine.login("admin", Some("admin123")).expect("login");
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
    fn pin_challenge_only_accepts_the_signed_in_operator() {
        let engine = engine_with_water_tank();
        assert!(!engine.verify_pin("1234").expect("no session"));

        engine
            .login("operator", Some("operator123"))
            .expect("operator login");
        assert!(engine.verify_pin("1111").expect("own pin"));
        assert!(
            !engine.verify_pin("1234").expect("admin pin"),
            "another account's PIN must not confirm this operator's action"
        );
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
            .login("operator", Some("operator123"))
            .expect("operator login");
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
            .change_password("admin123", "a-much-longer-password")
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
            .login("operator", Some("operator123"))
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
            .login("operator", Some("operator123"))
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
            .change_password("admin123", "a-much-longer-password")
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
            .login("operator", Some("operator123"))
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
    fn acknowledging_an_alarm_counts_as_activity() {
        let engine = engine_with_water_tank();
        engine
            .login("operator", Some("operator123"))
            .expect("operator login");
        engine.inner.write().last_activity_ts = Utc::now() - chrono::Duration::minutes(60);
        let alarm_id = engine
            .snapshot()
            .alarms
            .first()
            .map(|a| a.def_id.clone())
            .expect("water tank defines alarms");

        engine.ack_alarm(&alarm_id).expect("ack");
        engine.inner.write().mode = "runtime".into();
        engine.expire_idle_session();
        assert_eq!(
            engine.snapshot().security_level,
            100,
            "an operator acting on the plant must not be logged out"
        );
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

        let audit = Arc::new(crate::audit::AuditLog::new());
        let engine = Engine::new(audit);
        engine
            .install_project(project, false)
            .expect("install project");

        let snap = engine.snapshot();
        let live = snap
            .tags
            .iter()
            .find(|t| t.tag_id == "mem.sp_calc")
            .expect("live tag");
        assert_eq!(live.quality, Quality::Good);
        assert_eq!(live.value, 4.5);

        engine
            .login("operator", Some("operator123"))
            .expect("login");
        engine
            .change_password("operator123", "a-much-longer-password")
            .expect("change password");
        engine.inner.write().mode = "runtime".into();

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(engine.write_tag("mem.sp_calc", 12.8))
            .expect("write memory tag");

        let snap_after = engine.snapshot();
        let live_after = snap_after
            .tags
            .iter()
            .find(|t| t.tag_id == "mem.sp_calc")
            .expect("live tag after write");
        assert_eq!(live_after.value, 12.8);
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
}
