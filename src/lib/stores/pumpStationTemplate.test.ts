import assert from "node:assert/strict";
import { beforeEach, test } from "node:test";
import { get } from "svelte/store";
import type { ScadaProject, TagDefinition } from "$lib/types";
import {
  bulkInstantiateComponentTemplate,
  dirty,
  installPumpStationTemplate,
  mode,
  project,
  selectedFormId,
  switchMode,
} from "$lib/stores/app";

function baseProject(tags: TagDefinition[] = []): ScadaProject {
  return {
    schema_version: 3,
    id: "test-project",
    name: "Pump station test",
    description: "",
    devices: [
      {
        id: "dev-1",
        name: "Test PLC",
        host: "127.0.0.1",
        port: 502,
        unit_id: 1,
        poll_ms: 250,
        timeout_ms: 1000,
        enabled: true,
      },
    ],
    tags,
    forms: [
      {
        id: "main",
        name: "Main",
        width: 1200,
        height: 800,
        background: "#ffffff",
        grid: 8,
        widgets: [],
      },
    ],
    alarms: [],
    alarm_groups: [],
    component_templates: [],
    tree: [],
    content_hash: "",
  };
}

function csv(baseAddress = 1000, deviceId = "dev-1"): string {
  return [
    "objectId,name,tagPrefix,alarmGroup,location,deviceId,baseAddress",
    `PS_001,Pompownia 001,PLC.PS001,Zaklad/Pompownie/PS_001,North,${deviceId},${baseAddress}`,
  ].join("\n");
}

beforeEach(() => {
  project.set(baseProject());
  selectedFormId.set("main");
  dirty.set(false);
  mode.set("designer");
});

test("pump template creates P1/P2 feedback and explicit command targets", () => {
  const templateId = installPumpStationTemplate();
  assert.equal(bulkInstantiateComponentTemplate(templateId, csv()), 1);

  const result = get(project);
  assert.ok(result);
  const tags = new Map(result.tags.map((tag) => [tag.id, tag]));
  for (const id of [
    "PLC.PS001.P1_RunFb",
    "PLC.PS001.P2_RunFb",
    "PLC.PS001.P1_StartCmd",
    "PLC.PS001.P1_StopCmd",
    "PLC.PS001.P2_StartCmd",
    "PLC.PS001.P2_StopCmd",
  ]) {
    assert.ok(tags.has(id), `missing generated tag ${id}`);
  }
  assert.equal(tags.get("PLC.PS001.P1_StartCmd")?.binding.writable, true);
  assert.equal(tags.get("PLC.PS001.P2_StopCmd")?.binding.verify_readback, false);

  const faceplates = result.forms[0].widgets.filter((widget) => widget.widget_type === "faceplate");
  assert.equal(faceplates.length, 2);
  const configs = faceplates.map((widget) => widget.config);
  assert.ok(
    configs.some(
      (config) =>
        config.startTagId === "PLC.PS001.P1_StartCmd" &&
        config.stopTagId === "PLC.PS001.P1_StopCmd",
    ),
  );
  assert.ok(
    configs.some(
      (config) =>
        config.startTagId === "PLC.PS001.P2_StartCmd" &&
        config.stopTagId === "PLC.PS001.P2_StopCmd",
    ),
  );
  assert.equal(result.alarms.length, 5);
  assert.ok(result.alarm_groups?.some((group) => group.id === "Zaklad/Pompownie/PS_001"));
});

test("pump template rejects missing mapping, unknown devices and address collisions", () => {
  const templateId = installPumpStationTemplate();
  assert.throws(
    () =>
      bulkInstantiateComponentTemplate(
        templateId,
        "objectId,name,tagPrefix,alarmGroup,location\nPS_001,Pump,PLC.PS001,Plant/PS_001,North",
      ),
    /deviceId/,
  );
  assert.throws(() => bulkInstantiateComponentTemplate(templateId, csv(1000, "missing")), /does not exist/);

  project.set(
    baseProject([
      {
        id: "existing",
        name: "Existing register",
        device_id: "dev-1",
        data_type: "u16",
        binding: { address: 1000, table: "holding", writable: false },
        unit: "",
        description: "",
        scale: 1,
        offset: 0,
        decimals: 0,
      },
    ]),
  );
  selectedFormId.set("main");
  const collisionTemplateId = installPumpStationTemplate();
  assert.throws(
    () => bulkInstantiateComponentTemplate(collisionTemplateId, csv()),
    /physical holding address collision/,
  );
});

test("entering Runtime persists a dirty generated project", async () => {
  const templateId = installPumpStationTemplate();
  bulkInstantiateComponentTemplate(templateId, csv());
  assert.equal(get(dirty), true);

  await switchMode("runtime");

  assert.equal(get(dirty), false);
  assert.equal(get(mode), "runtime");
});

