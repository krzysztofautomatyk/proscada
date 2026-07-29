import assert from "node:assert/strict";
import { test } from "node:test";
import {
  ensureProjectTree,
  createEmptyProject,
  childrenOf,
  normalizeImportedProject,
  validateImportedProjectEnvelope,
} from "./projectTree";

test("ensureProjectTree guarantees Screens folder and syncs screen nodes for all forms", () => {
  // Scenario: Project has tree with only Images and Styles, but 2 forms (Main and Detail) in forms[]
  const rawProject = {
    schema_version: 3,
    id: "proj_test_screens",
    name: "Test Screens Sync",
    description: "",
    devices: [],
    tags: [],
    forms: [
      { id: "form_main", name: "Main", width: 1040, height: 700, background: "#ffffff", grid: 8, widgets: [] },
      { id: "form_2", name: "Detail", width: 1040, height: 700, background: "#ffffff", grid: 8, widgets: [] },
    ],
    alarms: [],
    tree: [
      { id: "img_fld", parent_id: null, kind: "folder" as const, name: "Images", order: 0 },
      { id: "style_fld", parent_id: null, kind: "folder" as const, name: "Styles", order: 1 },
    ],
    content_hash: "",
  };

  const synced = ensureProjectTree(rawProject);

  // 1. Screens folder must exist
  const screensFolder = synced.tree?.find((n) => n.kind === "folder" && n.name === "Screens");
  assert.ok(screensFolder, "Screens folder must exist");
  assert.equal(screensFolder?.collapsed, false, "Screens folder must be expanded by default");

  // 2. Both forms must be in tree as screen nodes
  const screenNodes = synced.tree?.filter((n) => n.kind === "screen");
  assert.equal(screenNodes?.length, 2, "Must have 2 screen nodes");
  assert.equal(screenNodes?.[0].ref_id, "form_main");
  assert.equal(screenNodes?.[0].name, "Main");
  assert.equal(screenNodes?.[1].ref_id, "form_2");
  assert.equal(screenNodes?.[1].name, "Detail");
});

test("ensureProjectTree deduplicates duplicate root system folders and merges screens under single canonical Screens folder", () => {
  // Exact scenario from user's corrupt project file: 4 root "Screens" folders accumulated
  const rawProject = {
    schema_version: 3,
    id: "proj_duplicate_screens",
    name: "Duplicate Screens Test",
    description: "",
    devices: [],
    tags: [],
    forms: [
      { id: "form_main", name: "Main", width: 1040, height: 700, background: "#ffffff", grid: 8, widgets: [] },
      { id: "form_screen_2", name: "Screen_2", width: 1040, height: 700, background: "#ffffff", grid: 8, widgets: [] },
    ],
    alarms: [],
    tree: [
      { id: "screens_1", parent_id: null, kind: "folder" as const, name: "Screens", order: 0, collapsed: true },
      { id: "screens_2", parent_id: null, kind: "folder" as const, name: "Screens", order: 0, collapsed: true },
      { id: "screens_3", parent_id: null, kind: "folder" as const, name: "Screens", order: 0, collapsed: true },
      { id: "screens_4", parent_id: null, kind: "folder" as const, name: "Screens", order: 0, collapsed: true },
      // Main screen was under screens_4
      { id: "scr_main", parent_id: "screens_4", kind: "screen" as const, name: "Main", order: 0, ref_id: "form_main" },
      // Screen_2 was under screens_1
      { id: "scr_s2", parent_id: "screens_1", kind: "screen" as const, name: "Screen_2", order: 1, ref_id: "form_screen_2" },
    ],
    content_hash: "",
  };

  const synced = ensureProjectTree(rawProject);

  // 1. Only ONE "Screens" folder must remain
  const screensFolders = synced.tree?.filter((n) => n.kind === "folder" && n.name === "Screens");
  assert.equal(screensFolders?.length, 1, "Must have exactly ONE Screens folder after deduplication");
  assert.equal(screensFolders?.[0].collapsed, false, "Screens folder must be expanded by default");

  const canonicalScreensId = screensFolders?.[0].id;

  // 2. Both screen nodes must be re-parented to the single canonical Screens folder
  const mainNode = synced.tree?.find((n) => n.kind === "screen" && n.ref_id === "form_main");
  const screen2Node = synced.tree?.find((n) => n.kind === "screen" && n.ref_id === "form_screen_2");

  assert.ok(mainNode, "Main screen node must exist");
  assert.ok(screen2Node, "Screen_2 screen node must exist");
  assert.equal(mainNode?.parent_id, canonicalScreensId, "Main screen must be under canonical Screens folder");
  assert.equal(screen2Node?.parent_id, canonicalScreensId, "Screen_2 must be under canonical Screens folder");
});

test("ensureProjectTree re-parents orphaned screen nodes to Screens folder", () => {
  const rawProject = {
    schema_version: 3,
    id: "proj_orphan",
    name: "Test Orphan Fix",
    description: "",
    devices: [],
    tags: [],
    forms: [
      { id: "form_main", name: "Main", width: 1040, height: 700, background: "#ffffff", grid: 8, widgets: [] },
    ],
    alarms: [],
    tree: [
      { id: "screens_fld", parent_id: null, kind: "folder" as const, name: "Screens", order: 0, collapsed: false },
      // Screen node with orphaned parent_id pointing to non-existent folder
      { id: "scr_main", parent_id: "non_existent_folder", kind: "screen" as const, name: "Main", order: 0, ref_id: "form_main" },
    ],
    content_hash: "",
  };

  const synced = ensureProjectTree(rawProject);
  const screensFolder = synced.tree?.find((n) => n.kind === "folder" && n.name === "Screens");
  const mainNode = synced.tree?.find((n) => n.kind === "screen" && n.ref_id === "form_main");

  assert.ok(screensFolder, "Screens folder must exist");
  assert.ok(mainNode, "Main screen node must exist");
  // Orphaned parent_id must be fixed to point to the Screens folder
  assert.equal(mainNode?.parent_id, screensFolder?.id, "Orphaned screen must be re-parented to Screens folder");
});

test("ensureProjectTree re-parents screens below unreachable folder chains", () => {
  const rawProject = {
    schema_version: 3,
    id: "proj_nested_orphan",
    name: "Test Nested Orphan Fix",
    description: "",
    devices: [],
    tags: [],
    forms: [
      { id: "form_main", name: "Main", width: 1040, height: 700, background: "#ffffff", grid: 8, widgets: [] },
    ],
    alarms: [],
    tree: [
      { id: "screens_fld", parent_id: null, kind: "folder" as const, name: "Screens", order: 0, collapsed: false },
      { id: "orphan_fld", parent_id: "missing_fld", kind: "folder" as const, name: "Legacy", order: 0 },
      { id: "scr_main", parent_id: "orphan_fld", kind: "screen" as const, name: "Main", order: 0, ref_id: "form_main" },
    ],
    content_hash: "",
  };

  const synced = ensureProjectTree(rawProject);
  const mainNode = synced.tree?.find((node) => node.kind === "screen" && node.ref_id === "form_main");

  assert.equal(mainNode?.parent_id, "screens_fld", "Screen below an unreachable folder must remain visible under Screens");
});

test("ensureProjectTree preserves valid parent_id for screens in sub-folders under Screens", () => {
  const rawProject = {
    schema_version: 3,
    id: "proj_subfolder",
    name: "Test Sub-folder",
    description: "",
    devices: [],
    tags: [],
    forms: [
      { id: "form_main", name: "Main", width: 1040, height: 700, background: "#ffffff", grid: 8, widgets: [] },
    ],
    alarms: [],
    tree: [
      { id: "screens_fld", parent_id: null, kind: "folder" as const, name: "Screens", order: 0, collapsed: false },
      { id: "sub_fld", parent_id: "screens_fld", kind: "folder" as const, name: "SubFolder", order: 0 },
      // Screen under a valid sub-folder inside Screens
      { id: "scr_main", parent_id: "sub_fld", kind: "screen" as const, name: "Main", order: 0, ref_id: "form_main" },
    ],
    content_hash: "",
  };

  const synced = ensureProjectTree(rawProject);
  const mainNode = synced.tree?.find((n) => n.kind === "screen" && n.ref_id === "form_main");

  assert.ok(mainNode, "Main screen node must exist");
  // Valid parent_id must be preserved
  assert.equal(mainNode?.parent_id, "sub_fld", "Valid parent_id under Screens must be preserved");
});

test("childrenOf and ensureProjectTree handle undefined or omitted parent_id on root nodes", () => {
  const rawProject = {
    schema_version: 3,
    id: "proj_undefined_parent",
    name: "Test Undefined Parent",
    description: "",
    devices: [],
    tags: [],
    forms: [
      { id: "form_main", name: "Main", width: 1040, height: 700, background: "#ffffff", grid: 8, widgets: [] },
    ],
    alarms: [],
    tree: [
      { id: "screens_fld", kind: "folder" as const, name: "Screens", order: 0, collapsed: false }, // parent_id omitted!
      { id: "scripts_fld", kind: "folder" as const, name: "Scripts", order: 1 }, // parent_id omitted!
      { id: "styles_fld", kind: "folder" as const, name: "Styles", order: 2 }, // parent_id omitted!
      { id: "images_fld", kind: "folder" as const, name: "Images", order: 3 }, // parent_id omitted!
      { id: "docs_fld", kind: "folder" as const, name: "Documents", order: 4 }, // parent_id omitted!
      { id: "var_node", kind: "variables" as const, name: "Variables", order: 5 }, // parent_id omitted!
      { id: "scr_main", parent_id: "screens_fld", kind: "screen" as const, name: "Main", order: 0, ref_id: "form_main" },
    ],
    content_hash: "",
  };

  const synced = ensureProjectTree(rawProject as unknown as import("../types").ScadaProject);
  assert.ok(synced.tree, "Synced tree must exist");

  const rootChildren = childrenOf(synced.tree, null);
  // The six seeded folders plus the guaranteed root "Komponenty" folder.
  assert.equal(
    rootChildren.length,
    7,
    "Must return every root item, including the guaranteed Components folder",
  );
  assert.ok(
    rootChildren.some((child) => child.kind === "components_folder"),
    "ensureProjectTree must guarantee a root Components folder",
  );

  for (const child of rootChildren) {
    assert.equal(child.parent_id, null, `Root item ${child.name} must have parent_id normalized to null`);
  }
});

test("import validation preserves the incoming content hash until engine verification", () => {
  const incoming = {
    schema_version: 3,
    id: "signed-project",
    name: "Signed Project",
    description: "",
    devices: [],
    tags: [],
    forms: [],
    alarms: [],
    tree: [],
    content_hash: "sha256-from-file",
  };

  assert.equal(validateImportedProjectEnvelope(incoming).content_hash, "sha256-from-file");
  const normalized = normalizeImportedProject(incoming);
  assert.equal(normalized.content_hash, "sha256-from-file");
  assert.equal(incoming.content_hash, "sha256-from-file", "normalization must not mutate input");
});
