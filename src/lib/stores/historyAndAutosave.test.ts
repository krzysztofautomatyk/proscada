import assert from "node:assert/strict";
import { beforeEach, test } from "node:test";
import { get } from "svelte/store";
import type { ScadaProject } from "$lib/types";
import { ensureProjectTree } from "$lib/utils/projectTree";
import { validateProject } from "$lib/utils/validation";
import {
  clearHistory,
  recordHistoryState,
  performUndo,
  performRedo,
  canUndo,
  canRedo,
  undoStack,
  redoStack,
} from "$lib/stores/history";
import { appSettings, updateAppSettings } from "$lib/stores/settings";
import {
  sanitizeRecentItems,
  recordRecentProject,
  recentProjects,
  clearRecentProjects,
  type RecentProjectItem,
} from "$lib/stores/recentProjects";

function baseTestProject(): ScadaProject {
  return {
    schema_version: 3,
    id: "test-proj",
    name: "Test Project",
    description: "",
    devices: [
      {
        id: "dev-1",
        name: "Test Device",
        host: "127.0.0.1",
        port: 502,
        unit_id: 1,
        poll_ms: 500,
        timeout_ms: 1000,
        enabled: true,
      },
    ],
    tags: [
      {
        id: "T1",
        name: "Temperature",
        device_id: "dev-1",
        data_type: "u16",
        binding: { address: 100, table: "holding" },
        unit: "C",
        description: "",
        scale: 1,
        offset: 0,
        decimals: 1,
      },
    ],
    forms: [
      {
        id: "main-screen",
        name: "MainScreen",
        width: 1000,
        height: 600,
        background: "#ffffff",
        grid: 8,
        widgets: [
          {
            id: "w1",
            widget_type: "numeric",
            x: 50,
            y: 50,
            w: 120,
            h: 60,
            z: 1,
            config: {},
          },
        ],
      },
    ],
    alarms: [],
    alarm_groups: [],
    component_templates: [],
    tree: [],
    content_hash: "hash123",
  };
}

beforeEach(() => {
  clearHistory();
  updateAppSettings({
    autosaveEnabled: true,
    autosaveIntervalMinutes: 10,
    autosaveOnlyIfNoError: true,
    lastAutosaveTs: null,
    lastAutosaveStatus: null,
  });
});

test("ensureProjectTree always creates a root Styles folder", () => {
  const p = baseTestProject();
  const normalized = ensureProjectTree(p);
  const hasStyles = normalized.tree?.some(
    (node) => node.kind === "folder" && node.parent_id === null && node.name.toLowerCase() === "styles",
  );
  assert.equal(hasStyles, true, "Root Styles folder must exist");
});

test("Undo / Redo history records and reverts changes correctly", () => {
  const p1 = baseTestProject();
  assert.equal(get(canUndo), false);
  assert.equal(get(canRedo), false);

  // Record initial state before mutation
  recordHistoryState(p1, "Add Widget", "main-screen", ["w1"], true);
  assert.equal(get(canUndo), true);
  assert.equal(get(undoStack).length, 1);

  // Mutate project state (simulate user adding a widget)
  const p2: ScadaProject = {
    ...p1,
    forms: [
      {
        ...p1.forms[0],
        widgets: [
          ...p1.forms[0].widgets,
          { id: "w2", widget_type: "button", x: 200, y: 200, w: 100, h: 40, z: 2, config: {} },
        ],
      },
    ],
  };

  // Perform Undo
  const undoTarget = performUndo(p2, "main-screen", ["w2"]);
  assert.ok(undoTarget);
  assert.equal(undoTarget.project.forms[0].widgets.length, 1, "Undone state should have 1 widget");
  assert.equal(get(canUndo), false);
  assert.equal(get(canRedo), true);

  // Perform Redo
  const redoTarget = performRedo(undoTarget.project, "main-screen", ["w1"]);
  assert.ok(redoTarget);
  assert.equal(redoTarget.project.forms[0].widgets.length, 2, "Redone state should have 2 widgets");
  assert.equal(get(canUndo), true);
  assert.equal(get(canRedo), false);
});

test("validateProject detects clean project vs broken project", () => {
  const clean = baseTestProject();
  const cleanResult = validateProject(clean);
  assert.equal(cleanResult.valid, true, "Clean project should pass validation");
  assert.equal(cleanResult.errors.length, 0);

  const broken: ScadaProject = {
    ...clean,
    devices: [
      {
        id: "dev-broken",
        name: "",
        host: "", // Missing host
        port: 999999, // Invalid port
        unit_id: 300, // Invalid unit ID
        poll_ms: 100,
        timeout_ms: 1000,
        enabled: true,
      },
    ],
    forms: [], // Empty forms array
  };

  const brokenResult = validateProject(broken);
  assert.equal(brokenResult.valid, false, "Broken project should fail validation");
  assert.ok(brokenResult.errors.length >= 3, "Should report multiple validation errors");
});

test("AppSettings store persists toggleable AutoSave settings", () => {
  updateAppSettings({ autosaveEnabled: false, autosaveIntervalMinutes: 15, showStartWindowOnStart: false });
  const settings = get(appSettings);
  assert.equal(settings.autosaveEnabled, false, "AutoSave should be disabled when toggled off");
  assert.equal(settings.autosaveIntervalMinutes, 15);
  assert.equal(settings.showStartWindowOnStart, false);
});

test("sanitizeRecentItems cleans corrupt inputs and records recent projects", () => {
  clearRecentProjects();

  const sanitized = sanitizeRecentItems([
    null,
    { id: "p1", name: "Project One" },
    { invalid: true },
  ]);
  assert.equal(sanitized.length, 1);
  assert.equal(sanitized[0].id, "p1");

  recordRecentProject({ id: "p2", name: "Project Two", description: "Desc" });
  const list: RecentProjectItem[] = get(recentProjects);
  assert.equal(list.length, 1);
  assert.equal(list[0].id, "p2");
});
