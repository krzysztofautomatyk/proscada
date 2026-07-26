import { writable } from "svelte/store";
import type { AppSettings } from "$lib/types";

const SETTINGS_KEY = "proscada.app.settings";

const defaultSettings: AppSettings = {
  autosaveEnabled: true,
  autosaveIntervalMinutes: 10,
  autosaveOnlyIfNoError: true,
  lastAutosaveTs: null,
  lastAutosaveStatus: null,
  showStartWindowOnStart: true,
};

function loadSettings(): AppSettings {
  if (typeof localStorage === "undefined") return { ...defaultSettings };
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (!raw) return { ...defaultSettings };
    const parsed = JSON.parse(raw) as Partial<AppSettings>;
    return {
      autosaveEnabled: parsed.autosaveEnabled ?? true,
      autosaveIntervalMinutes: Number(parsed.autosaveIntervalMinutes) || 10,
      autosaveOnlyIfNoError: parsed.autosaveOnlyIfNoError ?? true,
      lastAutosaveTs: parsed.lastAutosaveTs ?? null,
      lastAutosaveStatus: parsed.lastAutosaveStatus ?? null,
      showStartWindowOnStart: parsed.showStartWindowOnStart ?? true,
    };
  } catch {
    return { ...defaultSettings };
  }
}

export const appSettings = writable<AppSettings>(loadSettings());

appSettings.subscribe((val) => {
  if (typeof localStorage !== "undefined") {
    try {
      localStorage.setItem(SETTINGS_KEY, JSON.stringify(val));
    } catch {
      /* ignore */
    }
  }
});

export function updateAppSettings(patch: Partial<AppSettings>) {
  appSettings.update((s) => ({ ...s, ...patch }));
}
