import fs from "node:fs";
import type { ScadaProject, WidgetDef, FormDef, ComponentTemplate, TagDefinition } from "../src/lib/types.js";
import { ensureProjectTree } from "../src/lib/utils/projectTree.js";

const path = "/Users/krzysztof/Downloads/Nowy_Projekt_SCADA.proscada.json";
const raw = fs.readFileSync(path, "utf-8");
const proj: ScadaProject = JSON.parse(raw);

const mainForm = proj.forms.find((f) => f.name === "Main");
if (!mainForm) throw new Error("Main form not found");

// Min X, Y calculation
const minX = Math.min(...mainForm.widgets.map((w) => w.x));
const minY = Math.min(...mainForm.widgets.map((w) => w.y));
const maxX = Math.max(...mainForm.widgets.map((w) => w.x + w.w));
const maxY = Math.max(...mainForm.widgets.map((w) => w.y + w.h));

// Template widgets with parameterized tag IDs
const templateWidgets: WidgetDef[] = mainForm.widgets.map((w) => {
  const clone = JSON.parse(JSON.stringify(w)) as WidgetDef;
  clone.x = w.x - minX;
  clone.y = w.y - minY;

  // Parameterize tag_id
  if (clone.tag_id === "PompaNazwa") clone.tag_id = "{tagPrefix}Nazwa";
  else if (clone.tag_id === "1p") clone.tag_id = "{tagPrefix}Run";
  else if (clone.tag_id === "Step") clone.tag_id = "{tagPrefix}Step";
  else if (clone.tag_id === "p1_hh") clone.tag_id = "{tagPrefix}hh";
  else if (clone.tag_id === "p1_mm") clone.tag_id = "{tagPrefix}mm";
  else if (clone.tag_id === "p1_ss") clone.tag_id = "{tagPrefix}ss";

  // Parameterize config tags
  if (clone.config) {
    if (clone.config.blinkTagId === "1p") clone.config.blinkTagId = "{tagPrefix}Run";
    if (clone.config.stateTagId === "1p") clone.config.stateTagId = "{tagPrefix}Run";
    if (clone.config.visibilityTagId === "1p") clone.config.visibilityTagId = "{tagPrefix}Run";
    if (clone.config.animationTagId === "1p") clone.config.animationTagId = "{tagPrefix}Run";
  }

  return clone;
});

const template: ComponentTemplate = {
  id: "cmp_pompa_stacja",
  name: "Pompa Zespołowa (z Counter i Timer)",
  category: "Pompy",
  version: "1.0.0",
  description: "Zespól pompy ze wskaźnikami Counter i Timer z ekranu Main",
  width: maxX - minX,
  height: maxY - minY,
  parameter_names: ["tagPrefix", "name"],
  extracted_tags: [
    "{tagPrefix}Nazwa",
    "{tagPrefix}Run",
    "{tagPrefix}Step",
    "{tagPrefix}hh",
    "{tagPrefix}mm",
    "{tagPrefix}ss",
  ],
  widgets: templateWidgets,
};

proj.component_templates = [template];

// Create tags for P1 and P2 if missing
const extraTags: TagDefinition[] = [
  // P1 tags
  { id: "P1_Nazwa", name: "P1 Nazwa", device_id: "memory", data_type: "string", binding: { table: "memory", address: 0 }, unit: "", description: "", scale: 1, offset: 0, decimals: 0, initial_value: "Pompa 1" },
  { id: "P1_Run", name: "P1 Run", device_id: "dev_ms24ue9e_n4j8u", data_type: "bool", binding: { table: "holding", address: 101, bit: 0, writable: true }, unit: "", description: "", scale: 1, offset: 0, decimals: 0 },
  { id: "P1_Step", name: "P1 Counter Step", device_id: "dev_ms24ue9e_n4j8u", data_type: "u16", binding: { table: "holding", address: 106, writable: true }, unit: "", description: "", scale: 1, offset: 0, decimals: 0 },
  { id: "P1_hh", name: "P1 Timer HH", device_id: "dev_ms24ue9e_n4j8u", data_type: "u16", binding: { table: "holding", address: 114, writable: true }, unit: "", description: "", scale: 1, offset: 0, decimals: 0 },
  { id: "P1_mm", name: "P1 Timer MM", device_id: "dev_ms24ue9e_n4j8u", data_type: "u16", binding: { table: "holding", address: 115, writable: true }, unit: "", description: "", scale: 1, offset: 0, decimals: 0 },
  { id: "P1_ss", name: "P1 Timer SS", device_id: "dev_ms24ue9e_n4j8u", data_type: "u16", binding: { table: "holding", address: 116, writable: true }, unit: "", description: "", scale: 1, offset: 0, decimals: 0 },

  // P2 tags
  { id: "P2_Nazwa", name: "P2 Nazwa", device_id: "memory", data_type: "string", binding: { table: "memory", address: 0 }, unit: "", description: "", scale: 1, offset: 0, decimals: 0, initial_value: "Pompa 2" },
  { id: "P2_Run", name: "P2 Run", device_id: "dev_ms24ue9e_n4j8u", data_type: "u16", binding: { table: "holding", address: 102, writable: true }, unit: "", description: "", scale: 1, offset: 0, decimals: 0 },
  { id: "P2_Step", name: "P2 Counter Step", device_id: "dev_ms24ue9e_n4j8u", data_type: "u16", binding: { table: "holding", address: 107, writable: true }, unit: "", description: "", scale: 1, offset: 0, decimals: 0 },
  { id: "P2_hh", name: "P2 Timer HH", device_id: "dev_ms24ue9e_n4j8u", data_type: "u16", binding: { table: "holding", address: 124, writable: true }, unit: "", description: "", scale: 1, offset: 0, decimals: 0 },
  { id: "P2_mm", name: "P2 Timer MM", device_id: "dev_ms24ue9e_n4j8u", data_type: "u16", binding: { table: "holding", address: 125, writable: true }, unit: "", description: "", scale: 1, offset: 0, decimals: 0 },
  { id: "P2_ss", name: "P2 Timer SS", device_id: "dev_ms24ue9e_n4j8u", data_type: "u16", binding: { table: "holding", address: 126, writable: true }, unit: "", description: "", scale: 1, offset: 0, decimals: 0 },
];

for (const t of extraTags) {
  if (!proj.tags.some((x) => x.id === t.id)) {
    proj.tags.push(t);
  }
}

// Function to build component instance
function buildInstance(prefix: string, posX: number, posY: number, grpId: string): WidgetDef[] {
  return templateWidgets.map((source, index) => {
    const w = JSON.parse(JSON.stringify(source)) as WidgetDef;
    w.id = `w_pomp_${prefix.toLowerCase().replace(/[^a-z0-9]/g, "")}_${index}`;
    w.x = posX + source.x;
    w.y = posY + source.y;
    w.z = index + 1;
    w.group_id = grpId;

    if (w.tag_id) {
      w.tag_id = w.tag_id.replace("{tagPrefix}", prefix);
    }
    if (w.config) {
      if (w.config.blinkTagId) w.config.blinkTagId = String(w.config.blinkTagId).replace("{tagPrefix}", prefix);
      if (w.config.stateTagId) w.config.stateTagId = String(w.config.stateTagId).replace("{tagPrefix}", prefix);
      if (w.config.visibilityTagId) w.config.visibilityTagId = String(w.config.visibilityTagId).replace("{tagPrefix}", prefix);
      if (w.config.animationTagId) w.config.animationTagId = String(w.config.animationTagId).replace("{tagPrefix}", prefix);
    }
    return w;
  });
}

// Create new screen: Pompownia
const pompowniaWidgets = [
  ...buildInstance("P1_", 60, 60, "grp_p1"),
  ...buildInstance("P2_", 320, 60, "grp_p2"),
];

const newForm: FormDef = {
  id: "form_pompownia_stacja_2x",
  name: "Pompownia 2x Pompa",
  width: 1040,
  height: 700,
  background: "#F4F5F7",
  grid: 8,
  widgets: pompowniaWidgets,
};

// Check if form already exists
const existingFormIdx = proj.forms.findIndex((f) => f.id === newForm.id || f.name === newForm.name);
if (existingFormIdx >= 0) {
  proj.forms[existingFormIdx] = newForm;
} else {
  proj.forms.push(newForm);
}

// Normalize and synchronize project tree
const finalProj = ensureProjectTree(proj);

// Write back to user file
fs.writeFileSync(path, JSON.stringify(finalProj, null, 2), "utf-8");

// Never copy a user project into a public fixture: project files can contain
// credential hashes and site-specific OT configuration.
console.log("Updated the selected user project with the component and screen.");
