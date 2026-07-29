import assert from "node:assert/strict";
import test from "node:test";

import type { TagValue, WidgetDef } from "$lib/types";
import {
  formatTrustedValue,
  NO_VALUE_PLACEHOLDER,
  resolveTagQuality,
} from "$lib/components/widgets/shared/quality";

const widget = { tag_id: "wt.level_cm" } as Pick<WidgetDef, "tag_id">;

function sample(quality: TagValue["quality"], ageMs = 100): TagValue {
  return {
    tag_id: "wt.level_cm",
    value: 421,
    bool_value: false,
    quality,
    ts: new Date().toISOString(),
    age_ms: ageMs,
    raw: 421,
  };
}

test("an unbound widget is not treated as degraded", () => {
  const result = resolveTagQuality({ tag_id: null }, null);
  assert.equal(result.trust, "unbound");
  assert.equal(result.degraded, false);
});

test("designer preview never claims live process data", () => {
  const result = resolveTagQuality(widget, sample("good"), true);
  assert.equal(result.trust, "unbound");
  assert.equal(result.degraded, false);
});

test("a bound widget without a tag is degraded, not blank", () => {
  const result = resolveTagQuality(widget, null);
  assert.equal(result.trust, "missing");
  assert.equal(result.degraded, true);
  assert.equal(result.label, "NO TAG");
});

test("bad, comm_lost and uncertain quality are degraded, distinguishable, and retain last valid values", () => {
  const bad = resolveTagQuality(widget, sample("bad"));
  assert.equal(bad.trust, "bad");
  assert.equal(bad.degraded, true);
  assert.equal(bad.label, "BAD");
  assert.equal(bad.lastValidValue, 421);

  const commLost = resolveTagQuality(widget, sample("comm_lost"));
  assert.equal(commLost.trust, "comm_lost");
  assert.equal(commLost.degraded, true);
  assert.equal(commLost.label, "COMM LOST");
  assert.equal(commLost.lastValidValue, 421);

  const stale = resolveTagQuality(widget, sample("uncertain", 5200));
  assert.equal(stale.trust, "stale");
  assert.equal(stale.degraded, true);
  assert.equal(stale.label, "STALE");
  assert.match(stale.reason, /5 s old/);
  assert.equal(stale.lastValidValue, 421);
});

test("good quality is the only trusted state", () => {
  const result = resolveTagQuality(widget, sample("good"));
  assert.equal(result.trust, "ok");
  assert.equal(result.degraded, false);
  assert.equal(result.lastValidValue, 421);
});

test("formatTrustedValue refuses to render an un-annotated number when untrusted, but supports showLastKnown", () => {
  const bad = resolveTagQuality(widget, sample("bad"));
  assert.equal(formatTrustedValue(bad, 421, 0), NO_VALUE_PLACEHOLDER);
  assert.equal(formatTrustedValue(bad, 421, 0, { showLastKnown: true }), "421 (BAD)");

  const commLost = resolveTagQuality(widget, sample("comm_lost"));
  assert.equal(formatTrustedValue(commLost, 421, 1, { showLastKnown: true }), "421.0 (COMM LOST)");

  const good = resolveTagQuality(widget, sample("good"));
  assert.equal(formatTrustedValue(good, 421.44, 1), "421.4");
  assert.equal(formatTrustedValue(good, null), NO_VALUE_PLACEHOLDER);
  assert.equal(formatTrustedValue(good, Number.NaN), NO_VALUE_PLACEHOLDER);
});
