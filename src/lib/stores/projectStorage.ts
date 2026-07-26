import { writable, get } from "svelte/store";
import type { ScadaProject } from "$lib/types";
import { saveTextFile, openTextFile, writeDirectTextFile, readDirectTextFile } from "$lib/services/fileIo";
import { recordRecentProject } from "$lib/stores/recentProjects";
import { createEmptyProject, normalizeImportedProject } from "$lib/utils/projectTree";
import { api } from "$lib/services/api";

const STORAGE_KEY = "proscada.active.project.path";

function loadInitialPath(): string | null {
  if (typeof localStorage === "undefined") return null;
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

export const activeProjectPath = writable<string | null>(loadInitialPath());

activeProjectPath.subscribe((val) => {
  if (typeof localStorage !== "undefined") {
    try {
      if (val) localStorage.setItem(STORAGE_KEY, val);
      else localStorage.removeItem(STORAGE_KEY);
    } catch {
      /* ignore */
    }
  }
});

export function setActiveProjectPath(path: string | null) {
  activeProjectPath.set(path);
}

/** Save active project to disk file. If path is already known, writes directly; otherwise prompts save dialog. */
export async function saveProjectToDisk(projectObj: ScadaProject, forceDialog = false): Promise<{ ok: boolean; path?: string }> {
  const jsonText = JSON.stringify(projectObj, null, 2);
  let currentPath = get(activeProjectPath);

  if (currentPath && !forceDialog) {
    const directOk = await writeDirectTextFile(currentPath, jsonText);
    if (directOk) {
      recordRecentProject({
        id: projectObj.id,
        name: projectObj.name,
        path: currentPath,
        description: projectObj.description,
      });
      return { ok: true, path: currentPath };
    }
  }

  // Prompt native save file dialog
  const defaultName = `${(projectObj.name || projectObj.id || "project").replace(/[^a-z0-9_-]+/gi, "_")}.proscada.json`;
  const savedPath = await saveTextFile(defaultName, jsonText, [
    { name: "ProScada Project", extensions: ["proscada.json", "json"] },
  ]);

  if (!savedPath) return { ok: false };

  activeProjectPath.set(savedPath);
  recordRecentProject({
    id: projectObj.id,
    name: projectObj.name,
    path: savedPath,
    description: projectObj.description,
  });

  return { ok: true, path: savedPath };
}

/** Create a new project and IMMEDIATELY prompt for file location on disk and save it. */
export async function createAndSaveNewProject(
  name: string,
  description = "",
  customProject?: ScadaProject,
): Promise<{ project: ScadaProject; path: string | null } | null> {
  const p = customProject || createEmptyProject(name || "New Project");
  if (description && !customProject) p.description = description;

  const jsonText = JSON.stringify(p, null, 2);
  const defaultName = `${name.replace(/[^a-z0-9_-]+/gi, "_")}.proscada.json`;

  const savedPath = await saveTextFile(defaultName, jsonText, [
    { name: "ProScada Project", extensions: ["proscada.json", "json"] },
  ]);

  if (savedPath) {
    activeProjectPath.set(savedPath);
    recordRecentProject({
      id: p.id,
      name: p.name,
      path: savedPath,
      description: p.description,
    });
    return { project: p, path: savedPath };
  }

  // Fallback: If user cancelled file save dialog, still return project with null path
  activeProjectPath.set(null);
  recordRecentProject({
    id: p.id,
    name: p.name,
    description: p.description,
  });
  return { project: p, path: null };
}

/** Open/read project file directly from disk path. */
export async function openProjectFromDisk(filePath?: string): Promise<{ project: ScadaProject; path: string } | null> {
  let targetPath = filePath;
  let textContent: string | null = null;

  if (targetPath) {
    textContent = await readDirectTextFile(targetPath);
  }

  if (!targetPath || textContent === null) {
    const picked = await openTextFile([
      { name: "ProScada Project", extensions: ["proscada.json", "json"] },
    ]);
    if (!picked) return null;
    targetPath = picked.path;
    textContent = picked.text;
  }

  try {
    const raw = JSON.parse(textContent);
    const normalized = normalizeImportedProject(raw);
    activeProjectPath.set(targetPath);
    recordRecentProject({
      id: normalized.id,
      name: normalized.name,
      path: targetPath,
      description: normalized.description,
    });
    return { project: normalized, path: targetPath };
  } catch (e) {
    console.error("Failed to parse project file:", e);
    return null;
  }
}
