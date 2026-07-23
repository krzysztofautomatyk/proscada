//! Project schema — devices, tags, forms, widgets, alarms.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SCHEMA_VERSION: u32 = 1;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModbusTable {
    Holding,
    Input,
    Coil,
    Discrete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagDataType {
    Bool,
    U16,
    I16,
    F32,
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
    pub priority: AlarmPriority,
    /// For bool tags: alarm when true. For numeric: use limit.
    #[serde(default)]
    pub when_true: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hi_limit: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lo_limit: Option<f64>,
    #[serde(default)]
    pub message: String,
}

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
    #[serde(default)]
    pub content_hash: String,
}

impl ScadaProject {
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
    };

    let mut tags = Vec::new();

    // Word tags from block read
    let words: &[(&str, &str, u16, bool, &str, &str)] = &[
        ("wt.di_pack", "WT.DI_PACK", 100, false, "", "DI pack I0–I15"),
        ("wt.do_pack", "WT.DO_PACK", 101, false, "", "DO pack Q0–Q15"),
        ("wt.m_lo", "WT.M_LO", 102, false, "", "Markers M0–M15"),
        ("wt.m_hi", "WT.M_HI", 103, false, "", "Markers M16–M31"),
        ("wt.level_cm", "WT.LEVEL_cm", 104, true, "cm", "Process level 0…1000"),
        ("wt.k_x100", "WT.K_x100", 105, true, "×100", "Inflow factor K×100"),
        ("wt.fill_step", "WT.FILL_STEP", 106, true, "", "Inflow units / tick"),
        ("wt.pump_step", "WT.PUMP_STEP", 107, false, "", "Single pump capacity / tick"),
        ("wt.sp_stop", "WT.SP_STOP", 108, true, "cm", "Stop / reset demand"),
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
        ("wt.p1_starts", "WT.P1_STARTS", 120, false, "", "P1 start count"),
        ("wt.p2_starts", "WT.P2_STARTS", 121, false, "", "P2 start count"),
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
            },
            unit: (*unit).into(),
            description: (*desc).into(),
            scale: 1.0,
            offset: 0.0,
            decimals: 0,
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
        ("wt.alm_fail", "WT.ALM_FAIL", 101, 4, "Station fail — no pump"),
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
            },
            unit: String::new(),
            description: (*desc).into(),
            scale: 1.0,
            offset: 0.0,
            decimals: 0,
        });
    }

    let form = water_tank_form();

    let alarms = vec![
        AlarmDefinition {
            id: "alm_hi".into(),
            name: "High Level".into(),
            tag_id: "wt.alm_hi".into(),
            priority: AlarmPriority::High,
            when_true: true,
            hi_limit: None,
            lo_limit: None,
            message: "Wet-well high level (ALM_HI)".into(),
        },
        AlarmDefinition {
            id: "alm_fault".into(),
            name: "Pump Fault".into(),
            tag_id: "wt.alm_fault".into(),
            priority: AlarmPriority::High,
            when_true: true,
            hi_limit: None,
            lo_limit: None,
            message: "Pump fault present (ALM_FAULT)".into(),
        },
        AlarmDefinition {
            id: "alm_fail".into(),
            name: "Station Fail".into(),
            tag_id: "wt.alm_fail".into(),
            priority: AlarmPriority::Critical,
            when_true: true,
            hi_limit: None,
            lo_limit: None,
            message: "Demand with no available pump (ALM_FAIL)".into(),
        },
        AlarmDefinition {
            id: "alm_level_hi".into(),
            name: "Level Hi Limit".into(),
            tag_id: "wt.level_cm".into(),
            priority: AlarmPriority::Medium,
            when_true: false,
            hi_limit: Some(850.0),
            lo_limit: None,
            message: "Level above 850 cm".into(),
        },
    ];

    let mut project = ScadaProject {
        schema_version: SCHEMA_VERSION,
        id: "water_tank_dual_pump".into(),
        name: "Water Tank Dual-Pump Station".into(),
        description: "SCADA visualization for PLC LAD SIM Water Tank (Modbus HR100–121). Military/medical-grade engineering practices; lab use only.".into(),
        devices: vec![device],
        tags,
        forms: vec![form],
        alarms,
        content_hash: String::new(),
    };
    project.recompute_hash();
    project
}

fn water_tank_form() -> FormDef {
    use serde_json::json;

    let mut widgets = Vec::new();
    let mut z = 0;

    // Header Badges
    widgets.push(w("status_badges", "status_badge", 24.0, 16.0, 300.0, 40.0, z, None, None, json!({
        "simEn": true, "frozen": false
    })));
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

    widgets.push(w("m_p1", "bool_display", 510.0, 38.0, 150.0, 46.0, z, Some("wt.p1_run"), grp_m, json!({
        "label": "PUMP 1", "trueLabel": "RUNNING", "falseLabel": "STOPPED", "trueColor": "#16A34A"
    })));
    z += 1;

    widgets.push(w("m_p2", "bool_display", 670.0, 38.0, 150.0, 46.0, z, Some("wt.p2_run"), grp_m, json!({
        "label": "PUMP 2", "trueLabel": "RUNNING", "falseLabel": "STOPPED", "trueColor": "#16A34A"
    })));
    z += 1;

    widgets.push(w("m_inflow", "numeric", 830.0, 38.0, 170.0, 46.0, z, Some("wt.k_x100"), grp_m, json!({
        "title": "INFLOW K", "unit": "×100", "decimals": 0, "textColor": "#1F2937", "fontSize": 16, "borderRadius": 6, "bgColor": "#F9FAFB"
    })));
    z += 1;

    // Main Synoptic Elements
    widgets.push(w("iso_tank_1", "iso_water_tank", 24.0, 100.0, 360.0, 300.0, z, Some("wt.level_cm"), None, json!({
        "label": "Water Tank Cutaway"
    })));
    z += 1;

    widgets.push(w("iso_pump_1", "iso_pump", 400.0, 100.0, 160.0, 140.0, z, Some("wt.p1_run"), None, json!({
        "pumpName": "PUMP 1 (Lead)"
    })));
    z += 1;

    widgets.push(w("iso_pump_2", "iso_pump", 400.0, 250.0, 160.0, 140.0, z, Some("wt.p2_run"), None, json!({
        "pumpName": "PUMP 2 (Lag)"
    })));
    z += 1;

    widgets.push(w("inlet_pipe", "iso_pipe", 400.0, 400.0, 310.0, 70.0, z, Some("wt.sim_en"), None, json!({
        "label": "Inlet Pipe Stream"
    })));
    z += 1;

    widgets.push(w("terrain_cut", "iso_terrain", 24.0, 410.0, 360.0, 170.0, z, None, None, json!({
        "label": "Soil & Grass Cutaway"
    })));
    z += 1;

    // Deconstructed Atomic Group: SETPOINT CONTROLLER (group_id: grp_setpoints)
    let grp_sp = Some("grp_setpoints");
    widgets.push(w("sp_bg", "shape", 730.0, 100.0, 290.0, 230.0, z, None, grp_sp, json!({
        "title": "OPERATING LEVELS SETPOINTS", "borderRadius": 10, "bgColor": "#FFFFFF", "borderColor": "#E5E7EB", "borderWidth": 1
    })));
    z += 1;

    widgets.push(w("sp_stop_step", "numeric_input", 740.0, 130.0, 270.0, 44.0, z, Some("wt.sp_stop"), grp_sp, json!({
        "title": "SP_STOP", "step": 50, "unit": "cm", "labelColor": "#16A34A"
    })));
    z += 1;

    widgets.push(w("sp_p1_step", "numeric_input", 740.0, 178.0, 270.0, 44.0, z, Some("wt.sp_p1_on"), grp_sp, json!({
        "title": "SP_P1_ON", "step": 50, "unit": "cm", "labelColor": "#EAB308"
    })));
    z += 1;

    widgets.push(w("sp_p2_step", "numeric_input", 740.0, 226.0, 270.0, 44.0, z, Some("wt.sp_p2_on"), grp_sp, json!({
        "title": "SP_P2_ON", "step": 50, "unit": "cm", "labelColor": "#DC2626"
    })));
    z += 1;

    widgets.push(w("sp_apply_btn", "write_button", 740.0, 276.0, 270.0, 40.0, z, Some("wt.sp_stop"), grp_sp, json!({
        "label": "Apply setpoints", "bgColor": "#1F2937", "textColor": "#FFFFFF"
    })));
    z += 1;

    // Inflow, Process & Alarms Panels
    widgets.push(w("inflow_ctrl", "inflow_control", 730.0, 340.0, 290.0, 130.0, z, Some("wt.k_x100"), None, json!({
        "title": "Inflow Factor K"
    })));
    z += 1;

    widgets.push(w("proc_ctrl", "process_control", 400.0, 480.0, 310.0, 130.0, z, Some("wt.sim_en"), None, json!({
        "title": "Process Controls"
    })));
    z += 1;

    widgets.push(w("alarms_ctrl", "alarm_panel", 730.0, 480.0, 290.0, 200.0, z, None, None, json!({
        "title": "Active Alarms"
    })));

    FormDef {
        id: "main".into(),
        name: "Main_Synoptic".into(),
        width: 1040.0,
        height: 700.0,
        background: "#F4F5F7".into(),
        grid: 8,
        widgets,
    }
}

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
}
