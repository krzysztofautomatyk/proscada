import assert from "node:assert/strict";
import { test } from "node:test";
import { isMainScreen, canDeleteForm, canRenameForm, ensureMainFormExists } from "./screenProtection";

test("screenProtection correctly identifies Main screens", () => {
  assert.equal(isMainScreen({ id: "1", name: "Main" }), true);
  assert.equal(isMainScreen({ id: "2", name: "main_synoptic" }), true);
  assert.equal(isMainScreen({ id: "3", name: "Main.form" }), true);
  assert.equal(isMainScreen({ id: "4", name: "Overview" }), false);
});

test("isMainScreen works with partial objects for name reservation", () => {
  assert.equal(isMainScreen({ id: "", name: "Main" }), true);
  assert.equal(isMainScreen({ id: "", name: "main" }), true);
  assert.equal(isMainScreen({ id: "", name: " Main " }), true);
  assert.equal(isMainScreen({ id: "", name: "Screen_2" }), false);
});

test("canRenameForm blocks Main screen rename", () => {
  assert.equal(canRenameForm({ id: "1", name: "Main" }), false);
  assert.equal(canRenameForm({ id: "2", name: "Overview" }), true);
});

test("screenProtection prevents deletion of Main screen and single screen", () => {
  const forms = [
    { id: "main_id", name: "Main", width: 1040, height: 700, background: "#fff", grid: 8, widgets: [] },
    { id: "screen_2", name: "Alarms", width: 1040, height: 700, background: "#fff", grid: 8, widgets: [] },
  ];

  // Cannot delete Main screen
  assert.equal(canDeleteForm("main_id", forms), false);

  // Can delete secondary screen
  assert.equal(canDeleteForm("screen_2", forms), true);

  // Cannot delete when only 1 screen remains
  assert.equal(canDeleteForm("screen_2", [forms[1]]), false);
});

test("ensureMainFormExists guarantees Main screen is created if missing", () => {
  const emptyForms = ensureMainFormExists([]);
  assert.equal(emptyForms.length, 1);
  assert.equal(emptyForms[0].name, "Main");

  const secondaryForms = [
    { id: "sec1", name: "Overview", width: 1040, height: 700, background: "#fff", grid: 8, widgets: [] },
  ];
  const fixedForms = ensureMainFormExists(secondaryForms);
  assert.equal(fixedForms.length, 2);
  assert.equal(fixedForms[0].name, "Main");
});
