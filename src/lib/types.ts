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
  {
    type: "label",
    label: "Label",
    category: "Display",
    icon: "T",
    defaultW: 160,
    defaultH: 28,
    defaultConfig: { text: "Label", fontSize: 13, textColor: "#e2e8f0", align: "left" },
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
      unit: "",
      bgColor: "#0f172a",
      textColor: "#e2e8f0",
    },
  },
  {
    type: "lamp",
    label: "Indicator Lamp",
    category: "Indicators",
    icon: "●",
    defaultW: 120,
    defaultH: 44,
    defaultConfig: {
      title: "STATE",
      onColor: "#22c55e",
      offColor: "#334155",
      onLabel: "ON",
      offLabel: "OFF",
      blink: false,
    },
  },
  {
    type: "tank",
    label: "Tank Level",
    category: "Gauges",
    icon: "▣",
    defaultW: 120,
    defaultH: 220,
    defaultConfig: {
      title: "LEVEL",
      min: 0,
      max: 1000,
      unit: "cm",
      fillColor: "#1e90ff",
      warn: 700,
      alarm: 850,
      showValue: true,
      bgColor: "#111827",
    },
  },
  {
    type: "panel",
    label: "Group Panel",
    category: "Graphics",
    icon: "▭",
    defaultW: 240,
    defaultH: 160,
    defaultConfig: { title: "PANEL", bgColor: "#0f172a", borderColor: "#334155" },
  },
  {
    type: "write_button",
    label: "Write Button",
    category: "Controls",
    icon: "▶",
    defaultW: 140,
    defaultH: 36,
    defaultConfig: {
      label: "WRITE",
      confirm: true,
      valueKind: "number",
      writeValue: 0,
      bgColor: "#1d4ed8",
      textColor: "#fff",
    },
  },
  {
    type: "bar",
    label: "Bar Graph",
    category: "Gauges",
    icon: "▬",
    defaultW: 200,
    defaultH: 40,
    defaultConfig: {
      title: "BAR",
      min: 0,
      max: 100,
      fillColor: "#38bdf8",
      bgColor: "#0f172a",
    },
  },
];
