import assert from "node:assert/strict";
import { test } from "node:test";
import type { TagValue } from "$lib/types";
import { evaluateCondition } from "./dynamics";

function tag(quality: TagValue["quality"], value = 1): TagValue {
  return {
    tag_id: "process.run",
    value,
    bool_value: value !== 0,
    quality,
    ts: "2026-07-29T00:00:00Z",
    age_ms: 10,
    raw: value,
  };
}

test("data-driven dynamics evaluate only Good-quality samples", () => {
  for (const quality of ["bad", "uncertain"] as const) {
    const tags = new Map([["process.run", tag(quality)]]);
    assert.equal(
      evaluateCondition("tag_true", "process.run", 0, 0, tags),
      false,
      `${quality} must fail closed`,
    );
  }

  const good = new Map([["process.run", tag("good")]]);
  assert.equal(evaluateCondition("tag_true", "process.run", 0, 0, good), true);
});

test("constant dynamics do not require a process sample", () => {
  assert.equal(evaluateCondition("always", null, 0, 0, new Map()), true);
  assert.equal(evaluateCondition("none", null, 0, 0, new Map()), false);
});
