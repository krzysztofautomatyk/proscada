export type Role = "viewer" | "operator" | "engineer" | "administrator";

export type Quality = "good" | "uncertain" | "bad";

export type AlarmPriority = "low" | "medium" | "high" | "critical";

export type AlarmState =
  | "inactive"
  | "active_unacked"
  | "active_acked"
  | "cleared_unacked";

export interface DeviceConfig {
  id: string;
  name: string;
  host: string;
  port: number;
  unit_id: number;
  poll_ms: number;
  timeout_ms: number;
  enabled: boolean;
}

export interface TagBinding {
  address: number;
  bit?: number | null;
  table: "holding" | "input" | "coil" | "discrete";
  writable?: boolean;
}

export interface TagDefinition {
  id: string;
  name: string;
  device_id: string;
  data_type: "bool" | "u16" | "i16" | "f32";
  binding: TagBinding;
  unit: string;
  description: string;
  scale: number;
  offset: number;
  decimals: number;
}

export interface WidgetDef {
  id: string;
  widget_type: string;
  x: number;
  y: number;
  w: number;
  h: number;
  z: number;
  tag_id?: string | null;
  group_id?: string | null;
  config: Record<string, unknown>;
}

export interface FormDef {
  id: string;
  name: string;
  width: number;
  height: number;
  background: string;
  grid: number;
  widgets: WidgetDef[];
}

export interface AlarmDefinition {
  id: string;
  name: string;
  tag_id: string;
  priority: AlarmPriority;
  when_true: boolean;
  hi_limit?: number | null;
  lo_limit?: number | null;
  message: string;
}

export interface ScadaProject {
  schema_version: number;
  id: string;
  name: string;
  description: string;
  devices: DeviceConfig[];
  tags: TagDefinition[];
  forms: FormDef[];
  alarms: AlarmDefinition[];
  content_hash: string;
}

export interface TagValue {
  tag_id: string;
  value: number;
  bool_value: boolean;
  quality: Quality;
  ts: string;
  age_ms: number;
  raw: number;
}

export interface AlarmInstance {
  def_id: string;
  name: string;
  message: string;
  priority: AlarmPriority;
  state: AlarmState;
  active_since?: string | null;
  last_change: string;
}

export interface EngineSnapshot {
  connected: boolean;
  device_id?: string | null;
  last_error?: string | null;
  poll_count: number;
  last_poll_ms: number;
  tags: TagValue[];
  alarms: AlarmInstance[];
  role: Role;
  actor: string;
  project_name?: string | null;
  mode: string;
}

export interface AuditEntry {
  id: string;
  ts: string;
  actor: string;
  role: string;
  action: string;
  detail: string;
  prev_hash: string;
  hash: string;
}

export interface WidgetCatalogItem {
  type: string;
  label: string;
  category: string;
  icon: string;
  defaultW: number;
  defaultH: number;
  defaultConfig: Record<string, unknown>;
}

export const WIDGET_CATALOG: WidgetCatalogItem[] = [
  // Display & Text
  {
    type: "label",
    label: "Label",
    category: "Display",
    icon: "T",
    defaultW: 160,
    defaultH: 28,
    defaultConfig: { text: "Label", fontSize: 13, textColor: "#1f2937", align: "left" },
  },
  {
    type: "numeric",
    label: "Numeric Display",
    category: "Display",
    icon: "#",
    defaultW: 160,
    defaultH: 56,
    defaultConfig: {
      title: "Value",
      decimals: 0,
      unit: "cm",
      bgColor: "#ffffff",
      textColor: "#1f2937",
    },
  },
  {
    type: "status_badge",
    label: "Status Badges",
    category: "Display",
    icon: "🏷️",
    defaultW: 240,
    defaultH: 36,
    defaultConfig: { simEn: true, frozen: false },
  },
  {
    type: "shape",
    label: "Shape (Rounded Box)",
    category: "Display",
    icon: "▢",
    defaultW: 180,
    defaultH: 120,
    defaultConfig: {
      title: "",
      borderRadius: 10,
      bgColor: "#ffffff",
      borderColor: "#e5e7eb",
      borderWidth: 1,
      borderStyle: "solid",
    },
  },
  {
    type: "bool_display",
    label: "Bool Display",
    category: "Display",
    icon: "🔘",
    defaultW: 160,
    defaultH: 44,
    defaultConfig: {
      label: "BOOL STATUS",
      trueLabel: "ON",
      falseLabel: "OFF",
      trueColor: "#16a34a",
      falseColor: "#9ca3af",
    },
  },
  {
    type: "panel",
    label: "Group Panel",
    category: "Display",
    icon: "▭",
    defaultW: 240,
    defaultH: 160,
    defaultConfig: { title: "PANEL", bgColor: "#ffffff", borderColor: "#e5e7eb" },
  },

  // Indicators & Alarms
  {
    type: "lamp",
    label: "Indicator Lamp",
    category: "Indicators & Alarms",
    icon: "●",
    defaultW: 120,
    defaultH: 44,
    defaultConfig: {
      title: "STATE",
      onColor: "#16a34a",
      offColor: "#9ca3af",
      onLabel: "ON",
      offLabel: "OFF",
      blink: false,
    },
  },
  {
    type: "alarm_panel",
    label: "Alarm Panel",
    category: "Indicators & Alarms",
    icon: "🔔",
    defaultW: 320,
    defaultH: 200,
    defaultConfig: { title: "Alarms" },
  },

  // Gauges & Synoptic
  {
    type: "tank",
    label: "2D Tank Level",
    category: "Gauges & Synoptic",
    icon: "▣",
    defaultW: 120,
    defaultH: 220,
    defaultConfig: {
      title: "LEVEL",
      min: 0,
      max: 1000,
      unit: "cm",
      fillColor: "#39b7e6",
      warn: 700,
      alarm: 850,
      showValue: true,
      bgColor: "#ffffff",
    },
  },
  {
    type: "bar",
    label: "Bar Graph",
    category: "Gauges & Synoptic",
    icon: "▬",
    defaultW: 200,
    defaultH: 40,
    defaultConfig: {
      title: "BAR",
      min: 0,
      max: 100,
      fillColor: "#16a34a",
      bgColor: "#ffffff",
    },
  },
  {
    type: "iso_water_tank",
    label: "Iso Water Tank",
    category: "Gauges & Synoptic",
    icon: "🛢️",
    defaultW: 360,
    defaultH: 300,
    defaultConfig: { label: "Water Tank Cutaway" },
  },
  {
    type: "iso_pump",
    label: "Iso Pump",
    category: "Gauges & Synoptic",
    icon: "⚙️",
    defaultW: 160,
    defaultH: 140,
    defaultConfig: { pumpName: "PUMP 1" },
  },
  {
    type: "iso_pipe",
    label: "Iso Pipe Segment",
    category: "Gauges & Synoptic",
    icon: "🔍",
    defaultW: 260,
    defaultH: 70,
    defaultConfig: { label: "Inlet Pipe" },
  },
  {
    type: "iso_terrain",
    label: "Iso Terrain Cutaway",
    category: "Gauges & Synoptic",
    icon: "🏞️",
    defaultW: 400,
    defaultH: 180,
    defaultConfig: { label: "Soil & Grass Cutaway" },
  },

  // Process Controls
  {
    type: "write_button",
    label: "Write Button",
    category: "Process Controls",
    icon: "▶",
    defaultW: 140,
    defaultH: 36,
    defaultConfig: {
      label: "WRITE",
      confirm: true,
      valueKind: "number",
      writeValue: 0,
      bgColor: "#1f2937",
      textColor: "#fff",
    },
  },
  {
    type: "numeric_input",
    label: "Numeric Input Stepper",
    category: "Process Controls",
    icon: "🔢",
    defaultW: 200,
    defaultH: 64,
    defaultConfig: {
      title: "SP VALUE",
      step: 10,
      min: 0,
      max: 1000,
      unit: "cm",
      labelColor: "#16a34a",
    },
  },
  {
    type: "setpoint_control",
    label: "Setpoints Controller",
    category: "Process Controls",
    icon: "🎛️",
    defaultW: 280,
    defaultH: 230,
    defaultConfig: { title: "Setpoints" },
  },
  {
    type: "inflow_control",
    label: "Inflow K Controller",
    category: "Process Controls",
    icon: "🚰",
    defaultW: 240,
    defaultH: 120,
    defaultConfig: { title: "Inflow K" },
  },
  {
    type: "process_control",
    label: "Process Freeze Controller",
    category: "Process Controls",
    icon: "⏸️",
    defaultW: 260,
    defaultH: 120,
    defaultConfig: { title: "Process Freeze" },
  },
  {
    type: "metrics_panel",
    label: "Metrics Overview Bar",
    category: "Process Controls",
    icon: "📊",
    defaultW: 600,
    defaultH: 80,
    defaultConfig: { title: "Metrics Overview" },
  },
];

