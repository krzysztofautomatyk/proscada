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

test("bad and uncertain quality are both degraded and distinguishable", () => {
  const bad = resolveTagQuality(widget, sample("bad"));
  assert.equal(bad.trust, "bad");
  assert.equal(bad.degraded, true);
  assert.equal(bad.label, "BAD");

  const stale = resolveTagQuality(widget, sample("uncertain", 5200));
  assert.equal(stale.trust, "stale");
  assert.equal(stale.degraded, true);
  assert.equal(stale.label, "STALE");
  assert.match(stale.reason, /5 s old/);
});

test("good quality is the only trusted state", () => {
  const result = resolveTagQuality(widget, sample("good"));
  assert.equal(result.trust, "ok");
  assert.equal(result.degraded, false);
});

test("formatTrustedValue refuses to render a number that cannot be trusted", () => {
  const bad = resolveTagQuality(widget, sample("bad"));
  assert.equal(formatTrustedValue(bad, 421, 0), NO_VALUE_PLACEHOLDER);

  const good = resolveTagQuality(widget, sample("good"));
  assert.equal(formatTrustedValue(good, 421.44, 1), "421.4");
  assert.equal(formatTrustedValue(good, null), NO_VALUE_PLACEHOLDER);
  assert.equal(formatTrustedValue(good, Number.NaN), NO_VALUE_PLACEHOLDER);
});
