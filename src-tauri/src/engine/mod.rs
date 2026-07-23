//! Real-time tag engine, polling scheduler, alarm evaluation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::audit::AuditLog;
use crate::modbus::{self, ConnectionConfig};
use crate::project::{
    AlarmDefinition, AlarmPriority, DeviceConfig, ModbusTable, Role, ScadaProject, TagDataType,
    TagDefinition,
};

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
    pub state: AlarmState,
    pub active_since: Option<DateTime<Utc>>,
    pub last_change: DateTime<Utc>,
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
    pub project_name: Option<String>,
    pub mode: String,
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
    mode: String, // designer | runtime
    poll_handle: Option<JoinHandle<()>>,
    stop_tx: Option<watch::Sender<bool>>,
}

pub struct Engine {
    inner: Arc<RwLock<EngineInner>>,
    audit: Arc<AuditLog>,
    /// Dedicated Tokio runtime for Modbus I/O and the poll loop.
    /// Sync Tauri commands are not on a reactor — never use bare `tokio::spawn` here.
    rt: tokio::runtime::Runtime,
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
            inner: Arc::new(RwLock::new(EngineInner {
                project: None,
                tags: HashMap::new(),
                alarms: HashMap::new(),
                connected: false,
                device_id: None,
                last_error: None,
                poll_count: 0,
                last_poll_ms: 0,
                role: Role::Engineer,
                actor: "engineer".into(),
                mode: "designer".into(),
                poll_handle: None,
                stop_tx: None,
            })),
            audit,
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

    pub fn load_project(&self, project: ScadaProject) -> Result<(), String> {
        if !project.verify_hash() {
            return Err("Project content hash verification failed".into());
        }
        self.stop_polling();
        let mut g = self.inner.write();
        let mut tags = HashMap::new();
        for def in &project.tags {
            tags.insert(
                def.id.clone(),
                LiveTag {
                    def: def.clone(),
                    value: 0.0,
                    bool_value: false,
                    quality: Quality::Bad,
                    ts: Utc::now(),
                    raw: 0,
                },
            );
        }
        let mut alarms = HashMap::new();
        for def in &project.alarms {
            alarms.insert(
                def.id.clone(),
                AlarmInstance {
                    def_id: def.id.clone(),
                    name: def.name.clone(),
                    message: def.message.clone(),
                    priority: def.priority.clone(),
                    state: AlarmState::Inactive,
                    active_since: None,
                    last_change: Utc::now(),
                },
            );
        }
        let name = project.name.clone();
        g.project = Some(project);
        g.tags = tags;
        g.alarms = alarms;
        g.connected = false;
        g.device_id = None;
        g.last_error = None;
        g.poll_count = 0;
        drop(g);
        self.audit.append(
            &self.inner.read().actor,
            role_str(&self.inner.read().role),
            "project.load",
            &name,
        );
        Ok(())
    }

    pub fn get_project(&self) -> Option<ScadaProject> {
        self.inner.read().project.clone()
    }

    pub fn set_project_mut(&self, project: ScadaProject) -> Result<(), String> {
        {
            let g = self.inner.read();
            if !g.role.can_edit_project() {
                return Err("Role cannot edit project".into());
            }
        }
        let mut p = project;
        p.recompute_hash();
        self.load_project(p)
    }

    pub fn set_role(&self, role: Role, actor: String) {
        let mut g = self.inner.write();
        g.role = role.clone();
        g.actor = actor.clone();
        drop(g);
        self.audit
            .append(&actor, role_str(&role), "role.set", role_str(&role));
    }

    pub fn set_mode(&self, mode: String) {
        let mut g = self.inner.write();
        g.mode = mode.clone();
        let actor = g.actor.clone();
        let role = g.role.clone();
        drop(g);
        self.audit
            .append(&actor, role_str(&role), "mode.set", &mode);
    }

    pub fn snapshot(&self) -> EngineSnapshot {
        let g = self.inner.read();
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
                    quality: if age > 3000 && t.quality == Quality::Good {
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
        EngineSnapshot {
            connected: g.connected,
            device_id: g.device_id.clone(),
            last_error: g.last_error.clone(),
            poll_count: g.poll_count,
            last_poll_ms: g.last_poll_ms,
            tags,
            alarms: g.alarms.values().cloned().collect(),
            role: g.role.clone(),
            actor: g.actor.clone(),
            project_name: g.project.as_ref().map(|p| p.name.clone()),
            mode: g.mode.clone(),
        }
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
                    .cloned()
                    .ok_or_else(|| format!("Device not found: {id}"))?
            } else {
                project
                    .devices
                    .iter()
                    .find(|d| d.enabled)
                    .cloned()
                    .ok_or_else(|| "No enabled device".to_string())?
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

    pub async fn write_tag(&self, tag_id: &str, value: f64) -> Result<(), String> {
        let (role, actor, device, def) = {
            let g = self.inner.read();
            if !g.role.can_write() {
                return Err("Role cannot write process values".into());
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
            if tag.binding.bit.is_some() {
                return Err("Bit tags are read-only in v1".into());
            }
            if !matches!(tag.binding.table, ModbusTable::Holding) {
                return Err("Only holding registers are writable in v1".into());
            }
            let device = project
                .devices
                .iter()
                .find(|d| d.id == tag.device_id)
                .ok_or("Device missing")?
                .clone();
            (g.role.clone(), g.actor.clone(), device, tag)
        };

        let raw = ((value - def.offset) / def.scale).round() as i64;
        let raw = raw.clamp(0, u16::MAX as i64) as u16;
        let cfg = ConnectionConfig {
            host: device.host,
            port: device.port,
            unit_id: device.unit_id,
            timeout_ms: device.timeout_ms,
        };

        // Run Modbus write on the engine runtime (not the Tauri command thread).
        let addr = def.binding.address;
        let write_result = self
            .rt
            .spawn(async move { modbus::write_holding(&cfg, addr, raw).await })
            .await
            .map_err(|e| format!("write task join: {e}"))?;
        write_result.map_err(|e| e.to_string())?;

        {
            let mut g = self.inner.write();
            if let Some(t) = g.tags.get_mut(tag_id) {
                t.raw = raw;
                t.value = raw as f64 * def.scale + def.offset;
                t.quality = Quality::Good;
                t.ts = Utc::now();
            }
        }

        self.audit.append(
            &actor,
            role_str(&role),
            "tag.write",
            &format!("{tag_id}={value} (raw={raw})"),
        );
        Ok(())
    }

    pub fn ack_alarm(&self, def_id: &str) -> Result<(), String> {
        let mut g = self.inner.write();
        if !g.role.can_write() {
            return Err("Role cannot acknowledge alarms".into());
        }
        let inst = g.alarms.get_mut(def_id).ok_or("Alarm not found")?;
        match inst.state {
            AlarmState::ActiveUnacked => inst.state = AlarmState::ActiveAcked,
            AlarmState::ClearedUnacked => inst.state = AlarmState::Inactive,
            _ => {}
        }
        inst.last_change = Utc::now();
        let actor = g.actor.clone();
        let role = g.role.clone();
        drop(g);
        self.audit
            .append(&actor, role_str(&role), "alarm.ack", def_id);
        Ok(())
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

    // Water tank optimized block: HR100 qty 22
    let block_start: u16 = 100;
    let block_qty: u16 = 22;

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
            match modbus::read_holding(c, block_start, block_qty, device.timeout_ms).await {
                Ok(regs) => {
                    let mut g = inner.write();
                    g.connected = true;
                    g.last_error = None;
                    g.poll_count = g.poll_count.wrapping_add(1);
                    g.last_poll_ms = started.elapsed().as_millis() as u64;
                    apply_registers(&mut g, &project, block_start, &regs);
                    evaluate_alarms(&mut g, &project.alarms);
                }
                Err(e) => {
                    ctx = None;
                    let mut g = inner.write();
                    g.connected = false;
                    g.last_error = Some(e.to_string());
                    mark_all_bad(&mut g);
                }
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
    g.connected = false;
}

fn mark_all_bad(g: &mut EngineInner) {
    for t in g.tags.values_mut() {
        t.quality = Quality::Bad;
    }
}

fn apply_registers(
    g: &mut EngineInner,
    project: &ScadaProject,
    block_start: u16,
    regs: &[u16],
) {
    let now = Utc::now();
    for def in &project.tags {
        if def.device_id != g.device_id.as_deref().unwrap_or("")
            && g.device_id.is_some()
            && def.device_id != project.devices.first().map(|d| d.id.as_str()).unwrap_or("")
        {
            // still apply for primary device tags
        }
        if !matches!(def.binding.table, ModbusTable::Holding) {
            continue;
        }
        let addr = def.binding.address;
        if addr < block_start {
            continue;
        }
        let idx = (addr - block_start) as usize;
        if idx >= regs.len() {
            continue;
        }
        let raw = regs[idx];
        let Some(live) = g.tags.get_mut(&def.id) else {
            continue;
        };
        live.raw = raw;
        live.ts = now;
        live.quality = Quality::Good;
        match def.data_type {
            TagDataType::Bool => {
                let bit = def.binding.bit.unwrap_or(0) as u16;
                let b = (raw >> bit) & 1 == 1;
                live.bool_value = b;
                live.value = if b { 1.0 } else { 0.0 };
            }
            TagDataType::U16 | TagDataType::I16 | TagDataType::F32 => {
                let v = raw as f64 * def.scale + def.offset;
                live.value = v;
                live.bool_value = v != 0.0;
            }
        }
    }
}

fn evaluate_alarms(g: &mut EngineInner, defs: &[AlarmDefinition]) {
    let now = Utc::now();
    for def in defs {
        let Some(tag) = g.tags.get(&def.tag_id) else {
            continue;
        };
        let active = if matches!(tag.def.data_type, TagDataType::Bool) {
            tag.bool_value == def.when_true && tag.quality == Quality::Good
        } else if let Some(hi) = def.hi_limit {
            tag.value >= hi && tag.quality == Quality::Good
        } else if let Some(lo) = def.lo_limit {
            tag.value <= lo && tag.quality == Quality::Good
        } else {
            false
        };

        let Some(inst) = g.alarms.get_mut(&def.id) else {
            continue;
        };

        match (&inst.state, active) {
            (AlarmState::Inactive, true) => {
                inst.state = AlarmState::ActiveUnacked;
                inst.active_since = Some(now);
                inst.last_change = now;
            }
            (AlarmState::ActiveUnacked | AlarmState::ActiveAcked, false) => {
                inst.state = if matches!(inst.state, AlarmState::ActiveAcked) {
                    AlarmState::Inactive
                } else {
                    AlarmState::ClearedUnacked
                };
                inst.last_change = now;
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
