import assert from "node:assert/strict";
import { test } from "node:test";
import { get } from "svelte/store";
import { activeProjectPath, setActiveProjectPath } from "./projectStorage";

test("projectStorage manages active project path state", () => {
  setActiveProjectPath(null);
  assert.equal(get(activeProjectPath), null);

  setActiveProjectPath("/path/to/my_station.proscada.json");
  assert.equal(get(activeProjectPath), "/path/to/my_station.proscada.json");

  setActiveProjectPath(null);
  assert.equal(get(activeProjectPath), null);
});
