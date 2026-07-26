import { writable, derived, get } from "svelte/store";
import type {
  EngineSnapshot,
  ProjectNode,
  ProjectNodeKind,
  ScadaProject,
  WidgetDef,
  FormDef,
  AuditEntry,
  AlarmDefinition,
  AlarmGroupDefinition,
  ComponentTemplate,
  DeviceConfig,
  ProjectDesignSystem,
  TagDefinition,
} from "$lib/types";
import { WIDGET_CATALOG } from "$lib/components/widgets/registry";
import { getWidgetCatalogItem } from "$lib/components/widgets/registry";
import { defaultDynamicsConfig } from "$lib/utils/dynamics";
import { api } from "$lib/services/api";
import {
  collectDescendantIds,
  createEmptyProject,
  defaultContent,
  defaultExt,
  ensureProjectTree,
  findNode,
  isAncestor,
  nextOrder,
  normalizeImportedProject,
  uid,
} from "$lib/utils/projectTree";
import { expandSelectionWithGroups } from "$lib/stores/selection";
import {
  recordHistoryState,
  performUndo,
  performRedo,
  canUndo,
  canRedo,
  undoLabel,
  redoLabel,
  clearHistory,
} from "$lib/stores/history";
import { appSettings, updateAppSettings } from "$lib/stores/settings";
import { validateProject } from "$lib/utils/validation";
import { recordRecentProject } from "$lib/stores/recentProjects";
import {
  activeProjectPath,
  setActiveProjectPath,
  saveProjectToDisk,
  createAndSaveNewProject,
  openProjectFromDisk,
} from "$lib/stores/projectStorage";
import { canDeleteForm, isMainScreen } from "$lib/utils/screenProtection";

export { canUndo, canRedo, undoLabel, redoLabel, activeProjectPath, isMainScreen, canDeleteForm };

export type AppMode = "designer" | "runtime";

export const project = writable<ScadaProject | null>(null);
export const snapshot = writable<EngineSnapshot | null>(null);
export const mode = writable<AppMode>("designer");
export const selectedWidgetId = writable<string | null>(null);
export const selectedWidgetIds = writable<string[]>([]);
export const selectedFormId = writable<string | null>(null);
/** Solution Explorer selection (folder / screen / script / note / …). */
export const selectedNodeId = writable<string | null>(null);
export interface DeviceModalState {
  open: boolean;
  mode: "add" | "edit";
  deviceId?: string;
}

export const startWindowOpen = writable(false);
export const deviceModalState = writable<DeviceModalState>({ open: false, mode: "add" });
export const addDeviceModalOpen = writable(false);
export const addAlarmModalOpen = writable(false);
export const addVariableModalOpen = writable(false);
export const logs = writable<{ t: string; level: "info" | "ok" | "warn" | "err"; msg: string }[]>(
  [],
);
export const audit = writable<AuditEntry[]>([]);
export const dirty = writable(false);

/** Designer clipboard (in-app; not OS clipboard). */
export const clipboard = writable<WidgetDef[]>([]);
/** Request Properties panel focus (Attributes). */
export const focusPropertiesTick = writable(0);

function cloneWidgetDeep(w: WidgetDef): WidgetDef {
  return JSON.parse(JSON.stringify(w)) as WidgetDef;
}

function selectionIds(): string[] {
  const ids = get(selectedWidgetIds);
  const single = get(selectedWidgetId);
  const set = new Set([...ids, ...(single ? [single] : [])]);
  return Array.from(set);
}

function widgetsFromSelection(form: FormDef): WidgetDef[] {
  const ids = new Set(selectionIds());
  return form.widgets.filter((w) => ids.has(w.id));
}

export function log(msg: string, level: "info" | "ok" | "warn" | "err" = "info") {
  const t = new Date().toLocaleTimeString();
  logs.update((xs) => [{ t, level, msg }, ...xs].slice(0, 500));
}

export const activeForm = derived([project, selectedFormId], ([$p, $id]) => {
  if (!$p) return null;
  if ($id) return $p.forms.find((f) => f.id === $id) ?? $p.forms[0] ?? null;
  return $p.forms[0] ?? null;
});

export const selectedWidget = derived(
  [activeForm, selectedWidgetId],
  ([$form, $id]) => {
    if (!$form || !$id) return null;
    return $form.widgets.find((w) => w.id === $id) ?? null;
  },
);

export const tagMap = derived(snapshot, ($s) => {
  const m = new Map<string, (typeof $s extends null ? never : NonNullable<typeof $s>)["tags"][0]>();
  if ($s) for (const t of $s.tags) m.set(t.tag_id, t);
  return m;
});

function recordUndo(label = "Edit", forceNewStep = false) {
  const p = get(project);
  if (p) {
    recordHistoryState(p, label, get(selectedFormId), selectionIds(), forceNewStep);
  }
}

export function undoAction() {
  const curP = get(project);
  const curFormId = get(selectedFormId);
  const curWidgets = selectionIds();
  const snapshot = performUndo(curP, curFormId, curWidgets);
  if (snapshot) {
    project.set(ensureProjectTree(snapshot.project));
    if (snapshot.selectedFormId) selectedFormId.set(snapshot.selectedFormId);
    setSelection(snapshot.selectedWidgetIds);
    dirty.set(true);
    log(`Undo: ${snapshot.actionLabel}`, "info");
  }
}

export function redoAction() {
  const curP = get(project);
  const curFormId = get(selectedFormId);
  const curWidgets = selectionIds();
  const snapshot = performRedo(curP, curFormId, curWidgets);
  if (snapshot) {
    project.set(ensureProjectTree(snapshot.project));
    if (snapshot.selectedFormId) selectedFormId.set(snapshot.selectedFormId);
    setSelection(snapshot.selectedWidgetIds);
    dirty.set(true);
    log(`Redo: ${snapshot.actionLabel}`, "info");
  }
}

let pollTimer: ReturnType<typeof setInterval> | null = null;
let autosaveTimer: ReturnType<typeof setInterval> | null = null;
let lastAutosaveCheckTime = Date.now();

export function startAutosaveLoop() {
  if (autosaveTimer) clearInterval(autosaveTimer);
  autosaveTimer = setInterval(async () => {
    const settings = get(appSettings);
    if (!settings.autosaveEnabled) return;

    const intervalMs = (settings.autosaveIntervalMinutes || 10) * 60 * 1000;
    const now = Date.now();
    if (now - lastAutosaveCheckTime < intervalMs) return;

    lastAutosaveCheckTime = now;
    const isDirty = get(dirty);
    const p = get(project);

    if (!isDirty || !p) return;

    // Validate project before auto-saving
    const validation = validateProject(p);
    const timeStr = new Date().toLocaleTimeString();

    if (settings.autosaveOnlyIfNoError && !validation.valid) {
      updateAppSettings({
        lastAutosaveTs: timeStr,
        lastAutosaveStatus: "skipped_errors",
      });
      log(`AutoSave skipped (${timeStr}): project has ${validation.errors.length} error(s)`, "warn");
      return;
    }

    const ok = await persistProject();
    if (ok) {
      updateAppSettings({
        lastAutosaveTs: timeStr,
        lastAutosaveStatus: "ok",
      });
      log(`AutoSave (${timeStr}): project auto-saved successfully (0 errors)`, "ok");
    } else {
      updateAppSettings({
        lastAutosaveTs: timeStr,
        lastAutosaveStatus: "error",
      });
      log(`AutoSave (${timeStr}): save failed`, "err");
    }
  }, 15000); // Check every 15s
}

export function applyLoadedProject(p: ScadaProject, msg?: string, path?: string | null) {
  const normalized = ensureProjectTree(p);
  project.set(normalized);
  selectedFormId.set(normalized.forms[0]?.id ?? null);
  selectedNodeId.set(null);
  selectedWidgetId.set(null);
  selectedWidgetIds.set([]);
  dirty.set(false);
  clearHistory();
  if (path !== undefined) {
    setActiveProjectPath(path);
  }
  const currentPath = path ?? get(activeProjectPath) ?? undefined;
  recordRecentProject({
    id: normalized.id,
    name: normalized.name,
    path: currentPath,
    description: normalized.description,
  });
  if (msg) log(msg, "ok");
}

export async function initApp() {
  try {
    let p = await api.getProject();
    if (!p) {
      p = await api.loadBuiltinWaterTank();
      applyLoadedProject(p, "Loaded built-in Water Tank Dual-Pump project");
    } else {
      applyLoadedProject(p, `Project loaded: ${p.name}`);
    }
    if (get(appSettings).showStartWindowOnStart !== false) {
      startWindowOpen.set(true);
    }
    const chainOk = await api.verifyAudit();
    log(chainOk ? "Audit chain verified" : "Audit chain BROKEN", chainOk ? "ok" : "err");
    startUiPoll();
    startAutosaveLoop();
  } catch (e) {
    log(`Init error: ${e}`, "err");
    try {
      const p = await api.loadBuiltinWaterTank();
      applyLoadedProject(p);
      startUiPoll();
      startAutosaveLoop();
    } catch (e2) {
      log(`Fallback failed: ${e2}`, "err");
    }
  }
}

export function startUiPoll() {
  if (pollTimer) clearInterval(pollTimer);
  pollTimer = setInterval(async () => {
    try {
      const s = await api.getSnapshot();
      snapshot.set(s);
    } catch {
      /* ignore transient */
    }
  }, 200);
}

export async function connectDevice() {
  const p = get(project);
  if (!p) return;

  // Sync project definition with backend so the polling engine has updated device list
  try {
    await api.saveProject(p);
  } catch {
    /* fallback for mock */
  }

  const dev = p.devices.find((d) => d.enabled) ?? p.devices[0];
  try {
    const test = dev
      ? await api.testDevice(dev.host, dev.port, dev.unit_id, dev.timeout_ms)
      : { ok: false, message: "No device configured" };
    if (!test.ok) {
      log(`Device test failed: ${test.message}`, "warn");
    } else {
      log(`Device reachable ${dev?.host}:${dev?.port}`, "ok");
    }
    await api.startPolling(dev?.id);
    log(`Polling started → ${dev?.name ?? "device"} (${dev?.host}:${dev?.port})`, "ok");
  } catch (e) {
    log(`Connect error: ${e}`, "err");
  }
}

export async function disconnectDevice() {
  await api.stopPolling();
  log("Polling stopped", "warn");
}

export async function switchMode(m: AppMode) {
  if (m === "runtime" && get(dirty)) {
    const wasConnected = get(snapshot)?.connected ?? false;
    log("Saving project before Runtime so the engine receives the current tag map", "info");
    if (!(await persistProject())) {
      log("Runtime blocked: project save failed", "err");
      return;
    }
    if (wasConnected) await connectDevice();
  }
  mode.set(m);
  await api.setMode(m);
  const form = get(activeForm);
  const screen = form?.name ?? "(no screen)";
  if (m === "runtime") {
    log(`Runtime · running screen: ${screen}`, "ok");
  } else {
    log(`Designer · editing screen: ${screen}`, "info");
  }
}

export function updateWidget(patch: Partial<WidgetDef> & { id: string }) {
  recordUndo("Update Widget");
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      const target = f.widgets.find((w) => w.id === patch.id);
      if (!target) return f;

      // Pure geometry move (x/y only) → multi-select + group move together.
      // Config / tag / lock / resize MUST NOT go through this path.
      const keys = Object.keys(patch).filter((k) => k !== "id");
      const pureMove =
        keys.length > 0 &&
        keys.every((k) => k === "x" || k === "y") &&
        (patch.x !== undefined || patch.y !== undefined);

      if (pureMove) {
        const dx = (patch.x ?? target.x) - target.x;
        const dy = (patch.y ?? target.y) - target.y;
        if (dx === 0 && dy === 0) return f;

        const sel = new Set(selectionIds());
        sel.add(patch.id);
        const groups = new Set(
          f.widgets.filter((w) => sel.has(w.id) && w.group_id).map((w) => w.group_id!),
        );
        for (const w of f.widgets) {
          if (w.group_id && groups.has(w.group_id)) sel.add(w.id);
        }

        return {
          ...f,
          widgets: f.widgets.map((w) => {
            if (!sel.has(w.id) || w.locked) return w;
            if (w.id === patch.id) {
              return {
                ...w,
                x: patch.x !== undefined ? patch.x : w.x,
                y: patch.y !== undefined ? patch.y : w.y,
              };
            }
            return { ...w, x: w.x + dx, y: w.y + dy };
          }),
        };
      }

      // Config, lock, tag, resize, z, etc. — always merge onto the single target.
      // Locked widgets still accept property edits (text, colors, …).
      return {
        ...f,
        widgets: f.widgets.map((w) => {
          if (w.id !== patch.id) return w;
          const next: WidgetDef = { ...w, ...patch, id: w.id };
          if (patch.config) {
            next.config = { ...(w.config ?? {}), ...patch.config };
          }
          return next;
        }),
      };
    });
    dirty.set(true);
    return { ...p, forms };
  });
}

/** Explicit config patch (guaranteed property write). */
export function updateWidgetConfig(id: string, partial: Record<string, unknown>) {
  const form = get(activeForm);
  const w = form?.widgets.find((x) => x.id === id);
  if (!w) {
    log(`updateWidgetConfig: widget not found ${id}`, "err");
    return;
  }
  updateWidget({
    id,
    config: { ...(w.config ?? {}), ...partial },
  });
}

/**
 * Move many widgets from absolute origin snapshots (true multi-drag).
 * origins: id → starting x,y
 */
export function applyMultiMove(
  origins: Record<string, { x: number; y: number }>,
  dx: number,
  dy: number,
) {
  const ids = Object.keys(origins);
  if (ids.length === 0) return;
  recordUndo("Move Widgets");
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      const idSet = new Set(ids);
      // expand groups
      const groups = new Set(
        f.widgets.filter((w) => idSet.has(w.id) && w.group_id).map((w) => w.group_id!),
      );
      for (const w of f.widgets) {
        if (w.group_id && groups.has(w.group_id)) idSet.add(w.id);
      }
      return {
        ...f,
        widgets: f.widgets.map((w) => {
          if (!idSet.has(w.id) || w.locked) return w;
          const o = origins[w.id];
          if (o) return { ...w, x: o.x + dx, y: o.y + dy };
          // group member not in origins: keep relative — find any origin peer
          return w;
        }),
      };
    });
    dirty.set(true);
    return { ...p, forms };
  });
}

/** Select widgets by ids (replaces selection), expanding groups. */
export function setSelection(ids: string[], primaryId?: string | null) {
  const form = get(activeForm);
  if (!form) {
    selectedWidgetIds.set(ids);
    selectedWidgetId.set(primaryId ?? ids[0] ?? null);
    return;
  }
  const expanded = (() => {
    const set = new Set(ids);
    const groups = new Set(
      form.widgets.filter((w) => set.has(w.id) && w.group_id).map((w) => w.group_id!),
    );
    for (const w of form.widgets) {
      if (w.group_id && groups.has(w.group_id)) set.add(w.id);
    }
    return Array.from(set);
  })();
  selectedWidgetIds.set(expanded);
  selectedWidgetId.set(primaryId ?? expanded[0] ?? null);
}

export function toggleSelection(id: string) {
  const form = get(activeForm);
  if (!form) return;
  const w = form.widgets.find((x) => x.id === id);
  let next = new Set(selectionIds());
  if (next.has(id)) {
    // deselect id (+ its group)
    if (w?.group_id) {
      for (const x of form.widgets) {
        if (x.group_id === w.group_id) next.delete(x.id);
      }
    } else {
      next.delete(id);
    }
  } else {
    next.add(id);
    if (w?.group_id) {
      for (const x of form.widgets) {
        if (x.group_id === w.group_id) next.add(x.id);
      }
    }
  }
  const arr = Array.from(next);
  selectedWidgetIds.set(arr);
  selectedWidgetId.set(arr.includes(id) ? id : arr[0] ?? null);
}

export function generateUniqueWidgetId(): string {
  const ts = Date.now().toString(36);
  const rand = Math.random().toString(36).slice(2, 7);
  return `w_${ts}_${rand}`;
}

export function addWidget(w: WidgetDef) {
  recordUndo("Add Widget", true);
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      return { ...f, widgets: [...f.widgets, w] };
    });
    dirty.set(true);
    return { ...p, forms };
  });
  selectedWidgetId.set(w.id);
  selectedWidgetIds.set([w.id]);
}

export function addCatalogWidget(type: string, posX?: number, posY?: number) {
  console.log("[addCatalogWidget] called with type:", type);
  const cat = WIDGET_CATALOG.find((c) => c.type === type);
  if (!cat) { console.warn("[addCatalogWidget] type not found in catalog:", type); return; }
  const p = get(project);
  if (!p) { console.warn("[addCatalogWidget] project is null!"); return; }
  const formId = get(selectedFormId) ?? p.forms[0]?.id;
  console.log("[addCatalogWidget] formId:", formId, "forms:", p.forms.map(f => f.id));
  const activeFormObj = p.forms.find((f) => f.id === formId);
  if (!activeFormObj) { console.warn("[addCatalogWidget] no active form found!"); return; }

  const offset = (activeFormObj.widgets.length ?? 0) * 20;
  const x = posX !== undefined ? posX : Math.min(60 + (offset % 300), 650);
  const y = posY !== undefined ? posY : Math.min(60 + (offset % 220), 400);

  const id = generateUniqueWidgetId();
  console.log("[addCatalogWidget] adding widget id:", id, "at:", x, y);
  addWidget({
    id,
    widget_type: type,
    x,
    y,
    w: cat.defaultW,
    h: cat.defaultH,
    z: activeFormObj.widgets.length + 1,
    tag_id: null,
    group_id: null,
    locked: false,
    // Generic dynamics (blink / marquee / visibility) on every control
    config: { ...defaultDynamicsConfig(), ...cat.defaultConfig },
  });
  log(`Added widget: ${cat.label}`, "ok");
}

export function updateProjectDesignSystem(next: ProjectDesignSystem) {
  recordUndo("Update Design System", true);
  project.update((current) => {
    if (!current) return current;
    dirty.set(true);
    return { ...current, design_system: structuredClone(next) };
  });
  log("Project design system updated", "ok");
}

function selectedWidgetsForTemplate(): { form: FormDef; widgets: WidgetDef[] } | null {
  const form = get(activeForm);
  if (!form) return null;
  const widgets = widgetsFromSelection(form);
  return widgets.length > 0 ? { form, widgets } : null;
}

function stripUnsafeTemplateConfig(config: Record<string, unknown>): Record<string, unknown> {
  const clean: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(config)) {
    if (/script|javascript/i.test(key)) continue;
    if (typeof value === "string" && /^\s*javascript:/i.test(value)) continue;
    if (Array.isArray(value)) {
      clean[key] = value.map((item) =>
        item && typeof item === "object" && !Array.isArray(item)
          ? stripUnsafeTemplateConfig(item as Record<string, unknown>)
          : item,
      );
    } else if (value && typeof value === "object") {
      clean[key] = stripUnsafeTemplateConfig(value as Record<string, unknown>);
    } else {
      clean[key] = value;
    }
  }
  return clean;
}

export function createComponentTemplateFromSelection(
  name: string,
  category = "Custom",
): string | null {
  const selection = selectedWidgetsForTemplate();
  const trimmedName = name.trim();
  if (!selection || !trimmedName) {
    log("Select at least one widget and provide a component name", "warn");
    return null;
  }
  const minX = Math.min(...selection.widgets.map((widget) => widget.x));
  const minY = Math.min(...selection.widgets.map((widget) => widget.y));
  const maxX = Math.max(...selection.widgets.map((widget) => widget.x + widget.w));
  const maxY = Math.max(...selection.widgets.map((widget) => widget.y + widget.h));
  const template: ComponentTemplate = {
    id: uid("cmp"),
    name: trimmedName,
    category: category.trim() || "Custom",
    version: "1.0.0",
    description: `Created from ${selection.widgets.length} widget(s) on ${selection.form.name}`,
    width: Math.max(1, maxX - minX),
    height: Math.max(1, maxY - minY),
    parameter_names: [
      "objectId",
      "name",
      "tagPrefix",
      "alarmGroup",
      "location",
      "deviceId",
      "baseAddress",
    ],
    alarm_templates: [],
    widgets: selection.widgets.map((widget) => ({
      ...cloneWidgetDeep(widget),
      x: widget.x - minX,
      y: widget.y - minY,
      group_id: null,
      config: stripUnsafeTemplateConfig(widget.config ?? {}),
    })),
  };
  project.update((current) => {
    if (!current) return current;
    dirty.set(true);
    return {
      ...current,
      component_templates: [...(current.component_templates ?? []), template],
    };
  });
  log(`Component template created: ${template.name} v${template.version}`, "ok");
  return template.id;
}

function substituteTemplateValue(
  value: unknown,
  parameters: Record<string, string>,
): unknown {
  if (typeof value === "string") {
    return Object.entries(parameters).reduce(
      (result, [key, replacement]) => result.replaceAll(`{${key}}`, replacement),
      value,
    );
  }
  if (Array.isArray(value)) {
    return value.map((item) => substituteTemplateValue(item, parameters));
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>).map(([key, item]) => [
        key,
        substituteTemplateValue(item, parameters),
      ]),
    );
  }
  return value;
}

function buildComponentInstance(
  template: ComponentTemplate,
  x: number,
  y: number,
  zStart: number,
  parameters: Record<string, string>,
): WidgetDef[] {
  const instanceGroup = generateUniqueWidgetId().replace("w_", "cmpgrp_");
  return template.widgets.map((source, index) => ({
    ...cloneWidgetDeep(source),
    id: generateUniqueWidgetId(),
    x: x + source.x,
    y: y + source.y,
    z: zStart + index,
    group_id: instanceGroup,
    tag_id: source.tag_id
      ? String(substituteTemplateValue(source.tag_id, parameters))
      : null,
    locked: false,
    config: substituteTemplateValue(
      stripUnsafeTemplateConfig(source.config ?? {}),
      parameters,
    ) as Record<string, unknown>,
  }));
}

export function instantiateComponentTemplate(
  templateId: string,
  x = 60,
  y = 60,
  parameters: Record<string, string> = {},
): string[] {
  const current = get(project);
  const form = get(activeForm);
  const template = current?.component_templates?.find((item) => item.id === templateId);
  if (!current || !form || !template) {
    log(`Component template not found: ${templateId}`, "err");
    return [];
  }
  const widgets = buildComponentInstance(
    template,
    x,
    y,
    form.widgets.length + 1,
    parameters,
  );
  project.update((value) => {
    if (!value) return value;
    const forms = value.forms.map((item) =>
      item.id === form.id ? { ...item, widgets: [...item.widgets, ...widgets] } : item,
    );
    dirty.set(true);
    return { ...value, forms };
  });
  setSelection(widgets.map((widget) => widget.id), widgets[0]?.id);
  log(`Instantiated ${template.name}: ${widgets.length} widget(s)`, "ok");
  return widgets.map((widget) => widget.id);
}

export function deleteComponentTemplate(templateId: string) {
  project.update((current) => {
    if (!current) return current;
    const templates = (current.component_templates ?? []).filter(
      (template) => template.id !== templateId,
    );
    if (templates.length === (current.component_templates ?? []).length) return current;
    dirty.set(true);
    return { ...current, component_templates: templates };
  });
  log(`Component template removed: ${templateId}`, "warn");
}

interface ComponentPackage {
  format: "proscada.component";
  schemaVersion: 1;
  exportedAt: string;
  integrity: {
    algorithm: "SHA-256";
    digest: string;
  };
  template: ComponentTemplate;
}

async function sha256Hex(value: string): Promise<string> {
  const bytes = new TextEncoder().encode(value);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

function validateComponentTemplate(value: unknown): ComponentTemplate {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Component template must be an object");
  }
  const candidate = value as Record<string, unknown>;
  if (
    typeof candidate.id !== "string" ||
    typeof candidate.name !== "string" ||
    typeof candidate.version !== "string" ||
    !Array.isArray(candidate.widgets)
  ) {
    throw new Error("Component template requires id, name, version and widgets[]");
  }
  if (candidate.widgets.length === 0 || candidate.widgets.length > 500) {
    throw new Error("Component template must contain 1..500 widgets");
  }
  const ids = new Set<string>();
  for (const rawWidget of candidate.widgets) {
    if (!rawWidget || typeof rawWidget !== "object" || Array.isArray(rawWidget)) {
      throw new Error("Component widgets must be objects");
    }
    const widget = rawWidget as WidgetDef;
    if (!widget.id || ids.has(widget.id)) throw new Error(`Duplicate widget id: ${widget.id}`);
    if (!getWidgetCatalogItem(widget.widget_type)) {
      throw new Error(`Unsupported widget type: ${widget.widget_type}`);
    }
    if (
      !Number.isFinite(widget.x) ||
      !Number.isFinite(widget.y) ||
      !Number.isFinite(widget.w) ||
      !Number.isFinite(widget.h) ||
      widget.w <= 0 ||
      widget.h <= 0
    ) {
      throw new Error(`Invalid geometry for component widget: ${widget.id}`);
    }
    const configText = JSON.stringify(widget.config ?? {});
    if (/javascript:|onClickScriptId|<script/i.test(configText)) {
      throw new Error(`Executable content is not allowed in component widget: ${widget.id}`);
    }
    ids.add(widget.id);
  }
  return structuredClone(value) as ComponentTemplate;
}

export async function exportComponentTemplate(templateId: string) {
  const template = get(project)?.component_templates?.find((item) => item.id === templateId);
  if (!template) {
    log(`Component template not found: ${templateId}`, "err");
    return;
  }
  const serializedTemplate = JSON.stringify(template);
  const pkg: ComponentPackage = {
    format: "proscada.component",
    schemaVersion: 1,
    exportedAt: new Date().toISOString(),
    integrity: {
      algorithm: "SHA-256",
      digest: await sha256Hex(serializedTemplate),
    },
    template,
  };
  const { saveTextFile } = await import("$lib/services/fileIo");
  const safeName = template.name.replace(/[^a-z0-9_-]+/gi, "_");
  const path = await saveTextFile(
    `${safeName}-${template.version}.pscctrl`,
    JSON.stringify(pkg, null, 2),
    [{ name: "ProSCADA Component", extensions: ["pscctrl", "json"] }],
  );
  if (path) log(`Component exported: ${path}`, "ok");
}

export async function importComponentTemplateFile() {
  const { openTextFile } = await import("$lib/services/fileIo");
  const picked = await openTextFile([
    { name: "ProSCADA Component", extensions: ["pscctrl", "json"] },
  ]);
  if (!picked) return;
  const raw: unknown = JSON.parse(picked.text);
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    throw new Error("Invalid component package");
  }
  const pkg = raw as Partial<ComponentPackage>;
  if (pkg.format !== "proscada.component" || pkg.schemaVersion !== 1) {
    throw new Error("Unsupported component package format");
  }
  if (
    pkg.integrity?.algorithm !== "SHA-256" ||
    pkg.integrity.digest !== (await sha256Hex(JSON.stringify(pkg.template)))
  ) {
    throw new Error("Component package integrity verification failed");
  }
  const template = validateComponentTemplate(pkg.template);
  project.update((current) => {
    if (!current) return current;
    const templates = (current.component_templates ?? []).filter(
      (item) => item.id !== template.id,
    );
    dirty.set(true);
    return { ...current, component_templates: [...templates, template] };
  });
  log(`Component imported: ${template.name} v${template.version}`, "ok");
}

export function installPumpStationTemplate(): string {
  const existing = get(project)?.component_templates?.find(
    (template) => template.id === "builtin-pump-station-2p2f1s",
  );
  if (existing) return existing.id;
  const template: ComponentTemplate = {
    id: "builtin-pump-station-2p2f1s",
    name: "Pompownia 2P + 2F + 1S",
    category: "Pompownie",
    version: "1.0.0",
    description: "Dwie pompy, dwa pływaki, sonda hydrostatyczna, alarm roll-up i faceplate.",
    width: 520,
    height: 320,
    parameter_names: ["objectId", "name", "tagPrefix", "alarmGroup", "location"],
    widgets: [
      { id: "ps-bg", widget_type: "panel", x: 0, y: 0, w: 520, h: 320, z: 0, tag_id: null, group_id: null, config: { title: "{name}", bgColor: "#ffffff", borderColor: "#94a3b8" } },
      { id: "ps-p1", widget_type: "process_symbol", x: 20, y: 52, w: 145, h: 115, z: 1, tag_id: "{tagPrefix}.P1_RunFb", group_id: null, config: { variant: "pump", label: "P1" } },
      { id: "ps-p2", widget_type: "process_symbol", x: 180, y: 52, w: 145, h: 115, z: 2, tag_id: "{tagPrefix}.P2_RunFb", group_id: null, config: { variant: "pump", label: "P2" } },
      { id: "ps-flo", widget_type: "state_indicator", x: 20, y: 184, w: 145, h: 50, z: 3, tag_id: "{tagPrefix}.FloatLow", group_id: null, config: { title: "FLOAT LOW", variant: "bit" } },
      { id: "ps-fhi", widget_type: "state_indicator", x: 180, y: 184, w: 145, h: 50, z: 4, tag_id: "{tagPrefix}.FloatHigh", group_id: null, config: { title: "FLOAT HIGH", variant: "bit" } },
      { id: "ps-level", widget_type: "numeric", x: 345, y: 52, w: 155, h: 70, z: 5, tag_id: "{tagPrefix}.Level", group_id: null, config: { title: "LEVEL", unit: "m", decimals: 2 } },
      { id: "ps-alarm", widget_type: "alarm_indicator", x: 345, y: 136, w: 155, h: 70, z: 6, tag_id: null, group_id: null, config: { group: "{alarmGroup}", alarms: "[]" } },
      { id: "ps-faceplate-p1", widget_type: "faceplate", x: 20, y: 248, w: 235, h: 58, z: 7, tag_id: "{tagPrefix}.P1_RunFb", group_id: null, config: { variant: "compact", equipmentName: "P1 · {name}", mode: "AUTO", available: true, permissive: true, local: false, startTagId: "{tagPrefix}.P1_StartCmd", stopTagId: "{tagPrefix}.P1_StopCmd", startValue: 1, stopValue: 1 } },
      { id: "ps-faceplate-p2", widget_type: "faceplate", x: 265, y: 248, w: 235, h: 58, z: 8, tag_id: "{tagPrefix}.P2_RunFb", group_id: null, config: { variant: "compact", equipmentName: "P2 · {name}", mode: "AUTO", available: true, permissive: true, local: false, startTagId: "{tagPrefix}.P2_StartCmd", stopTagId: "{tagPrefix}.P2_StopCmd", startValue: 1, stopValue: 1 } },
    ],
    alarm_templates: [
      { id: "{objectId}-P1-FAULT", name: "{name} · P1 fault", tag_id: "{tagPrefix}.P1_Fault", group_id: "{alarmGroup}", priority: "high", when_true: true, hi_limit: null, lo_limit: null, deadband: 0, on_delay_ms: 250, off_delay_ms: 500, latching: true, message: "Pump 1 fault — operator action required" },
      { id: "{objectId}-P2-FAULT", name: "{name} · P2 fault", tag_id: "{tagPrefix}.P2_Fault", group_id: "{alarmGroup}", priority: "high", when_true: true, hi_limit: null, lo_limit: null, deadband: 0, on_delay_ms: 250, off_delay_ms: 500, latching: true, message: "Pump 2 fault — operator action required" },
      { id: "{objectId}-LOW", name: "{name} · Low level", tag_id: "{tagPrefix}.FloatLow", group_id: "{alarmGroup}", priority: "high", when_true: true, hi_limit: null, lo_limit: null, deadband: 0, on_delay_ms: 500, off_delay_ms: 500, latching: false, message: "Low level / dry-run risk" },
      { id: "{objectId}-HIGH", name: "{name} · High level", tag_id: "{tagPrefix}.FloatHigh", group_id: "{alarmGroup}", priority: "high", when_true: true, hi_limit: null, lo_limit: null, deadband: 0, on_delay_ms: 500, off_delay_ms: 500, latching: false, message: "High wet-well level" },
      { id: "{objectId}-HH", name: "{name} · High-high level", tag_id: "{tagPrefix}.Level", group_id: "{alarmGroup}", priority: "critical", when_true: false, hi_limit: 95, lo_limit: null, deadband: 2, on_delay_ms: 500, off_delay_ms: 1000, latching: true, message: "High-high level — immediate response required" },
    ],
  };
  project.update((current) => {
    if (!current) return current;
    dirty.set(true);
    return {
      ...current,
      component_templates: [...(current.component_templates ?? []), template],
    };
  });
  log(`Installed component template: ${template.name}`, "ok");
  return template.id;
}

function parseCsvLine(line: string): string[] {
  const cells: string[] = [];
  let value = "";
  let quoted = false;
  for (let index = 0; index < line.length; index++) {
    const char = line[index];
    if (char === '"') {
      if (quoted && line[index + 1] === '"') {
        value += '"';
        index++;
      } else {
        quoted = !quoted;
      }
    } else if (char === "," && !quoted) {
      cells.push(value.trim());
      value = "";
    } else {
      value += char;
    }
  }
  cells.push(value.trim());
  if (quoted) throw new Error("Unclosed CSV quote");
  return cells;
}

function pumpStationTags(
  parameters: Record<string, string>,
  current: ScadaProject,
): TagDefinition[] {
  const prefix = parameters.tagPrefix;
  const deviceId = parameters.deviceId;
  const baseAddress = Number(parameters.baseAddress);
  if (!deviceId) throw new Error(`${parameters.objectId}: deviceId is required`);
  if (!current.devices.some((device) => device.id === deviceId)) {
    throw new Error(`${parameters.objectId}: deviceId ${deviceId} does not exist`);
  }
  if (!Number.isInteger(baseAddress) || baseAddress < 0 || baseAddress > 65520) {
    throw new Error(`${parameters.objectId}: baseAddress must be an integer 0..65520`);
  }
  const boolTag = (
    suffix: string,
    name: string,
    address: number,
    bit: number,
    writable = false,
  ): TagDefinition => ({
    id: `${prefix}.${suffix}`,
    name: `${parameters.name} · ${name}`,
    device_id: deviceId,
    data_type: "bool",
    binding: {
      address,
      bit,
      table: "holding",
      writable,
      bit_write_mode: "mask_write",
      single_writer: false,
      verify_readback: !writable,
    },
    unit: "",
    description: `${parameters.objectId} ${name}`,
    scale: 1,
    offset: 0,
    decimals: 0,
  });
  return [
    boolTag("P1_RunFb", "P1 run feedback", baseAddress, 0),
    boolTag("P2_RunFb", "P2 run feedback", baseAddress, 1),
    boolTag("P1_Fault", "P1 fault", baseAddress, 2),
    boolTag("P2_Fault", "P2 fault", baseAddress, 3),
    boolTag("FloatLow", "Low float", baseAddress, 4),
    boolTag("FloatHigh", "High float", baseAddress, 5),
    boolTag("P1_StartCmd", "P1 start command", baseAddress + 2, 0, true),
    boolTag("P1_StopCmd", "P1 stop command", baseAddress + 2, 1, true),
    boolTag("P2_StartCmd", "P2 start command", baseAddress + 2, 2, true),
    boolTag("P2_StopCmd", "P2 stop command", baseAddress + 2, 3, true),
    {
      id: `${prefix}.Level`,
      name: `${parameters.name} · Level`,
      device_id: deviceId,
      data_type: "u16",
      binding: { address: baseAddress + 1, table: "holding", writable: false },
      unit: "m",
      description: `${parameters.objectId} hydrostatic level`,
      scale: 0.01,
      offset: 0,
      decimals: 2,
    },
  ];
}

function appendAlarmGroupPath(
  path: string,
  objectId: string,
  groups: AlarmGroupDefinition[],
): void {
  const segments = path.split("/").map((segment) => segment.trim()).filter(Boolean);
  let parentId: string | null = null;
  let fullPath = "";
  for (const segment of segments) {
    fullPath = fullPath ? `${fullPath}/${segment}` : segment;
    if (!groups.some((group) => group.id === fullPath)) {
      groups.push({
        id: fullPath,
        name: segment,
        parent_id: parentId,
        object_id: fullPath === path ? objectId : null,
        description: fullPath === path ? `Generated component alarm group for ${objectId}` : "",
      });
    }
    parentId = fullPath;
  }
}

export function bulkInstantiateComponentTemplate(templateId: string, csv: string): number {
  const current = get(project);
  const form = get(activeForm);
  const template = current?.component_templates?.find((item) => item.id === templateId);
  if (!current || !form || !template) throw new Error("Component template or active form missing");
  const lines = csv.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
  if (lines.length < 2) throw new Error("CSV requires a header and at least one data row");
  if (lines.length > 501) throw new Error("Bulk import is limited to 500 objects");
  const headers = parseCsvLine(lines[0]);
  const required = ["objectId", "name", "tagPrefix", "alarmGroup", "location"];
  if (template.id === "builtin-pump-station-2p2f1s") {
    required.push("deviceId", "baseAddress");
  }
  for (const header of required) {
    if (!headers.includes(header)) throw new Error(`CSV missing required column: ${header}`);
  }
  const objectIds = new Set<string>();
  const allWidgets: WidgetDef[] = [];
  const nextGroups = structuredClone(current.alarm_groups ?? []);
  const nextAlarms = structuredClone(current.alarms);
  const nextTags = structuredClone(current.tags);
  const existingTagIds = new Set(nextTags.map((tag) => tag.id));
  const existingAlarmIds = new Set(nextAlarms.map((alarm) => alarm.id));
  const occupiedHoldingAddresses = new Set(
    current.tags
      .filter((tag) => tag.binding.table === "holding")
      .map((tag) => `${tag.device_id}:${tag.binding.address}`),
  );
  for (let rowIndex = 1; rowIndex < lines.length; rowIndex++) {
    const cells = parseCsvLine(lines[rowIndex]);
    if (cells.length !== headers.length) {
      throw new Error(`CSV row ${rowIndex + 1} has ${cells.length} cells; expected ${headers.length}`);
    }
    const parameters = Object.fromEntries(headers.map((header, index) => [header, cells[index]]));
    if (!parameters.objectId) throw new Error(`CSV row ${rowIndex + 1}: objectId is required`);
    if (objectIds.has(parameters.objectId)) {
      throw new Error(`CSV row ${rowIndex + 1}: duplicate objectId ${parameters.objectId}`);
    }
    objectIds.add(parameters.objectId);
    appendAlarmGroupPath(parameters.alarmGroup, parameters.objectId, nextGroups);
    if (template.id === "builtin-pump-station-2p2f1s") {
      const generatedTags = pumpStationTags(parameters, current);
      const generatedAddresses = new Set(
        generatedTags.map((tag) => `${tag.device_id}:${tag.binding.address}`),
      );
      for (const address of generatedAddresses) {
        if (occupiedHoldingAddresses.has(address)) {
          throw new Error(`CSV row ${rowIndex + 1}: physical holding address collision ${address}`);
        }
        occupiedHoldingAddresses.add(address);
      }
      for (const tag of generatedTags) {
        if (existingTagIds.has(tag.id)) {
          throw new Error(`CSV row ${rowIndex + 1}: duplicate generated tag ${tag.id}`);
        }
        existingTagIds.add(tag.id);
        nextTags.push(tag);
      }
    }
    for (const alarmTemplate of template.alarm_templates ?? []) {
      const alarm = substituteTemplateValue(
        structuredClone(alarmTemplate),
        parameters,
      ) as AlarmDefinition;
      if (existingAlarmIds.has(alarm.id)) {
        throw new Error(`CSV row ${rowIndex + 1}: duplicate generated alarm ${alarm.id}`);
      }
      existingAlarmIds.add(alarm.id);
      nextAlarms.push(alarm);
    }
    const instanceIndex = rowIndex - 1;
    const column = instanceIndex % 4;
    const row = Math.floor(instanceIndex / 4);
    allWidgets.push(
      ...buildComponentInstance(
        template,
        40 + column * (template.width + 24),
        40 + row * (template.height + 24),
        form.widgets.length + allWidgets.length + 1,
        parameters,
      ),
    );
  }
  project.update((value) => {
    if (!value) return value;
    dirty.set(true);
    const objectCount = lines.length - 1;
    const requiredWidth = 80 + Math.min(4, objectCount) * (template.width + 24);
    const requiredHeight = 80 + Math.ceil(objectCount / 4) * (template.height + 24);
    return {
      ...value,
      tags: nextTags,
      alarms: nextAlarms,
      alarm_groups: nextGroups,
      forms: value.forms.map((item) =>
        item.id === form.id
          ? {
              ...item,
              width: Math.max(item.width, requiredWidth),
              height: Math.max(item.height, requiredHeight),
              widgets: [...item.widgets, ...allWidgets],
            }
          : item,
      ),
    };
  });
  log(`Bulk instantiated ${lines.length - 1} × ${template.name}`, "ok");
  return lines.length - 1;
}

export function deleteSelectedWidget() {
  const ids = selectionIds();
  if (ids.length === 0) return;
  const locked = get(activeForm)?.widgets.some((w) => ids.includes(w.id) && w.locked);
  if (locked) {
    log("Cannot delete locked widget(s) — unlock first", "warn");
    return;
  }
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      return { ...f, widgets: f.widgets.filter((w) => !ids.includes(w.id)) };
    });
    dirty.set(true);
    return { ...p, forms };
  });
  selectedWidgetId.set(null);
  selectedWidgetIds.set([]);
  log(`Deleted ${ids.length} object(s)`, "warn");
}

export function copySelectedWidgets() {
  const form = get(activeForm);
  if (!form) return;
  const items = widgetsFromSelection(form).map(cloneWidgetDeep);
  if (items.length === 0) {
    log("Nothing to copy", "warn");
    return;
  }
  clipboard.set(items);
  log(`Copied ${items.length} object(s)`, "ok");
}

export function cutSelectedWidgets() {
  const form = get(activeForm);
  if (!form) return;
  const items = widgetsFromSelection(form);
  if (items.length === 0) return;
  if (items.some((w) => w.locked)) {
    log("Cannot cut locked widget(s)", "warn");
    return;
  }
  clipboard.set(items.map(cloneWidgetDeep));
  const ids = items.map((w) => w.id);
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      return { ...f, widgets: f.widgets.filter((w) => !ids.includes(w.id)) };
    });
    dirty.set(true);
    return { ...p, forms };
  });
  selectedWidgetId.set(null);
  selectedWidgetIds.set([]);
  log(`Cut ${items.length} object(s)`, "ok");
}

/** Paste clipboard with optional offset; returns new ids. */
export function pasteWidgets(offsetX = 24, offsetY = 24): string[] {
  const items = get(clipboard);
  if (items.length === 0) {
    log("Clipboard empty", "warn");
    return [];
  }
  const form = get(activeForm);
  if (!form) return [];

  const maxZ = form.widgets.reduce((m, w) => Math.max(m, w.z ?? 0), 0);
  const idMap = new Map<string, string>();
  for (const w of items) idMap.set(w.id, generateUniqueWidgetId());

  // Remap group_ids for pasted set only
  const groupMap = new Map<string, string>();
  for (const w of items) {
    if (w.group_id && !groupMap.has(w.group_id)) {
      groupMap.set(w.group_id, `grp_${Date.now().toString(36)}_${groupMap.size}`);
    }
  }

  const newWidgets: WidgetDef[] = items.map((w, i) => ({
    ...cloneWidgetDeep(w),
    id: idMap.get(w.id)!,
    x: w.x + offsetX,
    y: w.y + offsetY,
    z: maxZ + i + 1,
    group_id: w.group_id ? groupMap.get(w.group_id) ?? null : null,
    locked: false,
  }));

  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      return { ...f, widgets: [...f.widgets, ...newWidgets] };
    });
    dirty.set(true);
    return { ...p, forms };
  });

  const newIds = newWidgets.map((w) => w.id);
  selectedWidgetIds.set(newIds);
  selectedWidgetId.set(newIds[0] ?? null);
  log(`Pasted ${newIds.length} object(s)`, "ok");
  return newIds;
}

/** Multi-copy: N duplicates of selection with cascading offset. */
export function multiCopySelected(count = 3, stepX = 20, stepY = 20) {
  const form = get(activeForm);
  if (!form) return;
  const items = widgetsFromSelection(form);
  if (items.length === 0) {
    log("Nothing to multi-copy", "warn");
    return;
  }
  // Seed clipboard then paste N times without clearing original
  clipboard.set(items.map(cloneWidgetDeep));
  const allNew: string[] = [];
  for (let i = 1; i <= count; i++) {
    const ids = pasteWidgets(stepX * i, stepY * i);
    allNew.push(...ids);
    // restore clipboard originals (paste doesn't mutate clipboard content identity but offsets from original)
    clipboard.set(items.map(cloneWidgetDeep));
  }
  if (allNew.length) {
    selectedWidgetIds.set(allNew);
    selectedWidgetId.set(allNew[0]);
  }
  log(`Multi-copy ×${count} → ${allNew.length} new object(s)`, "ok");
}

export function duplicateSelected() {
  copySelectedWidgets();
  pasteWidgets(16, 16);
}

export function toggleLockSelected() {
  const ids = selectionIds();
  if (ids.length === 0) return;
  const form = get(activeForm);
  if (!form) return;
  const anyUnlocked = form.widgets.some((w) => ids.includes(w.id) && !w.locked);
  const next = anyUnlocked; // if any unlocked → lock all; else unlock all
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      return {
        ...f,
        widgets: f.widgets.map((w) =>
          ids.includes(w.id) ? { ...w, locked: next } : w,
        ),
      };
    });
    dirty.set(true);
    return { ...p, forms };
  });
  log(next ? `Locked ${ids.length} object(s)` : `Unlocked ${ids.length} object(s)`, "ok");
}

export function selectAllWidgets() {
  const form = get(activeForm);
  if (!form) return;
  const ids = form.widgets.map((w) => w.id);
  selectedWidgetIds.set(ids);
  selectedWidgetId.set(ids[0] ?? null);
  log(`Selected all (${ids.length})`, "info");
}

export function selectWidgetById(id: string, additive = false) {
  if (additive) {
    selectedWidgetIds.update((ids) =>
      ids.includes(id) ? ids.filter((x) => x !== id) : [...ids, id],
    );
    selectedWidgetId.set(id);
  } else {
    selectedWidgetId.set(id);
    selectedWidgetIds.set([id]);
  }
}

export function openAttributesPanel() {
  focusPropertiesTick.update((n) => n + 1);
  log("Attributes (Properties) focused", "info");
}

export function bringSelectedToFront() {
  const id = get(selectedWidgetId);
  if (id) reorderWidget(id, "bring_to_front");
}
export function sendSelectedToBack() {
  const id = get(selectedWidgetId);
  if (id) reorderWidget(id, "send_to_back");
}
export function bringSelectedForward() {
  const id = get(selectedWidgetId);
  if (id) reorderWidget(id, "bring_forward");
}
export function sendSelectedBackward() {
  const id = get(selectedWidgetId);
  if (id) reorderWidget(id, "send_backward");
}

export function reorderWidget(
  id: string,
  action: "bring_to_front" | "send_to_back" | "bring_forward" | "send_backward",
) {
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;

      const sorted = [...f.widgets].sort((a, b) => (a.z ?? 0) - (b.z ?? 0));
      const targetWidget = sorted.find((w) => w.id === id);
      if (!targetWidget) return f;

      const selectedIds = get(selectedWidgetIds);
      const targetIds =
        targetWidget.group_id
          ? sorted.filter((w) => w.group_id === targetWidget.group_id).map((w) => w.id)
          : selectedIds.includes(id) && selectedIds.length > 1
          ? selectedIds
          : [id];

      const movingWidgets: WidgetDef[] = [];
      const remainingWidgets: WidgetDef[] = [];

      for (const w of sorted) {
        if (targetIds.includes(w.id)) {
          movingWidgets.push(w);
        } else {
          remainingWidgets.push(w);
        }
      }

      let finalOrder: WidgetDef[] = [];

      if (action === "bring_to_front") {
        finalOrder = [...remainingWidgets, ...movingWidgets];
      } else if (action === "send_to_back") {
        finalOrder = [...movingWidgets, ...remainingWidgets];
      } else if (action === "bring_forward") {
        const maxIdx = Math.max(...targetIds.map((tid) => sorted.findIndex((w) => w.id === tid)));
        const insertIdx = Math.min(sorted.length, maxIdx + 2) - movingWidgets.length;
        const temp = [...remainingWidgets];
        const safeInsertIdx = Math.min(temp.length, Math.max(0, insertIdx));
        temp.splice(safeInsertIdx, 0, ...movingWidgets);
        finalOrder = temp;
      } else if (action === "send_backward") {
        const minIdx = Math.min(...targetIds.map((tid) => sorted.findIndex((w) => w.id === tid)));
        const insertIdx = Math.max(0, minIdx - 1);
        const temp = [...remainingWidgets];
        const safeInsertIdx = Math.min(temp.length, Math.max(0, insertIdx));
        temp.splice(safeInsertIdx, 0, ...movingWidgets);
        finalOrder = temp;
      }

      const reindexed = finalOrder.map((w, i) => ({ ...w, z: i + 1 }));
      return { ...f, widgets: reindexed };
    });
    dirty.set(true);
    return { ...p, forms };
  });
  log(`Reordered widget layer: ${action}`, "ok");
}

export function groupSelectedWidgets() {
  const ids = selectionIds();
  if (ids.length < 2) {
    log("Select 2+ objects to group (Shift/Ctrl+click or marquee)", "warn");
    return;
  }

  const newGroupId = `grp_${Date.now().toString(36)}`;
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      return {
        ...f,
        widgets: f.widgets.map((w) =>
          ids.includes(w.id) ? { ...w, group_id: newGroupId } : w,
        ),
      };
    });
    dirty.set(true);
    return { ...p, forms };
  });
  // Keep whole group selected
  selectedWidgetIds.set(ids);
  selectedWidgetId.set(ids[0] ?? null);
  log(`Grouped ${ids.length} objects → ${newGroupId}`, "ok");
}

export function ungroupSelectedWidgets() {
  const ids = get(selectedWidgetIds);
  const single = get(selectedWidgetId);
  const targetIds = Array.from(new Set([...ids, ...(single ? [single] : [])]));
  if (targetIds.length === 0) return;

  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const activeFormObj = p.forms.find((f) => f.id === formId);
    if (!activeFormObj) return p;

    const groupIdsToDissolve = new Set(
      activeFormObj.widgets
        .filter((w) => targetIds.includes(w.id) && w.group_id)
        .map((w) => w.group_id!),
    );

    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      return {
        ...f,
        widgets: f.widgets.map((w) =>
          w.group_id && (groupIdsToDissolve.has(w.group_id) || targetIds.includes(w.id))
            ? { ...w, group_id: null }
            : w,
        ),
      };
    });
    dirty.set(true);
    return { ...p, forms };
  });
  log("Ungrouped selected widgets", "ok");
}

export function alignSelectedWidgets(
  alignment: "left" | "center" | "right" | "top" | "middle" | "bottom",
) {
  const form = get(activeForm);
  if (!form) return;
  const ids = selectionIds();
  if (ids.length < 2) {
    log("Select 2+ objects to align", "warn");
    return;
  }
  const targets = form.widgets.filter((w) => ids.includes(w.id));
  if (targets.length < 2) return;

  const minX = Math.min(...targets.map((w) => w.x));
  const maxX = Math.max(...targets.map((w) => w.x + w.w));
  const minY = Math.min(...targets.map((w) => w.y));
  const maxY = Math.max(...targets.map((w) => w.y + w.h));
  const centerX = minX + (maxX - minX) / 2;
  const centerY = minY + (maxY - minY) / 2;

  recordUndo("Align Widgets", true);
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      return {
        ...f,
        widgets: f.widgets.map((w) => {
          if (!ids.includes(w.id) || w.locked) return w;
          let x = w.x;
          let y = w.y;
          if (alignment === "left") x = minX;
          else if (alignment === "center") x = Math.round(centerX - w.w / 2);
          else if (alignment === "right") x = maxX - w.w;
          else if (alignment === "top") y = minY;
          else if (alignment === "middle") y = Math.round(centerY - w.h / 2);
          else if (alignment === "bottom") y = maxY - w.h;
          return { ...w, x, y };
        }),
      };
    });
    dirty.set(true);
    return { ...p, forms };
  });
  log(`Aligned ${targets.length} objects: ${alignment}`, "ok");
}

export function moveSelectedWidgets(dx: number, dy: number) {
  const form = get(activeForm);
  if (!form) return;
  const ids = selectionIds();
  if (ids.length === 0) return;

  const moveIds = expandSelectionWithGroups(form, ids);
  const anyLocked = form.widgets.some((x) => moveIds.includes(x.id) && x.locked);
  if (anyLocked) {
    log("Cannot move locked widget(s)", "warn");
    return;
  }

  recordUndo("Nudge Widgets");
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const idSet = new Set(moveIds);
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      return {
        ...f,
        widgets: f.widgets.map((w) => {
          if (!idSet.has(w.id) || w.locked) return w;
          return { ...w, x: w.x + dx, y: w.y + dy };
        }),
      };
    });
    dirty.set(true);
    return { ...p, forms };
  });
}

export async function persistProject(forceDialog = false): Promise<boolean> {
  const p = get(project);
  if (!p) return false;
  try {
    const saved = await api.saveProject(p);
    project.set(saved);
    const diskResult = await saveProjectToDisk(saved, forceDialog);
    dirty.set(false);
    if (diskResult.path) {
      log(`Project saved to disk → ${diskResult.path}`, "ok");
    } else {
      log("Project saved (in-memory)", "ok");
    }
    return true;
  } catch (e) {
    log(`Save failed: ${e}`, "err");
    return false;
  }
}

export function updateFormMeta(patch: Partial<FormDef>) {
  recordUndo("Update Screen", true);
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const targetForm = p.forms.find((f) => f.id === formId);
    const safePatch = { ...patch };
    if (targetForm && isMainScreen(targetForm) && safePatch.name && safePatch.name.trim().toLowerCase() !== "main") {
      log("Nazwa głównego ekranu 'Main' jest zablokowana i nie może zostać zmieniona!", "warn");
      delete safePatch.name;
    }
    // Block renaming any non-Main screen TO "Main"
    if (targetForm && !isMainScreen(targetForm) && safePatch.name && isMainScreen({ id: "", name: safePatch.name })) {
      log("Nazwa 'Main' jest zarezerwowana dla głównego ekranu!", "warn");
      delete safePatch.name;
    }
    const forms = p.forms.map((f) => (f.id === formId ? { ...f, ...safePatch } : f));
    const tree = (p.tree ?? []).map((n) =>
      n.kind === "screen" && n.ref_id === formId && safePatch.name
        ? { ...n, name: safePatch.name }
        : n,
    );
    dirty.set(true);
    return { ...p, forms, tree };
  });
}

function screensFolderId(tree: ProjectNode[]): string | null {
  const byName = tree.find(
    (n) => n.kind === "folder" && n.parent_id == null && n.name.toLowerCase() === "screens",
  );
  return byName?.id ?? tree.find((n) => n.kind === "folder" && n.parent_id == null)?.id ?? null;
}

export function addNewForm(
  name?: string,
  width = 1040,
  height = 700,
  background = "#F4F5F7",
  grid = 8,
  parentFolderId?: string | null,
) {
  // Block reserved Main screen name
  if (name && isMainScreen({ id: "", name })) {
    log("Nazwa 'Main' jest zarezerwowana dla głównego ekranu i nie może być użyta!", "warn");
    return;
  }

  recordUndo("Add Screen", true);
  let newFormId = "";
  let formName = "";
  let nodeId = "";
  project.update((p) => {
    if (!p) return p;
    const normalized = ensureProjectTree(p);
    const tree = [...(normalized.tree ?? [])];
    const count = normalized.forms.length + 1;
    newFormId = uid("form");
    formName = name || `Screen_${count}`;
    const newForm: FormDef = {
      id: newFormId,
      name: formName,
      width,
      height,
      background,
      grid,
      widgets: [],
    };
    const parent =
      parentFolderId ??
      screensFolderId(tree) ??
      null;
    nodeId = uid("scr");
    tree.push({
      id: nodeId,
      parent_id: parent,
      kind: "screen",
      name: formName,
      order: nextOrder(tree, parent),
      ref_id: newFormId,
    });
    dirty.set(true);
    return { ...normalized, forms: [...normalized.forms, newForm], tree };
  });
  if (newFormId) {
    selectedFormId.set(newFormId);
    selectedNodeId.set(nodeId);
    selectedWidgetId.set(null);
    selectedWidgetIds.set([]);
    log(`Created new screen: ${formName}`, "ok");
  }
}

export function deleteForm(formId: string) {
  const currentProject = get(project);
  if (!currentProject) return;

  const targetForm = currentProject.forms.find((f) => f.id === formId);
  if (targetForm && isMainScreen(targetForm)) {
    log(`Główny ekran (${targetForm.name}) jest chroniony i nigdy nie może zostać usunięty!`, "warn");
    return;
  }

  if (!canDeleteForm(formId, currentProject.forms)) {
    log("Nie można usunąć ostatniego ani głównego ekranu w projekcie", "warn");
    return;
  }

  let deletedName = "";
  let nextFormId: string | null = null;
  recordUndo("Delete Screen", true);
  project.update((p) => {
    if (!p) return p;
    const targetIdx = p.forms.findIndex((f) => f.id === formId);
    if (targetIdx === -1) return p;

    deletedName = p.forms[targetIdx].name;
    const remaining = p.forms.filter((f) => f.id !== formId);
    const nextIdx = Math.max(0, targetIdx - 1);
    nextFormId = remaining[nextIdx]?.id ?? remaining[0]?.id ?? null;
    const tree = (p.tree ?? []).filter(
      (n) => !(n.kind === "screen" && n.ref_id === formId),
    );

    dirty.set(true);
    return { ...p, forms: remaining, tree };
  });

  if (deletedName && nextFormId) {
    selectedFormId.set(nextFormId);
    selectedWidgetId.set(null);
    selectedWidgetIds.set([]);
    log(`Deleted screen: ${deletedName}`, "warn");
  }
}

export function selectSolutionNode(nodeId: string | null) {
  selectedNodeId.set(nodeId);
  if (!nodeId) return;
  const p = get(project);
  const node = p?.tree ? findNode(p.tree, nodeId) : undefined;
  if (node?.kind === "screen" && node.ref_id) {
    selectedFormId.set(node.ref_id);
    selectedWidgetId.set(null);
    selectedWidgetIds.set([]);
  }
}

export function addProjectFolder(parentId: string | null = null, name?: string) {
  recordUndo("Add Folder", true);
  let id = "";
  project.update((p) => {
    if (!p) return p;
    const normalized = ensureProjectTree(p);
    const tree = [...(normalized.tree ?? [])];
    id = uid("fld");
    const folderName = name || `NewFolder`;
    tree.push({
      id,
      parent_id: parentId,
      kind: "folder",
      name: folderName,
      order: nextOrder(tree, parentId),
      collapsed: false,
    });
    dirty.set(true);
    return { ...normalized, tree };
  });
  if (id) {
    selectedNodeId.set(id);
    log(`Folder created: ${name || "NewFolder"}`, "ok");
  }
  return id;
}

export function addProjectDocument(
  kind: Exclude<ProjectNodeKind, "folder" | "screen">,
  parentId: string | null = null,
  name?: string,
) {
  recordUndo(`Add ${kind}`, true);
  let id = "";
  let docName = "";
  project.update((p) => {
    if (!p) return p;
    const normalized = ensureProjectTree(p);
    const tree = [...(normalized.tree ?? [])];
    id = uid(kind.slice(0, 3));
    const base = name || (kind === "variables" ? "Variables" : `New_${kind}`);
    docName = base.endsWith(defaultExt(kind)) || kind === "variables" ? base : `${base}${defaultExt(kind)}`;
    tree.push({
      id,
      parent_id: parentId,
      kind,
      name: docName,
      order: nextOrder(tree, parentId),
      content: defaultContent(kind, docName),
      language: kind === "script" ? "javascript" : undefined,
    });
    dirty.set(true);
    return { ...normalized, tree };
  });
  if (id) {
    selectedNodeId.set(id);
    log(`Added ${kind}: ${docName}`, "ok");
  }
  return id;
}

export function addProjectImage(
  name: string,
  contentDataUrl: string,
  parentId: string | null = null,
) {
  recordUndo("Add Image", true);
  let id = "";
  let isUpdate = false;
  project.update((p) => {
    if (!p) return p;
    const normalized = ensureProjectTree(p);
    let tree = [...(normalized.tree ?? [])];

    let targetParent = parentId;
    if (!targetParent) {
      const imagesFolder = tree.find(
        (n) => n.kind === "folder" && n.parent_id === null && n.name.toLowerCase() === "images",
      );
      if (imagesFolder) targetParent = imagesFolder.id;
    }

    const existing = tree.find(
      (n) => n.parent_id === targetParent && n.name.toLowerCase() === name.toLowerCase(),
    );

    if (existing) {
      id = existing.id;
      isUpdate = true;
      tree = tree.map((n) => (n.id === existing.id ? { ...n, content: contentDataUrl } : n));
    } else {
      id = uid("img");
      tree.push({
        id,
        parent_id: targetParent,
        kind: "image",
        name,
        order: nextOrder(tree, targetParent),
        content: contentDataUrl,
      });
    }

    dirty.set(true);
    return { ...normalized, tree };
  });
  if (id) {
    selectedNodeId.set(id);
    log(isUpdate ? `Updated image: ${name}` : `Added image: ${name}`, "ok");
  }
  return id;
}

export async function importImageFiles(
  files: FileList | File[],
  parentId: string | null = null,
): Promise<string[]> {
  const ids: string[] = [];
  for (const file of Array.from(files)) {
    if (!file) continue;
    const isImage = file.type.startsWith("image/") || /\.(svg|png|jpg|jpeg|gif|webp)$/i.test(file.name);
    if (!isImage) continue;

    try {
      const dataUrl = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(String(reader.result ?? ""));
        reader.onerror = reject;
        reader.readAsDataURL(file);
      });

      if (dataUrl) {
        const id = addProjectImage(file.name, dataUrl, parentId);
        ids.push(id);
      }
    } catch {
      /* skipped corrupt file */
    }
  }
  return ids;
}

export function updateProjectNode(id: string, patch: Partial<ProjectNode>) {
  project.update((p) => {
    if (!p?.tree) return p;
    const tree = p.tree.map((n) => {
      if (n.id !== id) return n;
      const next = { ...n, ...patch };
      if (n.kind === "screen" && patch.name && n.ref_id) {
        // keep form name in sync
      }
      return next;
    });
    let forms = p.forms;
    const node = tree.find((n) => n.id === id);
    if (node?.kind === "screen" && node.ref_id && patch.name) {
      forms = forms.map((f) => (f.id === node.ref_id ? { ...f, name: patch.name! } : f));
    }
    dirty.set(true);
    return { ...p, tree, forms };
  });
}

export function toggleFolderCollapsed(id: string) {
  const p = get(project);
  const n = p?.tree?.find((x) => x.id === id);
  if (!n || n.kind !== "folder") return;
  updateProjectNode(id, { collapsed: !(n.collapsed ?? false) });
}

export function deleteProjectNode(nodeId: string) {
  const p = get(project);
  if (!p?.tree) return;
  const node = findNode(p.tree, nodeId);
  if (!node) return;

  // Protect Main screen from deletion via tree node
  if (node.kind === "screen" && isMainScreen(node)) {
    log("Główny ekran 'Main' nie może zostać usunięty!", "warn");
    return;
  }

  if (node.kind === "screen" && node.ref_id) {
    deleteForm(node.ref_id);
    return;
  }

  recordUndo("Delete Item", true);
  const removeIds = new Set(collectDescendantIds(p.tree, nodeId));
  // If deleting folder that contains screens, also remove those forms (keep ≥1 form, never delete Main)
  const screenFormIds = p.tree
    .filter((n) => removeIds.has(n.id) && n.kind === "screen" && n.ref_id && !isMainScreen(n))
    .map((n) => n.ref_id!);

  project.update((cur) => {
    if (!cur) return cur;
    let forms = cur.forms;
    for (const fid of screenFormIds) {
      if (forms.length <= 1) break;
      forms = forms.filter((f) => f.id !== fid);
    }
    const tree2 = (cur.tree ?? []).filter((n) => {
      if (!removeIds.has(n.id)) return true;
      if (n.kind === "screen" && n.ref_id && forms.some((f) => f.id === n.ref_id)) {
        return true;
      }
      return false;
    });
    dirty.set(true);
    return { ...cur, forms, tree: tree2 };
  });

  selectedNodeId.set(null);
  log(`Deleted: ${node.name}`, "warn");
}

export function renameProjectNode(nodeId: string, name: string) {
  const trimmed = name.trim();
  if (!trimmed) return;

  const p = get(project);
  if (p?.tree) {
    const cur = p.tree.find((n) => n.id === nodeId);
    if (cur && cur.kind === "screen" && isMainScreen(cur)) {
      log("Nazwa głównego ekranu 'Main' jest zablokowana i nie może zostać zmieniona!", "warn");
      return;
    }
    // Block renaming any screen TO the reserved "Main" name
    if (cur && cur.kind === "screen" && isMainScreen({ id: "", name: trimmed })) {
      log("Nazwa 'Main' jest zarezerwowana dla głównego ekranu!", "warn");
      return;
    }
    if (cur) {
      const duplicate = p.tree.find(
        (n) => n.id !== nodeId && n.parent_id === cur.parent_id && n.name.toLowerCase() === trimmed.toLowerCase(),
      );
      if (duplicate) {
        log(`An item with name "${trimmed}" already exists in this folder`, "warn");
        return;
      }
    }
  }

  recordUndo("Rename Item", true);
  updateProjectNode(nodeId, { name: trimmed });
  log(`Renamed → ${trimmed}`, "ok");
}

export function moveProjectNode(nodeId: string, newParentId: string | null) {
  recordUndo("Move Item", true);
  project.update((p) => {
    if (!p?.tree) return p;
    const node = findNode(p.tree, nodeId);
    if (node?.kind === "folder" && node.parent_id == null && node.name.toLowerCase() === "screens") {
      log("The root Screens folder cannot be moved", "warn");
      return p;
    }
    if (newParentId === nodeId) return p;
    if (newParentId && isAncestor(p.tree, nodeId, newParentId)) {
      log("Cannot move folder into its descendant", "warn");
      return p;
    }
    if (newParentId) {
      const parent = findNode(p.tree, newParentId);
      if (!parent || parent.kind !== "folder") {
        log("Target must be a folder", "warn");
        return p;
      }
    }
    const tree = p.tree.map((n) =>
      n.id === nodeId
        ? { ...n, parent_id: newParentId, order: nextOrder(p.tree!, newParentId) }
        : n,
    );
    dirty.set(true);
    return { ...p, tree };
  });
}

export async function newBlankProject(name?: string, description?: string) {
  const result = await createAndSaveNewProject(name || "New Project", description || "");
  if (!result) return;
  const { project: p, path } = result;
  try {
    await api.loadProject(p);
  } catch {
    /* browser mock */
  }
  applyLoadedProject(p, `Created project: ${p.name}`, path);
  dirty.set(false);
}

export async function importProjectFromJson(text: string, path?: string) {
  const raw = JSON.parse(text) as unknown;
  const p = normalizeImportedProject(raw);
  try {
    await api.loadProject(p);
  } catch {
    /* browser mock — keep in UI store */
  }
  applyLoadedProject(p, `Imported project: ${p.name}`, path);
  dirty.set(false);
}

export async function importProjectFile() {
  try {
    const res = await openProjectFromDisk();
    if (!res) {
      log("Import cancelled", "warn");
      return;
    }
    try {
      await api.loadProject(res.project);
    } catch {
      /* browser mock */
    }
    applyLoadedProject(res.project, `Loaded project: ${res.project.name}`, res.path);
  } catch (e) {
    log(`Import failed: ${e}`, "err");
  }
}

export const selectedSolutionNode = derived([project, selectedNodeId], ([$p, $id]) => {
  if (!$p?.tree || !$id) return null;
  return findNode($p.tree, $id) ?? null;
});

export const scriptNodes = derived(project, ($p) =>
  ($p?.tree ?? []).filter((n) => n.kind === "script"),
);

export async function refreshAudit() {
  try {
    audit.set(await api.getAudit(100));
  } catch {
    /* */
  }
}

export async function exportProjectJson() {
  const p = get(project);
  if (!p) {
    log("No project to export", "warn");
    return;
  }
  try {
    const { saveTextFile } = await import("$lib/services/fileIo");
    const json = JSON.stringify(p, null, 2);
    const defaultName = `${p.id || "project"}.proscada.json`;
    const path = await saveTextFile(defaultName, json);
    if (!path) {
      log("Export cancelled", "warn");
      return;
    }
    log(`Project exported → ${path}`, "ok");
  } catch (e) {
    log(`Export failed: ${e}`, "err");
  }
}

export function navigateToValidationIssue(issue: {
  targetFormId?: string;
  targetWidgetId?: string;
  targetNodeId?: string;
  targetTagId?: string;
  targetDeviceId?: string;
}) {
  const p = get(project);
  if (!p) return;

  if (issue.targetFormId) {
    switchMode("designer");
    selectedFormId.set(issue.targetFormId);
    if (issue.targetWidgetId) {
      setSelection([issue.targetWidgetId]);
      selectedWidgetId.set(issue.targetWidgetId);
      log(`Navigated to widget '${issue.targetWidgetId}' on screen`, "info");
    } else {
      selectedWidgetId.set(null);
      selectedWidgetIds.set([]);
      log(`Navigated to screen '${issue.targetFormId}'`, "info");
    }
    return;
  }

  if (issue.targetNodeId) {
    switchMode("designer");
    selectSolutionNode(issue.targetNodeId);
    log(`Navigated to item '${issue.targetNodeId}'`, "info");
    return;
  }

  if (issue.targetTagId) {
    const varsNode = p.tree?.find((n) => n.kind === "variables");
    if (varsNode) {
      switchMode("designer");
      selectSolutionNode(varsNode.id);
      log(`Navigated to Variables editor for tag '${issue.targetTagId}'`, "info");
    }
    return;
  }
}

export function addDeviceToProject(dev: DeviceConfig) {
  recordUndo("Add Device", true);
  project.update((p) => {
    if (!p) return p;
    dirty.set(true);
    const existing = p.devices.filter((d) => d.id !== dev.id);
    return { ...p, devices: [...existing, dev] };
  });
  log(`Device added: ${dev.name} (${dev.host}:${dev.port})`, "ok");
}

export function addAlarmToProject(alarm: AlarmDefinition, group?: AlarmGroupDefinition) {
  recordUndo("Add Alarm", true);
  project.update((p) => {
    if (!p) return p;
    dirty.set(true);
    const alarms = p.alarms.filter((a) => a.id !== alarm.id);
    let groups = p.alarm_groups ?? [];
    if (group && !groups.some((g) => g.id === group.id)) {
      groups = [...groups, group];
    }
    return { ...p, alarms: [...alarms, alarm], alarm_groups: groups };
  });
  log(`Alarm added: ${alarm.name} [${alarm.priority.toUpperCase()}]`, "ok");
}

export function addAlarmsToProject(newAlarms: AlarmDefinition[]) {
  recordUndo("Add Alarm List", true);
  project.update((p) => {
    if (!p) return p;
    dirty.set(true);
    const alarmMap = new Map(p.alarms.map((a) => [a.id, a]));
    for (const a of newAlarms) alarmMap.set(a.id, a);
    return { ...p, alarms: Array.from(alarmMap.values()) };
  });
  log(`Batch added ${newAlarms.length} alarm(s)`, "ok");
}

export function addTagToProject(tag: TagDefinition) {
  recordUndo("Add Tag", true);
  project.update((p) => {
    if (!p) return p;
    dirty.set(true);
    const tags = p.tags.filter((t) => t.id !== tag.id);
    return { ...p, tags: [...tags, tag] };
  });
  log(`Tag added: ${tag.name} (${tag.id})`, "ok");
}

export function addTagsToProject(newTags: TagDefinition[]) {
  recordUndo("Add Variable List", true);
  project.update((p) => {
    if (!p) return p;
    dirty.set(true);
    const tagMap = new Map(p.tags.map((t) => [t.id, t]));
    for (const t of newTags) tagMap.set(t.id, t);
    return { ...p, tags: Array.from(tagMap.values()) };
  });
  log(`Batch added ${newTags.length} variable(s)`, "ok");
}

export function openAddDeviceModal() {
  deviceModalState.set({ open: true, mode: "add" });
  addDeviceModalOpen.set(true);
}

export function openEditDeviceModal(deviceId: string) {
  deviceModalState.set({ open: true, mode: "edit", deviceId });
  addDeviceModalOpen.set(true);
}

export function closeDeviceModal() {
  deviceModalState.set({ open: false, mode: "add" });
  addDeviceModalOpen.set(false);
}

export function updateDeviceInProject(id: string, dev: DeviceConfig) {
  recordUndo("Edit Device", true);
  project.update((p) => {
    if (!p) return p;
    dirty.set(true);
    const devices = p.devices.map((d) => (d.id === id ? dev : d));
    return { ...p, devices };
  });
  log(`Device updated: ${dev.name} (${dev.host}:${dev.port})`, "ok");
}

export function deleteDeviceFromProject(id: string) {
  recordUndo("Delete Device", true);
  project.update((p) => {
    if (!p) return p;
    dirty.set(true);
    const devices = p.devices.filter((d) => d.id !== id);
    return { ...p, devices };
  });
  log(`Device deleted: ${id}`, "warn");
}
