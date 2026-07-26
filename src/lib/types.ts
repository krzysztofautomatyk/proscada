export type Role = "viewer" | "operator" | "engineer" | "administrator";

export type LeftPanelTab =
  | "solution"
  | "toolbox"
  | "objects"
  | "designSystem"
  | "components"
  | "alarms";

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
  bit_write_mode?: "mask_write" | "read_modify_write";
  single_writer?: boolean;
  verify_readback?: boolean;
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
  locked?: boolean;
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
  group_id?: string;
  priority: AlarmPriority;
  when_true: boolean;
  hi_limit?: number | null;
  lo_limit?: number | null;
  deadband?: number;
  on_delay_ms?: number;
  off_delay_ms?: number;
  latching?: boolean;
  message: string;
}

export interface AlarmGroupDefinition {
  id: string;
  name: string;
  parent_id?: string | null;
  object_id?: string | null;
  description?: string;
}

export interface ProjectFontToken {
  id: string;
  name: string;
  family: string;
  fallback: string;
  size: number;
  weight: string;
  lineHeight: number;
}

export interface ProjectStyleClass {
  id: string;
  name: string;
  target: string;
  surface: string;
  text: string;
  accent: string;
  border: string;
}

export type AnimationKind = "none" | "pulse" | "rotate" | "fade" | "slide";

export interface ProjectAnimationPreset {
  id: string;
  name: string;
  kind: AnimationKind;
  durationMs: number;
  easing: string;
}

export interface ProjectDesignSystem {
  version: number;
  fonts: ProjectFontToken[];
  styles: ProjectStyleClass[];
  animations: ProjectAnimationPreset[];
}

export interface ComponentTemplate {
  id: string;
  name: string;
  category: string;
  version: string;
  description: string;
  width: number;
  height: number;
  widgets: WidgetDef[];
  parameter_names: string[];
  alarm_templates?: AlarmDefinition[];
}

/** Solution Explorer node kinds (VS-style project tree). */
export type ProjectNodeKind =
  | "folder"
  | "screen"
  | "variables"
  | "script"
  | "note"
  | "markdown"
  | "image";

export interface ProjectNode {
  id: string;
  /** null = child of project root */
  parent_id: string | null;
  kind: ProjectNodeKind;
  name: string;
  order: number;
  /** For screen — FormDef.id; for variables — optional filter label */
  ref_id?: string | null;
  /** Body for script / note / markdown */
  content?: string;
  /** Script language (v1: javascript only) */
  language?: "javascript";
  collapsed?: boolean;
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
  alarm_groups?: AlarmGroupDefinition[];
  design_system?: ProjectDesignSystem;
  component_templates?: ComponentTemplate[];
  /** Hierarchical Solution Explorer items (folders, screens, scripts, docs). */
  tree?: ProjectNode[];
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
  group_id?: string;
  state: AlarmState;
  source_active?: boolean;
  latched?: boolean;
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
