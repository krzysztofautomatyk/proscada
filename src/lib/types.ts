export type Role = "viewer" | "operator" | "engineer" | "administrator";

export type LeftPanelTab =
  | "solution"
  | "toolbox"
  | "objects"
  | "designSystem"
  | "components"
  | "alarms";

export type Quality = "good" | "uncertain" | "bad" | "comm_lost";

export type AlarmPriority = "low" | "medium" | "high" | "critical";

export type AlarmState =
  | "inactive"
  | "active_unacked"
  | "active_acked"
  | "cleared_unacked";

export interface ModbusQueryConfig {
  id: string;
  name: string;
  table: "holding" | "input" | "coil" | "discrete";
  start_address: number;
  count: number;
  poll_ms?: number | null;
  enabled: boolean;
}

export interface DeviceConfig {
  id: string;
  name: string;
  host: string;
  port: number;
  unit_id: number;
  poll_ms: number;
  timeout_ms: number;
  enabled: boolean;
  queries?: ModbusQueryConfig[];
}

export type WordOrder = "high_word_first" | "low_word_first";

export interface TagBinding {
  address: number;
  bit?: number | null;
  table: "holding" | "input" | "coil" | "discrete" | "memory" | "system";
  writable?: boolean;
  bit_write_mode?: "mask_write" | "read_modify_write";
  single_writer?: boolean;
  verify_readback?: boolean;
  string_length?: number;
  /** Register order for data types wider than one register. */
  word_order?: WordOrder;
  /** Backend-enforced minimum security level required to write this tag. */
  min_security_level?: number;
}

export type TagDataType =
  | "bool"
  | "u16"
  | "i16"
  | "u32"
  | "i32"
  | "f32"
  | "u64"
  | "i64"
  | "f64"
  | "string";

export interface TagDefinition {
  id: string;
  name: string;
  device_id: string;
  data_type: TagDataType;
  binding: TagBinding;
  unit: string;
  description: string;
  scale: number;
  offset: number;
  decimals: number;
  initial_value?: string;
  is_system?: boolean;
}

export type UnauthorizedBehavior = "disabled" | "hidden" | "prompt_login";

export interface UserSummary {
  id: string;
  username: string;
  display_name: string;
  security_level: number;
  enabled: boolean;
  has_pin: boolean;
  /** Seeded accounts must replace their default password before they can act. */
  password_change_required?: boolean;
}

export interface UserAccountInput {
  id?: string | null;
  username: string;
  display_name: string;
  password?: string | null;
  pin?: string | null;
  security_level: number;
  enabled: boolean;
}

export interface SessionConfig {
  auto_logout_minutes: number;
  pin_challenge_on_write: boolean;
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
  min_level?: number | null;
  unauthorized_behavior?: UnauthorizedBehavior | null;
  config: Record<string, unknown>;
}

export interface FormDef {
  id: string;
  name: string;
  width: number;
  height: number;
  background: string;
  grid: number;
  min_level?: number | null;
  unauthorized_behavior?: UnauthorizedBehavior | null;
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

export interface ComponentTagSlotMeta {
  slotKey: string;
  name: string;
  comment?: string;
  description?: string;
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
  extracted_tags?: string[];
  tag_slots_meta?: ComponentTagSlotMeta[];
}

/** Solution Explorer node kinds (VS-style project tree). */
export type ProjectNodeKind =
  | "folder"
  | "screen"
  | "variables"
  | "script"
  | "note"
  | "markdown"
  | "image"
  | "style"
  | "components_folder"
  | "component";

export interface AppSettings {
  autosaveEnabled: boolean;
  autosaveIntervalMinutes: number;
  autosaveOnlyIfNoError: boolean;
  lastAutosaveTs: string | null;
  lastAutosaveStatus: "ok" | "skipped_errors" | "error" | null;
  showStartWindowOnStart?: boolean;
}

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
  /**
   * Script node dialect. `proscada-actions` is the deterministic action language
   * executed by `scriptRuntime`; `javascript` only appears in projects created
   * before that change and no longer runs.
   */
  language?: "proscada-actions" | "javascript";
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
  users?: UserSummary[];
  session_config?: SessionConfig;
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
  string_value?: string;
  quality: Quality;
  ts: string;
  age_ms: number;
  raw: number;
}

export interface WriteReceipt {
  tag_id: string;
  requested_value: number;
  observed_value: number;
  raw_readback: number;
  protocol: string;
  verify_readback: boolean;
  matches: boolean;
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
  /** True when the source tag is not Good, so the state shown is stale. */
  evaluation_suspended?: boolean;
  suspended_reason?: string | null;
  suspended_since?: string | null;
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
  current_user?: UserSummary | null;
  security_level: number;
  project_name?: string | null;
  mode: string;
  /** True when at least one alarm is not evaluated against live data. */
  alarms_suspended?: boolean;
  password_change_required?: boolean;
  /** True only while the loaded project has no user accounts yet. */
  requires_bootstrap?: boolean;
  /** False means the audit hash chain failed verification and process writes are blocked. */
  audit_chain_ok?: boolean;
  /** False means audit entries are not durably persisted and process writes are blocked. */
  audit_persisted?: boolean;
  audit_last_error?: string | null;
  /** False means alarm lifecycle/ACK state is not durably persisted. */
  alarm_state_persisted?: boolean;
  alarm_state_last_error?: string | null;
  /** False means the installation account realm is absent, corrupt or not durable. */
  user_realm_persisted?: boolean;
  user_realm_last_error?: string | null;
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

export interface AuditStatus {
  chain_ok: boolean;
  sink_path?: string | null;
  persisted: boolean;
  last_error?: string | null;
  in_memory: number;
  appended: number;
}
