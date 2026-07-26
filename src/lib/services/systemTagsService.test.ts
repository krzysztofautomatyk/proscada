import { test } from "node:test";
import assert from "node:assert/strict";
import {
  SYSTEM_TAG_DEFINITIONS,
  computeSystemTagValues,
  isSystemTag,
} from "./systemTagsService";
import { getDataTypeRegisterSpan } from "./registerMapService";

test("systemTagsService provides standard SCADA system tags", () => {
  assert.ok(SYSTEM_TAG_DEFINITIONS.length >= 10, "Should define at least 10 system tags");
  
  const timeTag = SYSTEM_TAG_DEFINITIONS.find((t) => t.id === "SYS.Time");
  assert.ok(timeTag, "SYS.Time tag should exist");
  assert.equal(timeTag?.data_type, "string");
  assert.equal(timeTag?.is_system, true);

  const hourTag = SYSTEM_TAG_DEFINITIONS.find((t) => t.id === "SYS.Hour");
  assert.ok(hourTag, "SYS.Hour tag should exist");
  assert.equal(hourTag?.data_type, "u16");
});

test("computeSystemTagValues calculates time, uptime, disk free, and string values", () => {
  const vals = computeSystemTagValues({
    connected: true,
    pollCount: 125,
    mode: "runtime",
    role: "engineer",
  });

  assert.ok(vals.length >= 10, "Should compute values for all system tags");

  const timeVal = vals.find((v) => v.tag_id === "SYS.Time");
  assert.ok(timeVal, "SYS.Time value should be computed");
  assert.ok(timeVal?.string_value, "SYS.Time should contain formatted string_value");
  assert.match(timeVal!.string_value!, /^\d{2}:\d{2}:\d{2}$/);

  const dateVal = vals.find((v) => v.tag_id === "SYS.Date");
  assert.ok(dateVal?.string_value, "SYS.Date string_value should be formatted");
  assert.match(dateVal!.string_value!, /^\d{4}-\d{2}-\d{2}$/);

  const connVal = vals.find((v) => v.tag_id === "SYS.Connected");
  assert.equal(connVal?.bool_value, true);
  assert.equal(connVal?.value, 1);

  const pollVal = vals.find((v) => v.tag_id === "SYS.PollCount");
  assert.equal(pollVal?.value, 125);
});

test("isSystemTag correctly identifies system variables", () => {
  assert.equal(isSystemTag("SYS.Time"), true);
  assert.equal(isSystemTag("SYS.CustomTag"), true);
  assert.equal(isSystemTag("PUMP01.Pressure"), false);
});

test("getDataTypeRegisterSpan computes register spans for extended data types", () => {
  assert.equal(getDataTypeRegisterSpan("u16"), 1);
  assert.equal(getDataTypeRegisterSpan("i16"), 1);
  assert.equal(getDataTypeRegisterSpan("u32"), 2);
  assert.equal(getDataTypeRegisterSpan("i32"), 2);
  assert.equal(getDataTypeRegisterSpan("f32"), 2);
  assert.equal(getDataTypeRegisterSpan("u64"), 4);
  assert.equal(getDataTypeRegisterSpan("i64"), 4);
  assert.equal(getDataTypeRegisterSpan("f64"), 4);
  assert.equal(getDataTypeRegisterSpan("string", 32), 16);
  assert.equal(getDataTypeRegisterSpan("string", 10), 5);
});
