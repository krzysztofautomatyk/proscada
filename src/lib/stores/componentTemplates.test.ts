import assert from "node:assert/strict";
import { test } from "node:test";
import { get } from "svelte/store";
import {
  createEmptyProject,
} from "$lib/utils/projectTree";
import {
  project,
  selectedFormId,
  setSelection,
  createComponentTemplateFromSelection,
  instantiateComponentTemplate,
  generateComponentInstancesBatch,
  deleteComponentTemplate,
  extractTagsFromWidgets,
  autoCreateMissingTagsForComponent,
} from "$lib/stores/app";
import type { WidgetDef } from "$lib/types";

test("extractTagsFromWidgets detects tag_id and config tags", () => {
  const widgets: WidgetDef[] = [
    {
      id: "w1",
      widget_type: "process_symbol",
      x: 0,
      y: 0,
      w: 100,
      h: 100,
      z: 1,
      tag_id: "PUMP_Status",
      group_id: null,
      config: { analogTagId: "PUMP_Speed" },
    },
    {
      id: "w2",
      widget_type: "button",
      x: 100,
      y: 0,
      w: 80,
      h: 40,
      z: 2,
      tag_id: "PUMP_Cmd",
      group_id: null,
      config: {},
    },
  ];

  const tags = extractTagsFromWidgets(widgets);
  assert.ok(tags.includes("PUMP_Status"));
  assert.ok(tags.includes("PUMP_Speed"));
  assert.ok(tags.includes("PUMP_Cmd"));
});

test("autoCreateMissingTagsForComponent creates missing memory tags with guessed data types", () => {
  const tagNames = ["PUMP1_Status", "PUMP1_Speed", "PUMP1_Cmd"];
  const created = autoCreateMissingTagsForComponent(tagNames, []);

  assert.equal(created.length, 3);
  const statusTag = created.find((t) => t.id === "PUMP1_Status");
  assert.equal(statusTag?.data_type, "bool");

  const speedTag = created.find((t) => t.id === "PUMP1_Speed");
  assert.equal(speedTag?.data_type, "f32");
});

test("Component creation and Solution Explorer tree synchronization", () => {
  const proj = createEmptyProject("Component Test Proj");
  project.set(proj);
  selectedFormId.set(proj.forms[0].id);

  // Add 2 widgets to form
  const w1: WidgetDef = {
    id: "w_pump1",
    widget_type: "process_symbol",
    x: 10,
    y: 10,
    w: 100,
    h: 100,
    z: 1,
    tag_id: "PUMP_Status",
    group_id: null,
    config: {},
  };
  const w2: WidgetDef = {
    id: "w_btn1",
    widget_type: "button",
    x: 120,
    y: 10,
    w: 80,
    h: 40,
    z: 2,
    tag_id: "PUMP_Cmd",
    group_id: null,
    config: {},
  };

  project.update((p) => {
    if (!p) return p;
    return {
      ...p,
      forms: [{ ...p.forms[0], widgets: [w1, w2] }],
    };
  });

  // Select widgets and save component
  setSelection(["w_pump1", "w_btn1"], "w_pump1");
  const templateId = createComponentTemplateFromSelection("Pompa Testowa");
  assert.ok(templateId);

  const updated = get(project);
  assert.ok(updated);

  // Check component_templates
  const tmpl = updated?.component_templates?.find((t) => t.id === templateId);
  assert.ok(tmpl);
  assert.equal(tmpl?.name, "Pompa Testowa");
  assert.ok(tmpl?.extracted_tags?.includes("PUMP_Status"));
  assert.ok(tmpl?.extracted_tags?.includes("PUMP_Cmd"));

  // Check Solution Explorer tree synchronization
  const compFolder = updated?.tree?.find((n) => n.kind === "components_folder");
  assert.ok(compFolder);
  assert.equal(compFolder?.name, "Komponenty");

  const compNode = updated?.tree?.find((n) => n.kind === "component" && n.ref_id === templateId);
  assert.ok(compNode);
  assert.equal(compNode?.name, "Pompa Testowa");
  assert.equal(compNode?.parent_id, compFolder?.id);
});

test("Multi-instance batch instantiation (5 pumps) creates widgets and missing tags", () => {
  const proj = createEmptyProject("Batch Test Proj");
  project.set(proj);
  selectedFormId.set(proj.forms[0].id);

  // Create component template
  const w1: WidgetDef = {
    id: "w_pump",
    widget_type: "process_symbol",
    x: 0,
    y: 0,
    w: 100,
    h: 100,
    z: 1,
    tag_id: "{tagPrefix}Status",
    group_id: null,
    config: {},
  };
  setSelection([], null);

  project.update((p) => {
    if (!p) return p;
    return {
      ...p,
      forms: [{ ...p.forms[0], widgets: [w1] }],
      component_templates: [
        {
          id: "cmp_pump_std",
          name: "Pompa Standardowa",
          category: "Pompy",
          version: "1.0.0",
          description: "Test component",
          width: 100,
          height: 100,
          parameter_names: ["tagPrefix"],
          extracted_tags: ["{tagPrefix}Status"],
          widgets: [w1],
        },
      ],
    };
  });

  // Batch instantiate 5 pumps (PUMP1..PUMP5)
  const widgetIds = generateComponentInstancesBatch("cmp_pump_std", 5, "PUMP{n}_", {
    autoCreateTags: true,
    layout: "row",
  });

  assert.ok(widgetIds.length >= 5);

  const updated = get(project);
  assert.ok(updated);

  // Verify tags in project.tags: PUMP1_Status .. PUMP5_Status
  const tagIds = updated?.tags.map((t) => t.id) ?? [];
  assert.ok(tagIds.includes("PUMP1_Status"));
  assert.ok(tagIds.includes("PUMP2_Status"));
  assert.ok(tagIds.includes("PUMP3_Status"));
  assert.ok(tagIds.includes("PUMP4_Status"));
  assert.ok(tagIds.includes("PUMP5_Status"));

  // Delete component and verify tree node removal
  deleteComponentTemplate("cmp_pump_std");
  const finalProj = get(project);
  const deletedNode = finalProj?.tree?.find((n) => n.ref_id === "cmp_pump_std");
  assert.equal(deletedNode, undefined);
});

test("createComponentTemplateFromSelection enforces unique name, supports category, version, and description", () => {
  const proj = createEmptyProject("Unique Component Test");
  project.set(proj);
  selectedFormId.set(proj.forms[0].id);

  const w1: WidgetDef = {
    id: "w_pump1",
    widget_type: "process_symbol",
    x: 10,
    y: 10,
    w: 100,
    h: 100,
    z: 1,
    tag_id: "{tagPrefix}Run",
    group_id: null,
    config: {},
  };

  project.update((p) => {
    if (!p) return p;
    return { ...p, forms: [{ ...p.forms[0], widgets: [w1] }] };
  });

  setSelection(["w_pump1"], "w_pump1");

  // First creation — should succeed
  const id1 = createComponentTemplateFromSelection(
    "Pompa Główna V1",
    "Pompy",
    "2.1.0",
    "Opis zespołu pompy głównej",
  );
  assert.ok(id1);

  const current = get(project);
  const tmpl = current?.component_templates?.find((t) => t.id === id1);
  assert.equal(tmpl?.name, "Pompa Główna V1");
  assert.equal(tmpl?.category, "Pompy");
  assert.equal(tmpl?.version, "2.1.0");
  assert.equal(tmpl?.description, "Opis zespołu pompy głównej");
  assert.ok(tmpl?.tag_slots_meta && tmpl.tag_slots_meta.length > 0);

  // Duplicate creation attempt — must be rejected (returns null)
  const id2 = createComponentTemplateFromSelection(
    "Pompa Główna V1",
    "Custom",
    "1.0.0",
    "Duplicate",
  );
  assert.equal(id2, null);
});
