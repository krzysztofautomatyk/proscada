import { writable } from "svelte/store";

export interface RecentProjectItem {
  id: string;
  name: string;
  path?: string;
  description?: string;
  lastOpened: string; // ISO string
  pinned: boolean;
}

const RECENT_PROJECTS_KEY = "proscada.recent.projects";

function loadRecentProjects(): RecentProjectItem[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(RECENT_PROJECTS_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return sanitizeRecentItems(parsed);
  } catch {
    return [];
  }
}

export const recentProjects = writable<RecentProjectItem[]>(loadRecentProjects());

recentProjects.subscribe((val) => {
  if (typeof localStorage !== "undefined") {
    try {
      localStorage.setItem(RECENT_PROJECTS_KEY, JSON.stringify(val));
    } catch {
      /* ignore */
    }
  }
});

export function recordRecentProject(projectInfo: {
  id: string;
  name: string;
  path?: string;
  description?: string;
}) {
  recentProjects.update((items) => {
    const existingIndex = items.findIndex((i) => i.id === projectInfo.id);
    const existing = existingIndex >= 0 ? items[existingIndex] : null;
    const isPinned = existing ? existing.pinned : false;

    const updatedItem: RecentProjectItem = {
      id: projectInfo.id,
      name: projectInfo.name || "Untitled Project",
      path: projectInfo.path || existing?.path,
      description: projectInfo.description || existing?.description,
      lastOpened: new Date().toISOString(),
      pinned: isPinned,
    };

    const remaining = items.filter((i) => i.id !== projectInfo.id);
    return [updatedItem, ...remaining].slice(0, 50);
  });
}

export function sanitizeRecentItems(parsed: unknown): RecentProjectItem[] {
  if (!Array.isArray(parsed)) return [];
  return parsed
    .filter((item): item is Record<string, unknown> => Boolean(item) && typeof item === "object")
    .map((item) => ({
      id: String(item.id || ""),
      name: String(item.name || "Projekt bez nazwy"),
      path: item.path ? String(item.path) : undefined,
      description: item.description ? String(item.description) : undefined,
      lastOpened: item.lastOpened && typeof item.lastOpened === "string" ? item.lastOpened : new Date().toISOString(),
      pinned: Boolean(item.pinned),
    }))
    .filter((item) => item.id.length > 0);
}

export async function openRecentProjectItem(item: RecentProjectItem): Promise<boolean> {
  const { importProjectFromJson, importProjectFile, project, log } = await import("./app");
  const { api } = await import("$lib/services/api");
  const { ensureProjectTree } = await import("$lib/utils/projectTree");

  if (item.id === "water_tank_dual_pump" || item.name.includes("Water Tank")) {
    const p = await api.loadBuiltinWaterTank();
    project.set(ensureProjectTree(p));
    log("Załadowano wbudowany projekt Water Tank", "ok");
    return true;
  }

  const isTauri =
    typeof window !== "undefined" &&
    ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

  if (isTauri && item.path) {
    try {
      const { readTextFile } = await import("@tauri-apps/plugin-fs");
      const content = await readTextFile(item.path);
      if (content) {
        await importProjectFromJson(content);
        log(`Załadowano projekt ze ścieżki: ${item.path}`, "ok");
        return true;
      }
    } catch {
      log(`Plik ${item.path} nie istnieje lub został przeniesiony. Wybierz plik z dysku.`, "warn");
    }
  }

  // Fallback to dialog picker
  await importProjectFile();
  return true;
}

export function togglePinRecentProject(id: string) {
  recentProjects.update((items) =>
    items.map((i) => (i.id === id ? { ...i, pinned: !i.pinned } : i)),
  );
}

export function removeRecentProject(id: string) {
  recentProjects.update((items) => items.filter((i) => i.id !== id));
}

export function clearRecentProjects() {
  recentProjects.set([]);
}
