//! Project schema — devices, tags, forms, widgets, alarms, solution tree.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub mod credentials;

pub const SCHEMA_VERSION: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Viewer,
    Operator,
    Engineer,
    Administrator,
}

impl Role {
    pub fn can_write(&self) -> bool {
        matches!(self, Role::Operator | Role::Engineer | Role::Administrator)
    }

    pub fn can_edit_project(&self) -> bool {
        matches!(self, Role::Engineer | Role::Administrator)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModbusTable {
    Holding,
    Input,
    Coil,
    Discrete,
    Memory,
    System,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BitWriteMode {
    #[default]
    MaskWrite,
    ReadModifyWrite,
}

fn is_default_bit_write_mode(mode: &BitWriteMode) -> bool {
    matches!(mode, BitWriteMode::MaskWrite)
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_true(value: &bool) -> bool {
    *value
}

fn default_true() -> bool {
    true
}

fn is_zero_f64(value: &f64) -> bool {
    *value == 0.0
}

fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TagDataType {
    Bool,
    U16,
    I16,
    U32,
    I32,
    F32,
    U64,
    I64,
    F64,
    String,
}

impl TagDataType {
    /// Number of 16-bit Modbus registers occupied by one value of this type.
    /// `String` has no fixed width and is rejected on Modbus tables by
    /// [`ScadaProject::validate`].
    pub fn register_count(&self) -> Option<u16> {
        match self {
            TagDataType::Bool | TagDataType::U16 | TagDataType::I16 => Some(1),
            TagDataType::U32 | TagDataType::I32 | TagDataType::F32 => Some(2),
            TagDataType::U64 | TagDataType::I64 | TagDataType::F64 => Some(4),
            TagDataType::String => None,
        }
    }
}

/// Register order for values wider than one holding/input register.
/// `HighWordFirst` matches the Modbus convention used by most PLC vendors.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WordOrder {
    #[default]
    HighWordFirst,
    LowWordFirst,
}

fn is_default_word_order(order: &WordOrder) -> bool {
    matches!(order, WordOrder::HighWordFirst)
}

fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}

fn default_query_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusQueryConfig {
    pub id: String,
    pub name: String,
    pub table: ModbusTable,
    pub start_address: u16,
    pub count: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll_ms: Option<u64>,
    #[serde(default = "default_query_enabled")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub unit_id: u8,
    pub poll_ms: u64,
    pub timeout_ms: u64,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queries: Vec<ModbusQueryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagBinding {
    /// Source holding/input address (word).
    pub address: u16,
    /// Optional bit index 0..15 for packed bools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bit: Option<u8>,
    pub table: ModbusTable,
    #[serde(default)]
    pub writable: bool,
    #[serde(default, skip_serializing_if = "is_default_bit_write_mode")]
    pub bit_write_mode: BitWriteMode,
    #[serde(default, skip_serializing_if = "is_false")]
    pub single_writer: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub verify_readback: bool,
    /// Register order for multi-register data types.
    #[serde(default, skip_serializing_if = "is_default_word_order")]
    pub word_order: WordOrder,
    /// Backend-enforced minimum security level required to write this tag.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub min_security_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDefinition {
    pub id: String,
    pub name: String,
    pub device_id: String,
    pub data_type: TagDataType,
    pub binding: TagBinding,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_scale")]
    pub scale: f64,
    #[serde(default)]
    pub offset: f64,
    #[serde(default)]
    pub decimals: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_value: Option<String>,
}

fn default_scale() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetDef {
    pub id: String,
    pub widget_type: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    #[serde(default)]
    pub z: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_level: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unauthorized_behavior: Option<String>,
    #[serde(default)]
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormDef {
    pub id: String,
    pub name: String,
    #[serde(default = "default_form_w")]
    pub width: f64,
    #[serde(default = "default_form_h")]
    pub height: f64,
    #[serde(default = "default_bg")]
    pub background: String,
    #[serde(default = "default_grid")]
    pub grid: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_level: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unauthorized_behavior: Option<String>,
    pub widgets: Vec<WidgetDef>,
}

fn default_form_w() -> f64 {
    1280.0
}
fn default_form_h() -> f64 {
    800.0
}
fn default_bg() -> String {
    "#0b1220".into()
}
fn default_grid() -> u32 {
    8
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlarmPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmDefinition {
    pub id: String,
    pub name: String,
    pub tag_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub group_id: String,
    pub priority: AlarmPriority,
    /// For bool tags: alarm when true. For numeric: use limit.
    #[serde(default)]
    pub when_true: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hi_limit: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lo_limit: Option<f64>,
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub deadband: f64,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub on_delay_ms: u64,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub off_delay_ms: u64,
    #[serde(default, skip_serializing_if = "is_false")]
    pub latching: bool,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmGroupDefinition {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectNodeKind {
    Folder,
    Screen,
    Variables,
    Script,
    Note,
    Markdown,
    Image,
    /// Root folder for component templates in Solution Explorer
    ComponentsFolder,
    /// A single component template entry
    Component,
}

/// Solution Explorer node (folders, screens, scripts, docs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectNode {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub kind: ProjectNodeKind,
    pub name: String,
    #[serde(default)]
    pub order: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ref_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collapsed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub salt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin_hash: Option<String>,
    pub security_level: u32,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Set on seeded/default accounts. While true the backend refuses process
    /// writes and user administration for this account.
    #[serde(default, skip_serializing_if = "is_false")]
    pub password_change_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub security_level: u32,
    pub enabled: bool,
    pub has_pin: bool,
    #[serde(default)]
    pub password_change_required: bool,
}

impl UserAccount {
    pub fn to_summary(&self) -> UserSummary {
        UserSummary {
            id: self.id.clone(),
            username: self.username.clone(),
            display_name: self.display_name.clone(),
            security_level: self.security_level,
            enabled: self.enabled,
            has_pin: self.pin_hash.is_some(),
            password_change_required: self.password_change_required,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(default = "default_auto_logout_minutes")]
    pub auto_logout_minutes: u32,
    #[serde(default)]
    pub pin_challenge_on_write: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            auto_logout_minutes: 15,
            pin_challenge_on_write: false,
        }
    }
}

fn default_auto_logout_minutes() -> u32 {
    15
}

#[cfg(test)]
pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

/// Legacy salt used by projects created before Argon2id was introduced.
pub const LEGACY_SALT: &str = "proscada_salt";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScadaProject {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub devices: Vec<DeviceConfig>,
    pub tags: Vec<TagDefinition>,
    pub forms: Vec<FormDef>,
    pub alarms: Vec<AlarmDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alarm_groups: Vec<AlarmGroupDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<UserAccount>,
    #[serde(default)]
    pub session_config: SessionConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub design_system: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_templates: Vec<serde_json::Value>,
    /// Hierarchical Solution Explorer items. Missing in v1 files → empty.
    #[serde(default)]
    pub tree: Vec<ProjectNode>,
    #[serde(default)]
    pub content_hash: String,
}

impl ScadaProject {
    /// Migrate projects created with the public demonstration credentials.
    ///
    /// Old files did not always carry `password_change_required`, and some
    /// shipped factory PINs. Detect the known secrets using the stored hash
    /// format, force first-use password replacement, and remove those PINs.
    pub fn harden_seeded_accounts(&mut self) {
        for user in &mut self.users {
            let default_password = match user.username.to_ascii_lowercase().as_str() {
                "admin" => Some("admin123"),
                "operator" => Some("operator123"),
                _ => None,
            };
            if default_password.is_some_and(|secret| {
                credentials::verify_secret(secret, &user.password_hash, &user.salt).is_accepted()
            }) {
                user.password_change_required = true;
            }

            let factory_pin = match user.username.to_ascii_lowercase().as_str() {
                "admin" => Some("1234"),
                "operator" => Some("1111"),
                _ => None,
            };
            if let (Some(secret), Some(pin_hash)) = (factory_pin, user.pin_hash.as_ref()) {
                if credentials::verify_secret(secret, pin_hash, &user.salt).is_accepted() {
                    user.pin_hash = None;
                }
            }
        }
    }

    /// Reject project content the engine cannot represent faithfully.
    ///
    /// Every rule here exists because the alternative would be a silently wrong
    /// process value or an unenforceable write gate.
    pub fn validate(&self) -> Result<(), String> {
        use std::collections::HashSet;

        if self.schema_version != SCHEMA_VERSION {
            return Err(format!(
                "Unsupported schema_version {}; expected {}",
                self.schema_version, SCHEMA_VERSION
            ));
        }
        if self.id.trim().is_empty() || self.name.trim().is_empty() {
            return Err("Project id and name must not be empty".into());
        }
        if self.session_config.auto_logout_minutes > 24 * 60 {
            return Err("Session auto-logout must be between 0 and 1440 minutes".into());
        }
        if self.devices.iter().filter(|device| device.enabled).count() > 1 {
            return Err(
                "Current runtime supports exactly one active device; disable additional devices"
                    .into(),
            );
        }

        let mut device_ids = HashSet::new();
        for device in &self.devices {
            if device.id.trim().is_empty() || !device_ids.insert(device.id.as_str()) {
                return Err(format!("Duplicate or empty device id: {}", device.id));
            }
            if device.host.trim().is_empty() {
                return Err(format!("Device {} has an empty host", device.id));
            }
            if device.port == 0 {
                return Err(format!("Device {} has invalid port 0", device.id));
            }
            if !(100..=600_000).contains(&device.poll_ms) {
                return Err(format!("Device {} poll_ms must be 100..=600000", device.id));
            }
            if !(50..=60_000).contains(&device.timeout_ms) {
                return Err(format!(
                    "Device {} timeout_ms must be 50..=60000",
                    device.id
                ));
            }
            if !device.queries.is_empty() {
                return Err(format!(
                    "Device {} defines queries, but the current runtime does not support query schedules",
                    device.id
                ));
            }
            let mut query_ids = HashSet::new();
            for query in &device.queries {
                if query.id.trim().is_empty() || !query_ids.insert(query.id.as_str()) {
                    return Err(format!(
                        "Device {} has a duplicate or empty query id: {}",
                        device.id, query.id
                    ));
                }
                if query.count == 0 {
                    return Err(format!("Query {} has a zero register count", query.id));
                }
                if let Some(poll_ms) = query.poll_ms {
                    if !(100..=600_000).contains(&poll_ms) {
                        return Err(format!("Query {} poll_ms must be 100..=600000", query.id));
                    }
                }
            }
        }
        let mut seen_tag_ids = std::collections::HashSet::new();

        for tag in &self.tags {
            if tag.id.trim().is_empty() || !seen_tag_ids.insert(tag.id.as_str()) {
                return Err(format!("Duplicate or empty tag id: {}", tag.id));
            }
            let is_internal =
                matches!(tag.binding.table, ModbusTable::Memory | ModbusTable::System);
            if is_internal && tag.device_id != "SYS_INTERNAL" {
                return Err(format!(
                    "Internal tag {} must use device SYS_INTERNAL",
                    tag.id
                ));
            }
            if !is_internal && !device_ids.contains(tag.device_id.as_str()) {
                return Err(format!(
                    "Tag {} references unknown device {}",
                    tag.id, tag.device_id
                ));
            }
            if tag.scale == 0.0 || !tag.scale.is_finite() {
                return Err(format!("Tag {} has a zero or non-finite scale", tag.id));
            }
            if !tag.offset.is_finite() {
                return Err(format!("Tag {} has a non-finite offset", tag.id));
            }
            if tag.binding.min_security_level > 1000 {
                return Err(format!(
                    "Tag {} min_security_level must be 0..=1000",
                    tag.id
                ));
            }

            let binding = &tag.binding;
            match binding.table {
                ModbusTable::Memory | ModbusTable::System => {
                    if tag.data_type == TagDataType::String {
                        return Err(format!(
                            "Internal tag {} uses unsupported String data type",
                            tag.id
                        ));
                    }
                    if binding.table == ModbusTable::System && binding.writable {
                        return Err(format!("System tag {} must be read-only", tag.id));
                    }
                    if binding.bit.is_some() {
                        return Err(format!(
                            "Internal tag {} must not define a Modbus register bit",
                            tag.id
                        ));
                    }
                    if binding.table == ModbusTable::System {
                        validate_system_tag(tag)?;
                    }
                    if let Some(initial) = tag.initial_value.as_deref() {
                        let parsed = parse_initial_value(tag, initial)?;
                        let scaled = (parsed - tag.offset) / tag.scale;
                        crate::modbus::codec::encode(tag.data_type, tag.binding.word_order, scaled)
                            .map_err(|error| {
                                format!("Tag {} initial_value is invalid: {error}", tag.id)
                            })?;
                    }
                }
                ModbusTable::Coil | ModbusTable::Discrete => {
                    if binding.bit.is_some() {
                        return Err(format!(
                            "Tag {} binds a bit index to a coil/discrete address",
                            tag.id
                        ));
                    }
                    if tag.data_type != TagDataType::Bool {
                        return Err(format!("Tag {} on a bit table must be bool", tag.id));
                    }
                }
                ModbusTable::Holding | ModbusTable::Input => {
                    if let Some(bit) = binding.bit {
                        if bit > 15 {
                            return Err(format!(
                                "Tag {} uses register bit {bit}; only 0..15 exist",
                                tag.id
                            ));
                        }
                        if tag.data_type != TagDataType::Bool {
                            return Err(format!(
                                "Tag {} binds a register bit but is not bool",
                                tag.id
                            ));
                        }
                        if binding.writable
                            && binding.bit_write_mode == BitWriteMode::ReadModifyWrite
                            && !binding.single_writer
                        {
                            return Err(format!(
                                "Tag {} uses read-modify-write without single_writer=true",
                                tag.id
                            ));
                        }
                    }
                    let Some(count) = tag.data_type.register_count() else {
                        return Err(format!(
                            "Tag {} uses data type {:?}, which has no fixed Modbus width",
                            tag.id, tag.data_type
                        ));
                    };
                    if binding.address.checked_add(count - 1).is_none() {
                        return Err(format!(
                            "Tag {} spans past the end of the register space",
                            tag.id
                        ));
                    }
                }
            }

            if binding.writable
                && matches!(
                    binding.table,
                    ModbusTable::Input | ModbusTable::Discrete | ModbusTable::System
                )
            {
                return Err(format!(
                    "Tag {} is marked writable but its table is read-only",
                    tag.id
                ));
            }
        }

        let tag_ids: HashSet<&str> = self.tags.iter().map(|t| t.id.as_str()).collect();
        let mut alarm_ids = HashSet::new();
        for alarm in &self.alarms {
            if alarm.id.trim().is_empty() || !alarm_ids.insert(alarm.id.as_str()) {
                return Err(format!("Duplicate or empty alarm id: {}", alarm.id));
            }
            if !tag_ids.contains(alarm.tag_id.as_str()) {
                return Err(format!(
                    "Alarm {} references unknown tag {}",
                    alarm.id, alarm.tag_id
                ));
            }
            let source = self
                .tags
                .iter()
                .find(|tag| tag.id == alarm.tag_id)
                .expect("tag id was checked");
            if source.data_type == TagDataType::Bool {
                if alarm.hi_limit.is_some() || alarm.lo_limit.is_some() {
                    return Err(format!(
                        "Boolean alarm {} must not define numeric limits",
                        alarm.id
                    ));
                }
            } else if alarm.hi_limit.is_some() == alarm.lo_limit.is_some() {
                return Err(format!(
                    "Numeric alarm {} must define exactly one of hi_limit or lo_limit",
                    alarm.id
                ));
            }
            if !alarm.deadband.is_finite() || alarm.deadband < 0.0 {
                return Err(format!(
                    "Alarm {} has a negative or non-finite deadband",
                    alarm.id
                ));
            }
            if alarm.hi_limit.is_some_and(|v| !v.is_finite())
                || alarm.lo_limit.is_some_and(|v| !v.is_finite())
            {
                return Err(format!("Alarm {} has a non-finite limit", alarm.id));
            }
        }

        let mut group_ids = HashSet::new();
        for group in &self.alarm_groups {
            if group.id.trim().is_empty() || !group_ids.insert(group.id.as_str()) {
                return Err(format!("Duplicate or empty alarm group id: {}", group.id));
            }
        }
        for group in &self.alarm_groups {
            if group
                .parent_id
                .as_deref()
                .is_some_and(|parent| parent == group.id || !group_ids.contains(parent))
            {
                return Err(format!(
                    "Alarm group {} references an invalid parent",
                    group.id
                ));
            }
        }
        for group in &self.alarm_groups {
            let mut path = HashSet::new();
            let mut current = Some(group.id.as_str());
            while let Some(id) = current {
                if !path.insert(id) {
                    return Err(format!("Alarm group hierarchy contains a cycle at {id}"));
                }
                current = self
                    .alarm_groups
                    .iter()
                    .find(|candidate| candidate.id == id)
                    .and_then(|candidate| candidate.parent_id.as_deref());
            }
        }
        for alarm in &self.alarms {
            if !alarm.group_id.is_empty() && !group_ids.contains(alarm.group_id.as_str()) {
                return Err(format!(
                    "Alarm {} references unknown group {}",
                    alarm.id, alarm.group_id
                ));
            }
        }

        let mut form_ids = HashSet::new();
        let mut widget_ids = HashSet::new();
        for form in &self.forms {
            if form.id.trim().is_empty() || !form_ids.insert(form.id.as_str()) {
                return Err(format!("Duplicate or empty form id: {}", form.id));
            }
            if !form.width.is_finite()
                || !form.height.is_finite()
                || form.width <= 0.0
                || form.height <= 0.0
            {
                return Err(format!("Form {} has invalid dimensions", form.id));
            }
            for widget in &form.widgets {
                if widget.id.trim().is_empty() || !widget_ids.insert(widget.id.as_str()) {
                    return Err(format!("Duplicate or empty widget id: {}", widget.id));
                }
                if widget.min_level.is_some_and(|level| level > 1000) {
                    return Err(format!("Widget {} min_level must be 0..=1000", widget.id));
                }
                if !widget.x.is_finite()
                    || !widget.y.is_finite()
                    || !widget.w.is_finite()
                    || !widget.h.is_finite()
                    || widget.w <= 0.0
                    || widget.h <= 0.0
                {
                    return Err(format!("Widget {} has invalid geometry", widget.id));
                }
            }
        }

        for (index, template) in self.component_templates.iter().enumerate() {
            if !template.is_object() {
                return Err(format!("Component template {index} must be a JSON object"));
            }
            if json_contains_disallowed_content(template) {
                return Err(format!(
                    "Component template {index} contains executable or remote content"
                ));
            }
        }

        let mut node_ids = HashSet::new();
        for node in &self.tree {
            if node.id.trim().is_empty() || !node_ids.insert(node.id.as_str()) {
                return Err(format!(
                    "Duplicate or empty project tree node id: {}",
                    node.id
                ));
            }
        }
        for node in &self.tree {
            if node
                .parent_id
                .as_deref()
                .is_some_and(|parent| parent == node.id || !node_ids.contains(parent))
            {
                return Err(format!(
                    "Project tree node {} references an invalid parent",
                    node.id
                ));
            }
            if node.kind == ProjectNodeKind::Script
                && node.language.as_deref() != Some("proscada-actions")
            {
                return Err(format!(
                    "Script node {} must use language proscada-actions",
                    node.id
                ));
            }
            if node
                .content
                .as_deref()
                .is_some_and(contains_disallowed_content)
            {
                return Err(format!(
                    "Project tree node {} contains executable or remote content",
                    node.id
                ));
            }
        }
        for node in &self.tree {
            let mut path = HashSet::new();
            let mut current = Some(node.id.as_str());
            while let Some(id) = current {
                if !path.insert(id) {
                    return Err(format!("Project tree contains a cycle at {id}"));
                }
                current = self
                    .tree
                    .iter()
                    .find(|candidate| candidate.id == id)
                    .and_then(|candidate| candidate.parent_id.as_deref());
            }
        }

        let mut user_ids = HashSet::new();
        let mut usernames = HashSet::new();
        for user in &self.users {
            let normalized = user.username.trim().to_ascii_lowercase();
            if user.id.trim().is_empty() || !user_ids.insert(user.id.as_str()) {
                return Err(format!("Duplicate or empty user id: {}", user.id));
            }
            if normalized.is_empty() || !usernames.insert(normalized) {
                return Err(format!("Duplicate or empty username: {}", user.username));
            }
            if user.security_level > 1000 {
                return Err(format!(
                    "User {} security_level must be 0..=1000",
                    user.username
                ));
            }
        }
        if !self.users.is_empty()
            && !self
                .users
                .iter()
                .any(|u| u.enabled && u.security_level >= 1000)
        {
            return Err("Project must retain at least one active Administrator".into());
        }
        Ok(())
    }

    pub fn recompute_hash(&mut self) {
        self.content_hash.clear();
        let body = serde_json::to_vec(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(&body);
        self.content_hash = hex::encode(hasher.finalize());
    }

    pub fn verify_hash(&self) -> bool {
        if self.content_hash.is_empty() {
            return true;
        }
        let mut clone = self.clone();
        let expected = clone.content_hash.clone();
        clone.recompute_hash();
        clone.content_hash == expected
    }
}

fn parse_initial_value(tag: &TagDefinition, initial: &str) -> Result<f64, String> {
    if tag.data_type == TagDataType::Bool {
        return match initial.trim().to_ascii_lowercase().as_str() {
            "true" | "1" => Ok(1.0),
            "false" | "0" => Ok(0.0),
            _ => Err(format!(
                "Tag {} initial_value must be true, false, 0 or 1",
                tag.id
            )),
        };
    }
    initial
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("Tag {} initial_value is not numeric", tag.id))
        .and_then(|value| {
            if value.is_finite() {
                Ok(value)
            } else {
                Err(format!("Tag {} initial_value must be finite", tag.id))
            }
        })
}

fn validate_system_tag(tag: &TagDefinition) -> Result<(), String> {
    let expected = match tag.id.as_str() {
        "system.connected" | "system.mode" => TagDataType::Bool,
        "system.poll_count" | "system.last_poll_ms" => TagDataType::U64,
        "system.security_level" => TagDataType::U16,
        _ => {
            return Err(format!(
                "Unknown System tag {}; allowed ids are system.connected, system.poll_count, system.last_poll_ms, system.security_level and system.mode",
                tag.id
            ))
        }
    };
    if tag.data_type != expected {
        return Err(format!(
            "System tag {} must use data type {:?}",
            tag.id, expected
        ));
    }
    if tag.scale != 1.0 || tag.offset != 0.0 {
        return Err(format!(
            "System tag {} must use scale=1 and offset=0",
            tag.id
        ));
    }
    if tag.initial_value.is_some() {
        return Err(format!(
            "System tag {} cannot define an initial_value",
            tag.id
        ));
    }
    Ok(())
}

fn json_contains_disallowed_content(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(map) => map.iter().any(|(key, value)| {
            contains_disallowed_content(key) || json_contains_disallowed_content(value)
        }),
        serde_json::Value::Array(values) => values.iter().any(json_contains_disallowed_content),
        serde_json::Value::String(value) => contains_disallowed_content(value),
        _ => false,
    }
}

fn contains_disallowed_content(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    [
        "javascript:",
        "eval(",
        "new function",
        "<script",
        "http://",
        "https://",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

/// Built-in Water Tank project matching PLC LAD SIM map HR100–121.
pub fn water_tank_project() -> ScadaProject {
    let device = DeviceConfig {
        id: "plc_water_tank".into(),
        name: "PLC_WaterTank".into(),
        host: "127.0.0.1".into(),
        port: 5020,
        unit_id: 1,
        poll_ms: 250,
        timeout_ms: 800,
        enabled: true,
        queries: Vec::new(),
    };

    let mut tags = Vec::new();

    // Word tags from block read
    let words: &[(&str, &str, u16, bool, &str, &str)] = &[
        ("wt.di_pack", "WT.DI_PACK", 100, false, "", "DI pack I0–I15"),
        ("wt.do_pack", "WT.DO_PACK", 101, false, "", "DO pack Q0–Q15"),
        ("wt.m_lo", "WT.M_LO", 102, false, "", "Markers M0–M15"),
        ("wt.m_hi", "WT.M_HI", 103, false, "", "Markers M16–M31"),
        (
            "wt.level_cm",
            "WT.LEVEL_cm",
            104,
            true,
            "cm",
            "Process level 0…1000",
        ),
        (
            "wt.k_x100",
            "WT.K_x100",
            105,
            true,
            "×100",
            "Inflow factor K×100",
        ),
        (
            "wt.fill_step",
            "WT.FILL_STEP",
            106,
            true,
            "",
            "Inflow units / tick",
        ),
        (
            "wt.pump_step",
            "WT.PUMP_STEP",
            107,
            false,
            "",
            "Single pump capacity / tick",
        ),
        (
            "wt.sp_stop",
            "WT.SP_STOP",
            108,
            true,
            "cm",
            "Stop / reset demand",
        ),
        ("wt.sp_p1_on", "WT.SP_P1_ON", 109, true, "cm", "Start P1"),
        ("wt.sp_p2_on", "WT.SP_P2_ON", 110, true, "cm", "Join P2"),
        ("wt.cap", "WT.CAP", 111, false, "", "Total pump capacity"),
        ("wt.drain", "WT.DRAIN_REG", 112, false, "", "CAP−FILL"),
        ("wt.fill_net", "WT.FILL_NET", 113, false, "", "FILL−CAP"),
        ("wt.p1_hh", "WT.P1_HH", 114, false, "h", "P1 run hours"),
        ("wt.p1_mm", "WT.P1_MM", 115, false, "min", "P1 run minutes"),
        ("wt.p1_ss", "WT.P1_SS", 116, false, "s", "P1 run seconds"),
        ("wt.p2_hh", "WT.P2_HH", 117, false, "h", "P2 run hours"),
        ("wt.p2_mm", "WT.P2_MM", 118, false, "min", "P2 run minutes"),
        ("wt.p2_ss", "WT.P2_SS", 119, false, "s", "P2 run seconds"),
        (
            "wt.p1_starts",
            "WT.P1_STARTS",
            120,
            false,
            "",
            "P1 start count",
        ),
        (
            "wt.p2_starts",
            "WT.P2_STARTS",
            121,
            false,
            "",
            "P2 start count",
        ),
    ];

    for (id, name, addr, writable, unit, desc) in words {
        tags.push(TagDefinition {
            id: (*id).into(),
            name: (*name).into(),
            device_id: device.id.clone(),
            data_type: TagDataType::U16,
            binding: TagBinding {
                address: *addr,
                bit: None,
                table: ModbusTable::Holding,
                writable: *writable,
                bit_write_mode: BitWriteMode::MaskWrite,
                single_writer: false,
                verify_readback: true,
                word_order: WordOrder::HighWordFirst,
                min_security_level: if *writable { 100 } else { 0 },
            },
            unit: (*unit).into(),
            description: (*desc).into(),
            scale: 1.0,
            offset: 0.0,
            decimals: 0,
            initial_value: None,
        });
    }

    let bits: &[(&str, &str, u16, u8, &str)] = &[
        ("wt.sim_en", "WT.SIM_EN", 100, 0, "Simulation enable"),
        ("wt.p1_fault", "WT.P1_FAULT", 100, 3, "Pump 1 fault"),
        ("wt.p2_fault", "WT.P2_FAULT", 100, 4, "Pump 2 fault"),
        ("wt.p1_lock", "WT.P1_LOCK", 100, 5, "Pump 1 lock"),
        ("wt.p2_lock", "WT.P2_LOCK", 100, 6, "Pump 2 lock"),
        ("wt.man_p1", "WT.MAN_P1", 100, 8, "Manual force P1"),
        ("wt.man_p2", "WT.MAN_P2", 100, 9, "Manual force P2"),
        ("wt.p1_run", "WT.P1_RUN", 101, 0, "Pump 1 run"),
        ("wt.p2_run", "WT.P2_RUN", 101, 1, "Pump 2 run"),
        ("wt.alm_hi", "WT.ALM_HI", 101, 2, "High level alarm"),
        ("wt.alm_fault", "WT.ALM_FAULT", 101, 3, "Any pump fault"),
        (
            "wt.alm_fail",
            "WT.ALM_FAIL",
            101,
            4,
            "Station fail — no pump",
        ),
        ("wt.demand", "WT.DEMAND", 102, 2, "Pump-out demand"),
        ("wt.join_p2", "WT.JOIN_P2", 102, 3, "Need lag pump"),
        ("wt.p1_ok", "WT.P1_OK", 102, 4, "P1 available"),
        ("wt.p2_ok", "WT.P2_OK", 102, 5, "P2 available"),
        ("wt.drain_regime", "WT.DRAIN", 103, 9, "Draining regime"),
    ];

    for (id, name, addr, bit, desc) in bits {
        tags.push(TagDefinition {
            id: (*id).into(),
            name: (*name).into(),
            device_id: device.id.clone(),
            data_type: TagDataType::Bool,
            binding: TagBinding {
                address: *addr,
                bit: Some(*bit),
                table: ModbusTable::Holding,
                writable: false,
                bit_write_mode: BitWriteMode::MaskWrite,
                single_writer: false,
                verify_readback: true,
                word_order: WordOrder::HighWordFirst,
                min_security_level: 0,
            },
            unit: String::new(),
            description: (*desc).into(),
            scale: 1.0,
            offset: 0.0,
            decimals: 0,
            initial_value: None,
        });
    }

    let form = water_tank_form();
    let pump_faceplate = pump_faceplate_form();

    let alarms = vec![
        AlarmDefinition {
            id: "alm_hi".into(),
            name: "High Level".into(),
            tag_id: "wt.alm_hi".into(),
            group_id: "water_tank".into(),
            priority: AlarmPriority::High,
            when_true: true,
            hi_limit: None,
            lo_limit: None,
            deadband: 0.0,
            on_delay_ms: 0,
            off_delay_ms: 0,
            latching: false,
            message: "Wet-well high level (ALM_HI)".into(),
        },
        AlarmDefinition {
            id: "alm_fault".into(),
            name: "Pump Fault".into(),
            tag_id: "wt.alm_fault".into(),
            group_id: "water_tank".into(),
            priority: AlarmPriority::High,
            when_true: true,
            hi_limit: None,
            lo_limit: None,
            deadband: 0.0,
            on_delay_ms: 0,
            off_delay_ms: 0,
            latching: true,
            message: "Pump fault present (ALM_FAULT)".into(),
        },
        AlarmDefinition {
            id: "alm_fail".into(),
            name: "Station Fail".into(),
            tag_id: "wt.alm_fail".into(),
            group_id: "water_tank".into(),
            priority: AlarmPriority::Critical,
            when_true: true,
            hi_limit: None,
            lo_limit: None,
            deadband: 0.0,
            on_delay_ms: 0,
            off_delay_ms: 0,
            latching: true,
            message: "Demand with no available pump (ALM_FAIL)".into(),
        },
        AlarmDefinition {
            id: "alm_level_hi".into(),
            name: "Level Hi Limit".into(),
            tag_id: "wt.level_cm".into(),
            group_id: "water_tank".into(),
            priority: AlarmPriority::Medium,
            when_true: false,
            hi_limit: Some(850.0),
            lo_limit: None,
            deadband: 10.0,
            on_delay_ms: 500,
            off_delay_ms: 500,
            latching: false,
            message: "Level above 850 cm".into(),
        },
    ];

    let form_id = form.id.clone();
    let tree = default_water_tank_tree(
        &form_id,
        &form.name,
        &pump_faceplate.id,
        &pump_faceplate.name,
    );

    let mut project = ScadaProject {
        schema_version: SCHEMA_VERSION,
        id: "water_tank_dual_pump".into(),
        name: "Water Tank Dual-Pump Station".into(),
        description: "SCADA visualization for PLC LAD SIM Water Tank (Modbus HR100–121). Military/medical-grade engineering practices; lab use only.".into(),
        devices: vec![device],
        tags,
        forms: vec![form, pump_faceplate],
        alarms,
        alarm_groups: vec![AlarmGroupDefinition {
            id: "water_tank".into(),
            name: "Water Tank Station".into(),
            parent_id: None,
            object_id: Some("WT-001".into()),
            description: "Built-in dual-pump water tank".into(),
        }],
        design_system: None,
        component_templates: Vec::new(),
        users: Vec::new(),
        session_config: SessionConfig::default(),
        tree,
        content_hash: String::new(),
    };
    project.recompute_hash();
    project
}

fn default_water_tank_tree(
    form_id: &str,
    form_name: &str,
    pf_id: &str,
    pf_name: &str,
) -> Vec<ProjectNode> {
    vec![
        ProjectNode {
            id: "fld_screens".into(),
            parent_id: None,
            kind: ProjectNodeKind::Folder,
            name: "Screens".into(),
            order: 0,
            ref_id: None,
            content: None,
            language: None,
            collapsed: Some(false),
        },
        ProjectNode {
            id: "scr_main".into(),
            parent_id: Some("fld_screens".into()),
            kind: ProjectNodeKind::Screen,
            name: form_name.into(),
            order: 0,
            ref_id: Some(form_id.into()),
            content: None,
            language: None,
            collapsed: None,
        },
        ProjectNode {
            id: "scr_pump_faceplate".into(),
            parent_id: Some("fld_screens".into()),
            kind: ProjectNodeKind::Screen,
            name: pf_name.into(),
            order: 1,
            ref_id: Some(pf_id.into()),
            content: None,
            language: None,
            collapsed: None,
        },
        ProjectNode {
            id: "fld_scripts".into(),
            parent_id: None,
            kind: ProjectNodeKind::Folder,
            name: "Scripts".into(),
            order: 1,
            ref_id: None,
            content: None,
            language: None,
            collapsed: Some(false),
        },
        ProjectNode {
            id: "scr_demo_click".into(),
            parent_id: Some("fld_scripts".into()),
            kind: ProjectNodeKind::Script,
            name: "OnButtonClick.action".into(),
            order: 0,
            ref_id: None,
            content: Some("log \"HMI click\"".into()),
            language: Some("proscada-actions".into()),
            collapsed: None,
        },
        ProjectNode {
            id: "fld_docs".into(),
            parent_id: None,
            kind: ProjectNodeKind::Folder,
            name: "Documents".into(),
            order: 2,
            ref_id: None,
            content: None,
            language: None,
            collapsed: Some(false),
        },
        ProjectNode {
            id: "md_readme".into(),
            parent_id: Some("fld_docs".into()),
            kind: ProjectNodeKind::Markdown,
            name: "README.md".into(),
            order: 0,
            ref_id: None,
            content: Some(
                "# Water Tank Dual-Pump\n\nOperator notes and commissioning checklist.\n".into(),
            ),
            language: None,
            collapsed: None,
        },
        ProjectNode {
            id: "note_ops".into(),
            parent_id: Some("fld_docs".into()),
            kind: ProjectNodeKind::Note,
            name: "OperatorNotes.txt".into(),
            order: 1,
            ref_id: None,
            content: Some("Shift handover notes…\n".into()),
            language: None,
            collapsed: None,
        },
        ProjectNode {
            id: "var_all".into(),
            parent_id: None,
            kind: ProjectNodeKind::Variables,
            name: "Variables".into(),
            order: 3,
            ref_id: None,
            content: Some("".into()),
            language: None,
            collapsed: None,
        },
    ]
}

fn water_tank_form() -> FormDef {
    use serde_json::json;

    let mut widgets = Vec::new();
    let mut z = 0;

    // Header Badges
    widgets.push(w(
        "status_badges",
        "data_status",
        24.0,
        16.0,
        300.0,
        40.0,
        z,
        Some("wt.sim_en"),
        None,
        json!({
            "environment": "LIVE", "staleAfterMs": 2000
        }),
    ));
    z += 1;

    // Deconstructed Atomic Group: METRICS OVERVIEW (group_id: grp_metrics)
    let grp_m = Some("grp_metrics");
    widgets.push(w("m_bg", "shape", 340.0, 16.0, 680.0, 74.0, z, None, grp_m, json!({
        "title": "METRICS OVERVIEW", "borderRadius": 10, "bgColor": "#FFFFFF", "borderColor": "#E5E7EB", "borderWidth": 1
    })));
    z += 1;

    widgets.push(w("m_level", "numeric", 350.0, 38.0, 150.0, 46.0, z, Some("wt.level_cm"), grp_m, json!({
        "title": "TANK LEVEL", "unit": "cm", "decimals": 0, "textColor": "#0284C7", "fontSize": 16, "borderRadius": 6, "bgColor": "#F9FAFB"
    })));
    z += 1;

    widgets.push(w("m_p1", "state_indicator", 510.0, 38.0, 150.0, 46.0, z, Some("wt.p1_run"), grp_m, json!({
        "title": "PUMP 1", "variant": "word", "states": "0|STOPPED|#64748b|○\n1|RUNNING|#16a34a|●"
    })));
    z += 1;

    widgets.push(w("m_p2", "state_indicator", 670.0, 38.0, 150.0, 46.0, z, Some("wt.p2_run"), grp_m, json!({
        "title": "PUMP 2", "variant": "word", "states": "0|STOPPED|#64748b|○\n1|RUNNING|#16a34a|●"
    })));
    z += 1;

    widgets.push(w("m_inflow", "numeric", 830.0, 38.0, 170.0, 46.0, z, Some("wt.k_x100"), grp_m, json!({
        "title": "INFLOW K", "unit": "×100", "decimals": 0, "textColor": "#1F2937", "fontSize": 16, "borderRadius": 6, "bgColor": "#F9FAFB"
    })));
    z += 1;

    // Main Synoptic Elements
    widgets.push(w(
        "iso_tank_1",
        "iso_water_tank",
        24.0,
        100.0,
        360.0,
        300.0,
        z,
        Some("wt.level_cm"),
        None,
        json!({
            "label": "Water Tank Cutaway"
        }),
    ));
    z += 1;

    widgets.push(w(
        "iso_pump_1",
        "iso_pump",
        400.0,
        100.0,
        160.0,
        140.0,
        z,
        Some("wt.p1_run"),
        None,
        json!({
            "pumpName": "PUMP 1 (Lead)"
        }),
    ));
    z += 1;

    widgets.push(w(
        "iso_pump_2",
        "iso_pump",
        400.0,
        250.0,
        160.0,
        140.0,
        z,
        Some("wt.p2_run"),
        None,
        json!({
            "pumpName": "PUMP 2 (Lag)"
        }),
    ));
    z += 1;

    widgets.push(w(
        "inlet_pipe",
        "iso_pipe",
        400.0,
        400.0,
        310.0,
        70.0,
        z,
        Some("wt.sim_en"),
        None,
        json!({
            "label": "Inlet Pipe Stream"
        }),
    ));
    z += 1;

    widgets.push(w(
        "terrain_cut",
        "iso_terrain",
        24.0,
        410.0,
        360.0,
        170.0,
        z,
        None,
        None,
        json!({
            "label": "Soil & Grass Cutaway"
        }),
    ));
    z += 1;

    // Deconstructed Atomic Group: SETPOINT CONTROLLER (group_id: grp_setpoints)
    let grp_sp = Some("grp_setpoints");
    widgets.push(w("sp_bg", "shape", 730.0, 100.0, 290.0, 230.0, z, None, grp_sp, json!({
        "title": "OPERATING LEVELS SETPOINTS", "borderRadius": 10, "bgColor": "#FFFFFF", "borderColor": "#E5E7EB", "borderWidth": 1
    })));
    z += 1;

    widgets.push(w(
        "sp_stop_step",
        "numeric_input",
        740.0,
        130.0,
        270.0,
        44.0,
        z,
        Some("wt.sp_stop"),
        grp_sp,
        json!({
            "title": "SP_STOP", "step": 50, "unit": "cm", "labelColor": "#16A34A"
        }),
    ));
    z += 1;

    widgets.push(w(
        "sp_p1_step",
        "numeric_input",
        740.0,
        178.0,
        270.0,
        44.0,
        z,
        Some("wt.sp_p1_on"),
        grp_sp,
        json!({
            "title": "SP_P1_ON", "step": 50, "unit": "cm", "labelColor": "#EAB308"
        }),
    ));
    z += 1;

    widgets.push(w(
        "sp_p2_step",
        "numeric_input",
        740.0,
        226.0,
        270.0,
        44.0,
        z,
        Some("wt.sp_p2_on"),
        grp_sp,
        json!({
            "title": "SP_P2_ON", "step": 50, "unit": "cm", "labelColor": "#DC2626"
        }),
    ));
    z += 1;

    widgets.push(w(
        "sp_apply_btn",
        "command_button",
        740.0,
        276.0,
        270.0,
        40.0,
        z,
        Some("wt.sp_stop"),
        grp_sp,
        json!({
            "label": "Set SP_STOP to 600", "mode": "value", "writeValue": 600,
            "confirm": true, "disabledWhenBad": true
        }),
    ));
    z += 1;

    // Inflow, Process & Alarms Panels
    widgets.push(w(
        "inflow_ctrl",
        "inflow_control",
        730.0,
        340.0,
        290.0,
        130.0,
        z,
        Some("wt.k_x100"),
        None,
        json!({
            "title": "Inflow Factor K"
        }),
    ));
    z += 1;

    widgets.push(w(
        "proc_ctrl",
        "process_control",
        400.0,
        480.0,
        310.0,
        130.0,
        z,
        Some("wt.sim_en"),
        None,
        json!({
            "title": "Process Controls"
        }),
    ));
    z += 1;

    widgets.push(w(
        "alarms_ctrl",
        "alarm_panel",
        730.0,
        480.0,
        290.0,
        200.0,
        z,
        None,
        None,
        json!({
            "title": "Active Alarms"
        }),
    ));

    FormDef {
        id: "main".into(),
        name: "Main_Synoptic".into(),
        width: 1040.0,
        height: 700.0,
        background: "#F4F5F7".into(),
        grid: 8,
        min_level: None,
        unauthorized_behavior: None,
        widgets,
    }
}

fn pump_faceplate_form() -> FormDef {
    use serde_json::json;

    let mut widgets = Vec::new();
    let mut z = 0;

    // 1. Label widget
    widgets.push(w(
        "pf_label",
        "label",
        12.0,
        8.0,
        160.0,
        24.0,
        z,
        None,
        None,
        json!({
            "text": "PUMP MODULE",
            "fontSize": 13,
            "fontWeight": "bold",
            "textColor": "#1E293B"
        }),
    ));
    z += 1;

    // 2. Image widget (Vector Pump Graphic)
    widgets.push(w(
        "pf_image",
        "image",
        12.0,
        36.0,
        90.0,
        85.0,
        z,
        None,
        None,
        json!({
            "src": "",
            "fit": "contain",
            "alt": "Pump Graphic"
        }),
    ));
    z += 1;

    // 3. Bool LED Indicator (Status)
    widgets.push(w(
        "pf_led",
        "state_indicator",
        110.0,
        48.0,
        118.0,
        46.0,
        z,
        Some("run"),
        None,
        json!({
            "title": "STATUS",
            "variant": "word",
            "states": "0|STOPPED|#64748b|○\n1|RUNNING|#16a34a|●"
        }),
    ));
    z += 1;

    // 4. Connection Line / Arrow
    widgets.push(w(
        "pf_line",
        "line",
        12.0,
        134.0,
        216.0,
        24.0,
        z,
        None,
        None,
        json!({
            "x1": 5, "y1": 50, "x2": 95, "y2": 50,
            "stroke": "#2563EB", "strokeWidth": 3.0,
            "lineStyle": "solid", "endCap": "arrow"
        }),
    ));

    FormDef {
        id: "pump_faceplate".into(),
        name: "Pump_Faceplate_Master".into(),
        width: 240.0,
        height: 170.0,
        background: "#FFFFFF".into(),
        grid: 8,
        min_level: None,
        unauthorized_behavior: None,
        widgets,
    }
}

// The compact fixture factory mirrors WidgetDef's serialized fields at each call site.
#[allow(clippy::too_many_arguments)]
fn w(
    id: &str,
    widget_type: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    z: i32,
    tag_id: Option<&str>,
    group_id: Option<&str>,
    config: serde_json::Value,
) -> WidgetDef {
    WidgetDef {
        id: id.into(),
        widget_type: widget_type.into(),
        x,
        y,
        w: width,
        h: height,
        z,
        tag_id: tag_id.map(|s| s.into()),
        group_id: group_id.map(|s| s.into()),
        locked: None,
        min_level: None,
        unauthorized_behavior: None,
        config,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_tank_hash_stable() {
        let p = water_tank_project();
        assert!(p.verify_hash());
        assert!(!p.tags.is_empty());
        assert!(!p.forms[0].widgets.is_empty());
        assert_eq!(p.devices[0].port, 5020);
    }

    #[test]
    fn validation_rejects_runtime_configuration_that_is_not_implemented() {
        let mut project = water_tank_project();
        let mut second = project.devices[0].clone();
        second.id = "second-plc".into();
        project.devices.push(second);
        assert!(project
            .validate()
            .expect_err("two active devices")
            .contains("exactly one active device"));

        let mut project = water_tank_project();
        project.devices[0].queries.push(ModbusQueryConfig {
            id: "fast".into(),
            name: "Fast".into(),
            table: ModbusTable::Holding,
            start_address: 100,
            count: 4,
            poll_ms: Some(100),
            enabled: true,
        });
        assert!(project
            .validate()
            .expect_err("query schedules")
            .contains("does not support query schedules"));
    }

    #[test]
    fn validation_rejects_out_of_range_memory_and_unknown_system_tags() {
        let mut project = water_tank_project();
        let mut memory = project.tags[0].clone();
        memory.id = "memory.u16".into();
        memory.device_id = "SYS_INTERNAL".into();
        memory.binding.table = ModbusTable::Memory;
        memory.binding.bit = None;
        memory.data_type = TagDataType::U16;
        memory.initial_value = Some("70000".into());
        project.tags.push(memory);
        assert!(project
            .validate()
            .expect_err("out of range memory")
            .contains("out of range"));

        let mut project = water_tank_project();
        let mut system = project.tags[0].clone();
        system.id = "system.arbitrary".into();
        system.device_id = "SYS_INTERNAL".into();
        system.binding.table = ModbusTable::System;
        system.binding.writable = false;
        system.binding.bit = None;
        system.data_type = TagDataType::U16;
        system.initial_value = None;
        project.tags.push(system);
        assert!(project
            .validate()
            .expect_err("unknown system tag")
            .contains("Unknown System tag"));
    }

    #[test]
    fn validation_rejects_hierarchy_cycles_and_executable_content() {
        let mut project = water_tank_project();
        project.alarm_groups.push(AlarmGroupDefinition {
            id: "child".into(),
            name: "Child".into(),
            parent_id: Some("water_tank".into()),
            object_id: None,
            description: String::new(),
        });
        project.alarm_groups[0].parent_id = Some("child".into());
        assert!(project
            .validate()
            .expect_err("alarm group cycle")
            .contains("cycle"));

        let mut project = water_tank_project();
        project.component_templates.push(serde_json::json!({
            "id": "unsafe",
            "action": "javascript:alert(1)"
        }));
        assert!(project
            .validate()
            .expect_err("executable template")
            .contains("executable or remote"));

        let mut project = water_tank_project();
        let script = project
            .tree
            .iter_mut()
            .find(|node| node.kind == ProjectNodeKind::Script)
            .expect("builtin action script");
        script.language = Some("javascript".into());
        script.content = Some("eval('unsafe')".into());
        assert!(project
            .validate()
            .expect_err("arbitrary script")
            .contains("proscada-actions"));
    }

    #[test]
    fn public_water_tank_fixture_is_canonical_and_contains_no_account_realm() {
        let project: ScadaProject = serde_json::from_str(include_str!(
            "../../../public/projects/WaterTank.proscada.json"
        ))
        .expect("deserialize public fixture");
        project.validate().expect("validate public fixture");
        assert!(
            project.verify_hash(),
            "public fixture hash must be canonical"
        );
        assert!(
            project.users.is_empty(),
            "public fixture must not ship accounts"
        );
        assert!(
            project
                .devices
                .iter()
                .all(|device| device.queries.is_empty()),
            "public fixture must not advertise an unsupported query scheduler"
        );
    }
}
