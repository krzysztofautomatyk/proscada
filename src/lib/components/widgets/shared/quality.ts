import type { TagValue, WidgetDef } from "$lib/types";

/**
 * How trustworthy a bound process value currently is.
 *
 * `ok` is the only state in which a widget may render the value as a fact.
 * Every other state must be visually distinguishable, otherwise the operator
 * cannot tell a live reading from a dead one.
 */
export type TagTrust = "ok" | "stale" | "bad" | "missing" | "unbound";

export interface TagQuality {
  trust: TagTrust;
  /** True when the widget is bound to a tag but the value must not be trusted. */
  degraded: boolean;
  /** Short operator-facing label, e.g. `BAD`. */
  label: string;
  /** Longer explanation for tooltips. */
  reason: string;
  /** Age of the sample in milliseconds, when known. */
  ageMs: number | null;
}

const UNBOUND: TagQuality = {
  trust: "unbound",
  degraded: false,
  label: "",
  reason: "",
  ageMs: null,
};

/**
 * Resolve the trust level of the value a widget is about to display.
 *
 * `design` short-circuits to `unbound` because the Designer intentionally shows
 * placeholder geometry, not process data.
 */
export function resolveTagQuality(
  widget: Pick<WidgetDef, "tag_id">,
  tag: TagValue | null | undefined,
  design = false,
): TagQuality {
  if (design || !widget.tag_id) return UNBOUND;

  if (!tag) {
    return {
      trust: "missing",
      degraded: true,
      label: "NO TAG",
      reason: "The bound tag is not present in the running project",
      ageMs: null,
    };
  }

  const ageMs = Number.isFinite(tag.age_ms) ? tag.age_ms : null;

  if (tag.quality === "bad") {
    return {
      trust: "bad",
      degraded: true,
      label: "BAD",
      reason: "The device reported no usable value for this tag",
      ageMs,
    };
  }

  if (tag.quality === "uncertain") {
    return {
      trust: "stale",
      degraded: true,
      label: "STALE",
      reason:
        ageMs === null
          ? "The sample is older than the accepted refresh window"
          : `The sample is ${Math.round(ageMs / 1000)} s old`,
      ageMs,
    };
  }

  return {
    trust: "ok",
    degraded: false,
    label: "GOOD",
    reason: ageMs === null ? "Live value" : `Live value, ${ageMs} ms old`,
    ageMs,
  };
}

/**
 * Text to render instead of a numeric value when the value cannot be trusted.
 *
 * Widgets must never substitute a plausible-looking number: a fabricated level
 * is indistinguishable from a real one.
 */
export const NO_VALUE_PLACEHOLDER = "––";

/** Format a numeric process value, or the placeholder when it is not trustworthy. */
export function formatTrustedValue(
  quality: TagQuality,
  value: number | null | undefined,
  decimals = 0,
): string {
  if (quality.degraded || value === null || value === undefined || !Number.isFinite(value)) {
    return NO_VALUE_PLACEHOLDER;
  }
  return value.toFixed(decimals);
}
