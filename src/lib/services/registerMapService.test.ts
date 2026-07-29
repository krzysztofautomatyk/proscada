import { describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  extractDevicePollQueries,
  buildRegisterMap,
  validateRegisterTag,
  setTagReadonly,
} from "./registerMapService";
import type { DeviceConfig, TagDefinition } from "../types";

describe("registerMapService domain rules", () => {
  const sampleDevice: DeviceConfig = {
    id: "PLC_TEST",
    name: "Test PLC",
    host: "127.0.0.1",
    port: 5020,
    unit_id: 1,
    poll_ms: 1000,
    timeout_ms: 2000,
    enabled: true,
    queries: [
      {
        id: "q1",
        name: "Analog Inputs Block",
        table: "holding",
        start_address: 100,
        count: 20,
        poll_ms: 500,
        enabled: true,
      },
    ],
  };

  it("extracts queries from device config correctly", () => {
    const queries = extractDevicePollQueries(sampleDevice);
    assert.equal(queries.length, 1);
    assert.equal(queries[0].startAddress, 100);
    assert.equal(queries[0].endAddress, 119);
    assert.equal(queries[0].table, "holding");
    assert.ok(queries[0].displayLabel.includes("Modbus TCP 4x R100-119"));
  });

  it("supports MULTIPLE tags per single register address (word tag + bit tags)", () => {
    const query = extractDevicePollQueries(sampleDevice)[0];
    const tags: TagDefinition[] = [
      {
        id: "di_pack",
        name: "DI_PACK",
        device_id: "PLC_TEST",
        data_type: "u16",
        binding: { address: 100, table: "holding", writable: false },
        unit: "",
        description: "Packed Discrete Inputs",
        scale: 1,
        offset: 0,
        decimals: 0,
      },
      {
        id: "sim_en",
        name: "SIM_EN",
        device_id: "PLC_TEST",
        data_type: "bool",
        binding: { address: 100, table: "holding", bit: 0, writable: false },
        unit: "",
        description: "Sim mode enable bit",
        scale: 1,
        offset: 0,
        decimals: 0,
      },
      {
        id: "flt_lo",
        name: "FLT_LO",
        device_id: "PLC_TEST",
        data_type: "bool",
        binding: { address: 100, table: "holding", bit: 1, writable: false },
        unit: "",
        description: "Low Float sensor bit",
        scale: 1,
        offset: 0,
        decimals: 0,
      },
    ];

    const mapRows = buildRegisterMap(query, tags);
    const r100 = mapRows.find((r) => r.address === 100);
    assert.ok(r100);
    assert.equal(r100.tags.length, 3); // 1 word tag + 2 bit tags
    assert.equal(r100.primaryTag?.id, "di_pack");
    assert.ok(r100.bits);
    assert.equal(r100.bits.find((b) => b.bitIndex === 0)?.tagName, "SIM_EN");
    assert.equal(r100.bits.find((b) => b.bitIndex === 1)?.tagName, "FLT_LO");
  });

  it("handles f32 register multi-word span continuations", () => {
    const query = extractDevicePollQueries(sampleDevice)[0];
    const tags: TagDefinition[] = [
      {
        id: "flow_f32",
        name: "FLOW_RATE",
        device_id: "PLC_TEST",
        data_type: "f32",
        binding: {
          address: 104,
          table: "holding",
          writable: true,
        },
        unit: "m³/h",
        description: "Flow rate float",
        scale: 1,
        offset: 0,
        decimals: 2,
      },
    ];

    const mapRows = buildRegisterMap(query, tags);
    const r104 = mapRows.find((r) => r.address === 104);
    const r105 = mapRows.find((r) => r.address === 105);

    assert.equal(r104?.dataType, "f32");
    assert.equal(r104?.span, 2);
    assert.equal(r104?.isSpanContinuation, false);

    assert.equal(r105?.dataType, "f32");
    assert.equal(r105?.isSpanContinuation, true);
    assert.equal(r105?.parentAddress, 104);
  });

  it("validates bit index bounds 0..15 for holding registers", () => {
    const invalidTag: Partial<TagDefinition> = {
      id: "invalid_bit",
      name: "Bit Out of Bounds",
      device_id: "PLC_TEST",
      data_type: "bool",
      binding: {
        address: 100,
        table: "holding",
        bit: 18, // Invalid! Range is 0..15
        writable: true,
      },
    };

    const res = validateRegisterTag(invalidTag, []);
    assert.equal(res.valid, false);
    assert.ok(res.errors.some((e) => e.includes("0..15")));
  });

  it("toggles readonly flag cleanly", () => {
    const tag: TagDefinition = {
      id: "t1",
      name: "Tag 1",
      device_id: "PLC1",
      data_type: "u16",
      binding: { address: 10, table: "holding", writable: true },
      unit: "",
      description: "",
      scale: 1,
      offset: 0,
      decimals: 0,
    };

    const readonlyTag = setTagReadonly(tag, true);
    assert.equal(readonlyTag.binding.writable, false);

    const writableTag = setTagReadonly(readonlyTag, false);
    assert.equal(writableTag.binding.writable, true);
  });

  it("validates internal memory tags (bool, f32) without requiring Modbus device or bit index", () => {
    const memoryBoolTag: Partial<TagDefinition> = {
      id: "mem_flag",
      name: "Memory Flag",
      device_id: "SYS_INTERNAL",
      data_type: "bool",
      binding: {
        address: 0,
        table: "memory",
        writable: true,
      },
    };

    const resBool = validateRegisterTag(memoryBoolTag, []);
    assert.equal(resBool.valid, true);

    const memoryF32Tag: Partial<TagDefinition> = {
      id: "mem_calc",
      name: "Calculated Value",
      device_id: "SYS_INTERNAL",
      data_type: "f32",
      binding: {
        address: 0,
        table: "memory",
        writable: true,
      },
    };

    const resF32 = validateRegisterTag(memoryF32Tag, []);
    assert.equal(resF32.valid, true);
  });
});
