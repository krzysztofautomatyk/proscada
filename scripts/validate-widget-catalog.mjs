import { existsSync, readdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const catalogPath = join(
  root,
  "src",
  "lib",
  "components",
  "widgets",
  "registry",
  "catalog.ts",
);
const viewPath = join(
  root,
  "src",
  "lib",
  "components",
  "widgets",
  "WidgetView.svelte",
);
const componentRoot = join(root, "src", "lib", "components", "widgets", "catalog");

const expected = new Map([
  ["label", "primitives"],
  ["shape", "primitives"],
  ["line", "primitives"],
  ["image", "assets"],
  ["vector_symbol", "assets"],
  ["state_indicator", "indicators"],
  ["numeric", "indicators"],
  ["meter", "indicators"],
  ["data_status", "indicators"],
  ["process_symbol", "process"],
  ["faceplate", "process"],
  ["command_button", "commands"],
  ["numeric_input", "inputs"],
  ["boolean_input", "inputs"],
  ["select_input", "inputs"],
  ["text_input", "inputs"],
  ["datetime_range", "inputs"],
  ["trend", "data"],
  ["collection_view", "data"],
  ["event_timeline", "data"],
  ["event_audit_viewer", "data"],
  ["panel", "layout"],
  ["disclosure_panel", "layout"],
  ["navigation_link", "navigation"],
  ["tab_set", "navigation"],
  ["navigation_menu", "navigation"],
  ["embedded_screen", "navigation"],
  ["breadcrumb", "navigation"],
  ["dialog", "feedback"],
  ["notification", "feedback"],
  ["tooltip", "feedback"],
  ["alarm_panel", "alarms"],
  ["alarm_banner", "alarms"],
  ["alarm_indicator", "alarms"],
  ["qr_code", "utilities"],
]);

const catalog = readFileSync(catalogPath, "utf8");
const view = readFileSync(viewPath, "utf8");
const appStore = readFileSync(
  join(root, "src", "lib", "stores", "app.ts"),
  "utf8",
);

const idMatch = catalog.match(/export const CANONICAL_WIDGET_IDS = \[([\s\S]*?)\] as const;/);
if (!idMatch) throw new Error("CANONICAL_WIDGET_IDS declaration not found");
const canonicalIds = [...idMatch[1].matchAll(/"([^"]+)"/g)].map((match) => match[1]);
if (canonicalIds.length !== 35 || new Set(canonicalIds).size !== 35) {
  throw new Error(`Expected 35 unique canonical IDs, got ${canonicalIds.length}`);
}

const sourceMigrationBlock = catalog.match(
  /export const SOURCE_WIDGET_MIGRATIONS:[\s\S]*?= \{([\s\S]*?)\r?\n\};/,
);
if (!sourceMigrationBlock) throw new Error("SOURCE_WIDGET_MIGRATIONS declaration not found");
const sourceMigrations = [
  ...sourceMigrationBlock[1].matchAll(/^\s*(?:"[^"]+"|[A-Za-z][A-Za-z0-9]*)\s*:\s*"([^"]+)"/gm),
].map((match) => match[1]);
if (sourceMigrations.length !== 33) {
  throw new Error(`Expected 33 source-control migrations, got ${sourceMigrations.length}`);
}

const canonicalBlock = catalog.match(
  /const canonicalWidgets: WidgetCatalogItem\[\] = \[([\s\S]*?)\r?\n\];\r?\n\r?\nconst processTemplates/,
);
if (!canonicalBlock) throw new Error("canonicalWidgets block not found");
const catalogTypes = [...canonicalBlock[1].matchAll(/\n\s*type: "([^"]+)"/g)].map(
  (match) => match[1],
);
if (catalogTypes.length !== 35 || new Set(catalogTypes).size !== 35) {
  throw new Error(`Expected 35 unique canonical widget types, got ${catalogTypes.length}`);
}

const missingExpected = [...expected.keys()].filter((type) => !catalogTypes.includes(type));
const unexpected = catalogTypes.filter((type) => !expected.has(type));
if (missingExpected.length || unexpected.length) {
  throw new Error(
    `Canonical type mismatch; missing=${missingExpected.join(",")}; unexpected=${unexpected.join(",")}`,
  );
}
const missingMigrationTargets = sourceMigrations.filter((type) => !catalogTypes.includes(type));
if (missingMigrationTargets.length) {
  throw new Error(`Source migrations target missing types: ${missingMigrationTargets.join(", ")}`);
}

const imports = new Map(
  [...view.matchAll(/import\s+(\w+)\s+from\s+"\.\/catalog\/([^"]+\.svelte)";/g)].map(
    (match) => [match[1], match[2]],
  ),
);
const branches = new Map(
  [
    ...view.matchAll(
      /\{(?:#if|:else if)\s+widget\.widget_type\s+===\s+"([^"]+)"\}\s*\r?\n\s*<(\w+)/g,
    ),
  ].map((match) => [match[1], match[2]]),
);

const missingBranches = catalogTypes.filter((type) => !branches.has(type));
if (missingBranches.length) {
  throw new Error(`Canonical widgets without renderer branch: ${missingBranches.join(", ")}`);
}

const canonicalRenderers = new Set();
for (const [type, folder] of expected) {
  const component = branches.get(type);
  const importPath = component ? imports.get(component) : undefined;
  if (!component || !importPath) {
    throw new Error(`Renderer import missing for ${type}`);
  }
  if (!importPath.startsWith(`${folder}/`)) {
    throw new Error(`${type} must live in ${folder}/, found ${importPath}`);
  }
  if (!existsSync(join(componentRoot, importPath))) {
    throw new Error(`Renderer file missing for ${type}: ${importPath}`);
  }
  if (canonicalRenderers.has(component)) {
    throw new Error(`Canonical controls must have separate component files; reused ${component}`);
  }
  canonicalRenderers.add(component);
}

const rootSvelteFiles = readdirSync(componentRoot).filter((name) => name.endsWith(".svelte"));
if (rootSvelteFiles.length) {
  throw new Error(`Widget components must be grouped in folders: ${rootSvelteFiles.join(", ")}`);
}

for (const required of [
  'startTagId: "{tagPrefix}.P1_StartCmd"',
  'stopTagId: "{tagPrefix}.P1_StopCmd"',
  'tag_id: "{tagPrefix}.P1_RunFb"',
  'startTagId: "{tagPrefix}.P2_StartCmd"',
  'stopTagId: "{tagPrefix}.P2_StopCmd"',
  'tag_id: "{tagPrefix}.P2_RunFb"',
  'boolTag("P1_StartCmd"',
  'boolTag("P1_StopCmd"',
  "const generatedTags = pumpStationTags(parameters, current)",
  "for (const tag of generatedTags)",
  "tags: nextTags",
  "verify_readback: !writable",
  'required.push("deviceId", "baseAddress")',
  "physical holding address collision",
  "Saving project before Runtime so the engine receives the current tag map",
]) {
  if (!appStore.includes(required)) {
    throw new Error(`Pump-station faceplate contract missing: ${required}`);
  }
}

console.log(
  `Widget catalog OK: ${canonicalIds.length} canonical IDs, ${catalogTypes.length} types, ${canonicalRenderers.size} separate renderers, ${sourceMigrations.length}/33 source migrations.`,
);
