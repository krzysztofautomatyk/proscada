import assert from "node:assert/strict";
import { test } from "node:test";
import { get } from "svelte/store";
import {
  recentProjects,
  recordRecentProject,
  togglePinRecentProject,
  removeRecentProject,
  clearRecentProjects,
  sanitizeRecentItems,
} from "./recentProjects";

test("recentProjects sanitizes invalid/corrupt storage items", () => {
  const corruptData = [
    null,
    123,
    "string",
    {},
    { id: "proj_1", name: "Valid Project", pinned: true },
    { id: "", name: "No ID" },
  ];
  const sanitized = sanitizeRecentItems(corruptData);
  assert.equal(sanitized.length, 1);
  assert.equal(sanitized[0].id, "proj_1");
  assert.equal(sanitized[0].name, "Valid Project");
  assert.equal(sanitized[0].pinned, true);
});

test("recentProjects records, pins, removes and clears project history", () => {
  clearRecentProjects();
  assert.equal(get(recentProjects).length, 0);

  recordRecentProject({
    id: "p1",
    name: "Station Alpha",
    description: "Main Water Tank",
  });
  recordRecentProject({
    id: "p2",
    name: "Station Beta",
    description: "Secondary Tank",
  });

  let items = get(recentProjects);
  assert.equal(items.length, 2);
  assert.equal(items[0].id, "p2"); // newest first

  // Pin p1
  togglePinRecentProject("p1");
  items = get(recentProjects);
  assert.equal(items.find((i) => i.id === "p1")?.pinned, true);

  // Remove p2
  removeRecentProject("p2");
  items = get(recentProjects);
  assert.equal(items.length, 1);
  assert.equal(items[0].id, "p1");

  // Clear all
  clearRecentProjects();
  assert.equal(get(recentProjects).length, 0);
});
