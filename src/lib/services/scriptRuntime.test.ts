import assert from "node:assert/strict";
import test from "node:test";

import {
  evaluateCondition,
  parseScript,
  ScriptParseError,
  type ScriptCondition,
} from "$lib/services/scriptRuntime";

test("parseScript accepts the whitelisted action language", () => {
  const statements = parseScript(
    [
      "# start the lead pump",
      '  writeTag "wt.sp_p1_on" 700',
      '// comment',
      '',
      'ackAlarm "alm.level_high"',
      'navigate "form_main"',
      'log "sequence complete"',
    ].join("\n"),
  );

  assert.equal(statements.length, 4);
  assert.deepEqual(statements[0].action, {
    kind: "writeTag",
    tagId: "wt.sp_p1_on",
    value: 700,
  });
  assert.deepEqual(statements[1].action, { kind: "ackAlarm", alarmId: "alm.level_high" });
  assert.deepEqual(statements[2].action, { kind: "navigate", formId: "form_main" });
  assert.deepEqual(statements[3].action, { kind: "log", message: "sequence complete" });
  assert.equal(statements[0].condition, null);
});

test("parseScript understands guarded statements", () => {
  const [statement] = parseScript('if "wt.level_cm" >= 800 then writeTag "wt.sp_stop" 1');
  assert.deepEqual(statement.condition, {
    tagId: "wt.level_cm",
    comparator: ">=",
    value: 800,
  });
  assert.deepEqual(statement.action, { kind: "writeTag", tagId: "wt.sp_stop", value: 1 });
});

test("parseScript rejects JavaScript instead of silently ignoring it", () => {
  assert.throws(
    () => parseScript('fetch("https://example.invalid")'),
    (error: unknown) => error instanceof ScriptParseError,
    "arbitrary JavaScript must not parse",
  );
  assert.throws(
    () => parseScript("while (true) {}"),
    (error: unknown) => error instanceof ScriptParseError,
  );
  assert.throws(
    () => parseScript('writeTag "wt.sp_stop"'),
    (error: unknown) => error instanceof ScriptParseError,
    "writeTag without a value must be an error",
  );
  assert.throws(
    () => parseScript('writeTag "wt.sp_stop" not-a-number'),
    (error: unknown) => error instanceof ScriptParseError,
  );
  assert.throws(
    () => parseScript('if "t" >= 1 writeTag "x" 1'),
    (error: unknown) => error instanceof ScriptParseError,
    "a missing then must be an error",
  );
  assert.throws(
    () => parseScript('log "unterminated'),
    (error: unknown) => error instanceof ScriptParseError,
  );
});

test("parse errors carry the offending line number", () => {
  try {
    parseScript(['log "ok"', "", "explode 1"].join("\n"));
    assert.fail("expected a parse error");
  } catch (error) {
    assert.ok(error instanceof ScriptParseError);
    assert.equal(error.line, 3);
  }
});

test("conditions never fire on missing or untrusted tag values", () => {
  const condition: ScriptCondition = { tagId: "t", comparator: ">", value: 10 };

  assert.equal(evaluateCondition(condition, () => undefined), false, "missing tag");
  assert.equal(
    evaluateCondition(condition, () => ({ value: 50, quality: "bad" })),
    false,
    "bad quality must not satisfy a condition",
  );
  assert.equal(
    evaluateCondition(condition, () => ({ value: 50, quality: "uncertain" })),
    false,
    "stale data must not satisfy a condition",
  );
  assert.equal(evaluateCondition(condition, () => ({ value: 50, quality: "good" })), true);
  assert.equal(evaluateCondition(condition, () => ({ value: 5, quality: "good" })), false);
});

test("every comparator behaves as written", () => {
  const good = (value: number) => () => ({ value, quality: "good" });
  const cases: Array<[ScriptCondition["comparator"], number, boolean]> = [
    ["==", 10, true],
    ["!=", 10, false],
    [">", 10, false],
    [">=", 10, true],
    ["<", 10, false],
    ["<=", 10, true],
  ];
  for (const [comparator, value, expected] of cases) {
    assert.equal(
      evaluateCondition({ tagId: "t", comparator, value: 10 }, good(value)),
      expected,
      `comparator ${comparator}`,
    );
  }
});
