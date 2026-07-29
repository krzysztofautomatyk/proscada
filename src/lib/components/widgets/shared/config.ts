import type { Quality, TagValue, WidgetDef } from "$lib/types";
import type { WidgetConfig } from "./types";

export function configOf(widget: WidgetDef): WidgetConfig {
  return widget.config ?? {};
}

export function readString(config: WidgetConfig, key: string, fallback = ""): string {
  const value = config[key];
  return value === undefined || value === null ? fallback : String(value);
}

export function readNumber(
  config: WidgetConfig,
  key: string,
  fallback = 0,
  min = Number.NEGATIVE_INFINITY,
  max = Number.POSITIVE_INFINITY,
): number {
  const candidate = Number(config[key] ?? fallback);
  const finite = Number.isFinite(candidate) ? candidate : fallback;
  return Math.min(max, Math.max(min, finite));
}

export function readBoolean(config: WidgetConfig, key: string, fallback = false): boolean {
  const value = config[key];
  if (typeof value === "boolean") return value;
  if (typeof value === "string") {
    if (value.toLowerCase() === "true") return true;
    if (value.toLowerCase() === "false") return false;
  }
  if (typeof value === "number") return value !== 0;
  return fallback;
}

export function readStringList(
  config: WidgetConfig,
  key: string,
  fallback: string[] = [],
): string[] {
  const value = config[key];
  if (Array.isArray(value)) {
    return value.map(String).map((item) => item.trim()).filter(Boolean);
  }
  if (typeof value === "string") {
    return value
      .split(/[\n,;]/)
      .map((item) => item.trim())
      .filter(Boolean);
  }
  return fallback;
}

export function readRecordList(
  config: WidgetConfig,
  key: string,
): { rows: Record<string, unknown>[]; error: string | null } {
  const value = config[key];
  if (Array.isArray(value)) {
    return {
      rows: value.filter(
        (item): item is Record<string, unknown> =>
          typeof item === "object" && item !== null && !Array.isArray(item),
      ),
      error: null,
    };
  }
  if (typeof value !== "string" || value.trim() === "") {
    return { rows: [], error: null };
  }
  try {
    const parsed: unknown = JSON.parse(value);
    if (!Array.isArray(parsed)) {
      return { rows: [], error: `${key} must be a JSON array` };
    }
    const rows = parsed.filter(
      (item): item is Record<string, unknown> =>
        typeof item === "object" && item !== null && !Array.isArray(item),
    );
    if (rows.length !== parsed.length) {
      return { rows, error: `${key} contains non-object rows` };
    }
    return { rows, error: null };
  } catch (error) {
    return {
      rows: [],
      error: `${key}: ${error instanceof Error ? error.message : "invalid JSON"}`,
    };
  }
}

export function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

export function tagNumber(tag: TagValue | null | undefined, fallback = 0): number {
  return tag && Number.isFinite(tag.value) ? tag.value : fallback;
}

export function qualityLabel(quality: Quality | undefined): string {
  switch (quality) {
    case "good":
      return "GOOD";
    case "uncertain":
      return "UNCERTAIN";
    case "bad":
      return "BAD";
    default:
      return "NO DATA";
  }
}

export function invokeWrite(
  widget: WidgetDef,
  design: boolean,
  onWrite: ((tagId: string, value: number) => void) | undefined,
  value: number,
): boolean {
  return invokeWriteToTag(widget.tag_id, design, onWrite, value);
}

export function invokeWriteToTag(
  tagId: string | null | undefined,
  design: boolean,
  onWrite: ((tagId: string, value: number) => void) | undefined,
  value: number,
): boolean {
  if (design || !tagId || !onWrite) return false;
  if (typeof value === "number" && !Number.isFinite(value)) return false;
  onWrite(tagId, value);
  return true;
}

export function formatNumericValue(
  valNum: number,
  decimals: number = 0,
  padZeros: number = 0,
): string {
  const safeDecimals = Math.max(0, Math.min(8, decimals));
  const safePad = Math.max(0, Math.min(12, padZeros));
  const isNegative = valNum < 0;
  const absVal = Math.abs(valNum);
  const formatted = absVal.toFixed(safeDecimals);
  if (safePad > 0) {
    const parts = formatted.split(".");
    parts[0] = parts[0].padStart(safePad, "0");
    const padded = parts.join(".");
    return isNegative ? `-${padded}` : padded;
  }
  return isNegative ? `-${formatted}` : formatted;
}

