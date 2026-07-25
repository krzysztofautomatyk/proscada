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
use crate::modbus::{self, ConnectionConfig};
use crate::project::{
    AlarmDefinition, AlarmPriority, BitWriteMode, DeviceConfig, ModbusTable, Role, ScadaProject,
    TagDataType, TagDefinition,
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
    pub group_id: String,
    pub state: AlarmState,
    pub source_active: bool,
    pub latched: bool,
    pub active_since: Option<DateTime<Utc>>,
    pub last_change: DateTime<Utc>,
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
    write_locks: RegisterWriteLocks,
    /// Dedicated Tokio runtime for Modbus I/O and the poll loop.
    /// Sync Tauri commands are not on a reactor — never use bare `tokio::spawn` here.
    rt: tokio::runtime::Runtime,
}

type RegisterWriteLocks = Mutex<HashMap<(String, u16), Arc<AsyncMutex<()>>>>;

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
            write_locks: Mutex::new(HashMap::new()),
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
                    group_id: def.group_id.clone(),
                    state: AlarmState::Inactive,
                    source_active: false,
                    latched: def.latching,
                    active_since: None,
                    last_change: Utc::now(),
                    pending_active_since: None,
                    pending_clear_since: None,
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
        if !value.is_finite() {
            return Err("Write value must be finite".into());
        }
        let (role, actor, device, def) = {
            let g = self.inner.read();
            if !g.role.can_write() {
                return Err("Role cannot write process values".into());
            }
            if g.mode != "runtime" {
                return Err("Process writes are blocked outside Runtime mode".into());
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
            let live = g.tags.get(tag_id).ok_or("Live tag not found")?;
            if live.quality != Quality::Good {
                return Err(format!(
                    "Tag quality must be Good before write (current: {:?})",
                    live.quality
                ));
            }
            let device = project
                .devices
                .iter()
                .find(|d| d.id == tag.device_id)
                .ok_or("Device missing")?
                .clone();
            (g.role.clone(), g.actor.clone(), device, tag)
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
        let verify_readback = def.binding.verify_readback;

        let (raw, bool_value, engineering_value, protocol) = match def.binding.table {
            ModbusTable::Holding => {
                if let Some(bit) = def.binding.bit {
                    if !matches!(def.data_type, TagDataType::Bool) {
                        return Err("Bit binding requires bool data_type".into());
                    }
                    if bit > 15 {
                        return Err(format!("Holding-register bit must be 0..15, got {bit}"));
                    }
                    let requested = value != 0.0;
                    let cfg_for_write = cfg.clone();
                    let readback = match def.binding.bit_write_mode {
                        BitWriteMode::MaskWrite => self
                            .rt
                            .spawn(async move {
                                modbus::write_holding_bit_masked(
                                    &cfg_for_write,
                                    addr,
                                    bit,
                                    requested,
                                    verify_readback,
                                )
                                .await
                            })
                            .await
                            .map_err(|e| format!("bit write task join: {e}"))?
                            .map_err(|e| e.to_string())?,
                        BitWriteMode::ReadModifyWrite => {
                            if !def.binding.single_writer {
                                return Err(
                                    "Read-modify-write requires binding.single_writer=true; use FC22 or a dedicated PLC coil"
                                        .into(),
                                );
                            }
                            self.rt
                                .spawn(async move {
                                    modbus::write_holding_bit_rmw(
                                        &cfg_for_write,
                                        addr,
                                        bit,
                                        requested,
                                        verify_readback,
                                    )
                                    .await
                                })
                                .await
                                .map_err(|e| format!("bit RMW task join: {e}"))?
                                .map_err(|e| e.to_string())?
                        }
                    };
                    let actual = ((readback >> bit) & 1) == 1;
                    (
                        readback,
                        actual,
                        if actual { 1.0 } else { 0.0 },
                        match def.binding.bit_write_mode {
                            BitWriteMode::MaskWrite => "FC22+FC03",
                            BitWriteMode::ReadModifyWrite => "FC03+FC06+FC03(single-writer)",
                        },
                    )
                } else {
                    if def.scale == 0.0 {
                        return Err("Tag scale cannot be zero".into());
                    }
                    let requested_raw = ((value - def.offset) / def.scale).round() as i64;
                    let requested_raw = requested_raw.clamp(0, u16::MAX as i64) as u16;
                    let cfg_for_write = cfg.clone();
                    let readback = self
                        .rt
                        .spawn(async move {
                            modbus::write_holding(
                                &cfg_for_write,
                                addr,
                                requested_raw,
                                verify_readback,
                            )
                            .await
                        })
                        .await
                        .map_err(|e| format!("holding write task join: {e}"))?
                        .map_err(|e| e.to_string())?;
                    (
                        readback,
                        readback != 0,
                        readback as f64 * def.scale + def.offset,
                        "FC06+FC03",
                    )
                }
            }
            ModbusTable::Coil => {
                if def.binding.bit.is_some() {
                    return Err("Coil bindings must not define a register bit index".into());
                }
                if !matches!(def.data_type, TagDataType::Bool) {
                    return Err("Coil binding requires bool data_type".into());
                }
                let requested = value != 0.0;
                let cfg_for_write = cfg.clone();
                let readback = self
                    .rt
                    .spawn(async move {
                        modbus::write_coil(&cfg_for_write, addr, requested, verify_readback).await
                    })
                    .await
                    .map_err(|e| format!("coil write task join: {e}"))?
                    .map_err(|e| e.to_string())?;
                (
                    if readback { 1 } else { 0 },
                    readback,
                    if readback { 1.0 } else { 0.0 },
                    "FC05+FC01",
                )
            }
            ModbusTable::Input | ModbusTable::Discrete => {
                return Err("Input registers and discrete inputs are read-only".into())
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
        }

        self.audit.append(
            &actor,
            role_str(&role),
            "tag.write",
            &format!(
                "{tag_id}={value} (raw_readback={raw}, protocol={protocol}, bit={:?}, verify_readback={})",
                def.binding.bit,
                verify_readback
            ),
        );
        Ok(())
    }

    pub fn ack_alarm(&self, def_id: &str) -> Result<(), String> {
        let mut g = self.inner.write();
        if !g.role.can_write() {
            return Err("Role cannot acknowledge alarms".into());
        }
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
        let mut addresses: Vec<u16> = project
            .tags
            .iter()
            .filter(|tag| tag.device_id == device_id && tag.binding.table == table)
            .map(|tag| tag.binding.address)
            .collect();
        addresses.sort_unstable();
        addresses.dedup();
        let Some(&first) = addresses.first() else {
            continue;
        };
        let max_quantity = match table {
            ModbusTable::Holding | ModbusTable::Input => 120_u16,
            ModbusTable::Coil | ModbusTable::Discrete => 1000_u16,
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
            } else {
                let mut g = inner.write();
                g.connected = true;
                g.last_error = None;
                g.poll_count = g.poll_count.wrapping_add(1);
                g.last_poll_ms = started.elapsed().as_millis() as u64;
                evaluate_alarms(&mut g, &project.alarms);
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
            TagDataType::I16 => {
                let v = (raw as i16) as f64 * def.scale + def.offset;
                live.value = v;
                live.bool_value = v != 0.0;
            }
            TagDataType::U16 | TagDataType::F32 => {
                let v = raw as f64 * def.scale + def.offset;
                live.value = v;
                live.bool_value = v != 0.0;
            }
        }
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
        let Some(tag) = g.tags.get(&def.tag_id) else {
            continue;
        };
        if tag.quality != Quality::Good {
            continue;
        }
        let was_source_active = g
            .alarms
            .get(&def.id)
            .map(|alarm| alarm.source_active)
            .unwrap_or(false);
        let condition_active = if matches!(tag.def.data_type, TagDataType::Bool) {
            tag.bool_value == def.when_true
        } else if let Some(hi) = def.hi_limit {
            let threshold = if was_source_active {
                hi - def.deadband.max(0.0)
            } else {
                hi
            };
            tag.value >= threshold
        } else if let Some(lo) = def.lo_limit {
            let threshold = if was_source_active {
                lo + def.deadband.max(0.0)
            } else {
                lo
            };
            tag.value <= threshold
        } else {
            false
        };

        let Some(inst) = g.alarms.get_mut(&def.id) else {
            continue;
        };
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
    use crate::project::{water_tank_project, BitWriteMode, TagBinding};

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
            alarms: HashMap::from([(
                alarm.id.clone(),
                AlarmInstance {
                    def_id: alarm.id.clone(),
                    name: alarm.name.clone(),
                    message: alarm.message.clone(),
                    priority: alarm.priority.clone(),
                    group_id: alarm.group_id.clone(),
                    state: AlarmState::Inactive,
                    source_active: false,
                    latched: alarm.latching,
                    active_since: None,
                    last_change: Utc::now(),
                    pending_active_since: None,
                    pending_clear_since: None,
                },
            )]),
            connected: true,
            device_id: Some("test-device".into()),
            last_error: None,
            poll_count: 0,
            last_poll_ms: 0,
            role: Role::Engineer,
            actor: "test".into(),
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
            },
            unit: String::new(),
            description: String::new(),
            scale: 1.0,
            offset: 0.0,
            decimals: 0,
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
}
