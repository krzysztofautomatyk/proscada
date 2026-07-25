/**
 * Generic SCADA widget dynamics (ISA-style conditional behavior).
 *
 * Reusable by Label and every future widget type:
 * - blink / marquee-scroll / visibility
 * - condition modes: none | always | bit | register_bit | register_value (eq/gt/lt/neq)
 */
import type { TagValue } from "$lib/types";

/** Condition modes exposed in PropertyGrid (PL/EN labels in UI). */
export type ConditionMode =
  | "none"
  | "always"
  | "tag_true" // bool / non-zero
  | "tag_false"
  | "tag_bit" // register_bit
  | "tag_val_eq" // register_value ==
  | "tag_val_gt"
  | "tag_val_lt"
  | "tag_val_neq";

export const CONDITION_MODE_OPTIONS: { value: ConditionMode; label: string }[] = [
  { value: "none", label: "None (off)" },
  { value: "always", label: "Always (on)" },
  { value: "tag_true", label: "Bit / BOOL = true" },
  { value: "tag_false", label: "Bit / BOOL = false" },
  { value: "tag_bit", label: "Register bit (N)" },
  { value: "tag_val_eq", label: "Register value ==" },
  { value: "tag_val_gt", label: "Register value >" },
  { value: "tag_val_lt", label: "Register value <" },
  { value: "tag_val_neq", label: "Register value !=" },
];

export interface ConditionFields {
  modeKey: string;
  tagKey: string;
  bitKey: string;
  valKey: string;
  defaultMode: ConditionMode;
}

/** Standard field prefixes for blink / scroll / visibility — keep config flat & generic. */
export const BLINK_FIELDS: ConditionFields = {
  modeKey: "blinkMode",
  tagKey: "blinkTagId",
  bitKey: "blinkBit",
  valKey: "blinkVal",
  defaultMode: "none",
};

export const SCROLL_FIELDS: ConditionFields = {
  modeKey: "scrollMode",
  tagKey: "scrollTagId",
  bitKey: "scrollBit",
  valKey: "scrollVal",
  defaultMode: "none",
};

export const VISIBILITY_FIELDS: ConditionFields = {
  modeKey: "visibilityMode",
  tagKey: "visibilityTagId",
  bitKey: "visibilityBit",
  valKey: "visibilityVal",
  defaultMode: "always",
};

export function evaluateCondition(
  mode: ConditionMode | string | undefined,
  tagId: string | null | undefined,
  bitIndex: number | undefined,
  targetVal: number | undefined,
  tagMap: Map<string, TagValue>,
  defaultTagId?: string | null,
): boolean {
  if (!mode || mode === "none") return false;
  if (mode === "always") return true;

  const actualTagId = tagId || defaultTagId;
  if (!actualTagId) return false;

  const tag = tagMap.get(actualTagId);
  if (!tag) return false;

  const numVal = tag.value ?? 0;
  const boolVal = tag.bool_value ?? numVal !== 0;
  const bit = Math.max(0, Math.min(31, bitIndex ?? 0));
  const target = targetVal ?? 0;

  switch (mode) {
    case "tag_true":
      return boolVal || numVal !== 0;
    case "tag_false":
      return !boolVal && numVal === 0;
    case "tag_bit": {
      const intVal = Math.floor(numVal) >>> 0;
      return ((intVal >> bit) & 1) === 1;
    }
    case "tag_val_eq":
      return Math.abs(numVal - target) < 0.0001;
    case "tag_val_gt":
      return numVal > target;
    case "tag_val_lt":
      return numVal < target;
    case "tag_val_neq":
      return Math.abs(numVal - target) >= 0.0001;
    default:
      return false;
  }
}

function readCondition(
  config: Record<string, unknown>,
  fields: ConditionFields,
  tagMap: Map<string, TagValue>,
  mainTagId?: string | null,
): boolean {
  const mode = String(config[fields.modeKey] ?? fields.defaultMode) as ConditionMode;
  const tagId = (config[fields.tagKey] as string) || mainTagId;
  const bitIndex = Number(config[fields.bitKey] ?? 0);
  const targetVal = Number(config[fields.valKey] ?? 0);
  return evaluateCondition(mode, tagId, bitIndex, targetVal, tagMap, mainTagId);
}

export function isWidgetBlinking(
  config: Record<string, unknown>,
  tagMap: Map<string, TagValue>,
  mainTagId?: string | null,
): boolean {
  return readCondition(config, BLINK_FIELDS, tagMap, mainTagId);
}

export function isWidgetScrolling(
  config: Record<string, unknown>,
  tagMap: Map<string, TagValue>,
  mainTagId?: string | null,
): boolean {
  // legacy key scrollTopTagId
  if (config.scrollTopTagId && !config.scrollTagId) {
    config = { ...config, scrollTagId: config.scrollTopTagId };
  }
  return readCondition(config, SCROLL_FIELDS, tagMap, mainTagId);
}

export function isWidgetVisible(
  config: Record<string, unknown>,
  tagMap: Map<string, TagValue>,
  mainTagId?: string | null,
): boolean {
  const mode = String(config.visibilityMode ?? "always") as ConditionMode;
  if (!mode || mode === "always") return true;
  if (mode === "none") return false;
  return readCondition(config, VISIBILITY_FIELDS, tagMap, mainTagId);
}

/** Default dynamics keys for any new widget catalog entry. */
export function defaultDynamicsConfig(): Record<string, unknown> {
  return {
    blinkMode: "none",
    blinkTagId: "",
    blinkBit: 0,
    blinkVal: 1,
    blinkSpeedMs: 600,
    scrollMode: "none",
    scrollTagId: "",
    scrollBit: 0,
    scrollVal: 1,
    scrollSpeedSec: 8,
    scrollDir: "left",
    visibilityMode: "always",
    visibilityTagId: "",
    visibilityBit: 0,
    visibilityVal: 1,
  };
}

/** Label appearance defaults (generic text chrome). */
export function defaultLabelAppearance(): Record<string, unknown> {
  return {
    text: "Label",
    fontFamily: "Segoe UI, system-ui, sans-serif",
    fontSize: 14,
    fontWeight: "normal",
    fontStyle: "normal",
    textColor: "#1f2937",
    bgColor: "transparent",
    borderColor: "transparent",
    borderWidth: 0,
    borderRadius: 0,
    align: "left",
    vAlign: "center",
  };
}

export const FONT_OPTIONS = [
  "Segoe UI, system-ui, sans-serif",
  "Inter, system-ui, sans-serif",
  "Arial, Helvetica, sans-serif",
  "Tahoma, Geneva, sans-serif",
  "Verdana, Geneva, sans-serif",
  "Trebuchet MS, sans-serif",
  "Georgia, serif",
  "Times New Roman, Times, serif",
  "Courier New, Courier, monospace",
  "Consolas, ui-monospace, monospace",
  "Cascadia Code, Consolas, monospace",
  "Impact, Haettenschweiler, sans-serif",
] as const;
