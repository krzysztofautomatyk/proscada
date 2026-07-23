import { writable, derived, get } from "svelte/store";
import type {
  EngineSnapshot,
  ScadaProject,
  WidgetDef,
  FormDef,
  AuditEntry,
} from "$lib/types";
import { WIDGET_CATALOG } from "$lib/types";
import { api } from "$lib/services/api";

export type AppMode = "designer" | "runtime";

export const project = writable<ScadaProject | null>(null);
export const snapshot = writable<EngineSnapshot | null>(null);
export const mode = writable<AppMode>("designer");
export const selectedWidgetId = writable<string | null>(null);
export const selectedWidgetIds = writable<string[]>([]);
export const selectedFormId = writable<string | null>(null);
export const logs = writable<{ t: string; level: "info" | "ok" | "warn" | "err"; msg: string }[]>(
  [],
);
export const audit = writable<AuditEntry[]>([]);
export const dirty = writable(false);

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

let pollTimer: ReturnType<typeof setInterval> | null = null;

export async function initApp() {
  try {
    let p = await api.getProject();
    if (!p) {
      p = await api.loadBuiltinWaterTank();
      log("Loaded built-in Water Tank Dual-Pump project", "ok");
    } else {
      log(`Project loaded: ${p.name}`, "ok");
    }
    project.set(p);
    selectedFormId.set(p.forms[0]?.id ?? null);
    const chainOk = await api.verifyAudit();
    log(chainOk ? "Audit chain verified" : "Audit chain BROKEN", chainOk ? "ok" : "err");
    startUiPoll();
  } catch (e) {
    log(`Init error: ${e}`, "err");
    try {
      const p = await api.loadBuiltinWaterTank();
      project.set(p);
      selectedFormId.set(p.forms[0]?.id ?? null);
      startUiPoll();
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
  const dev = p?.devices.find((d) => d.enabled) ?? p?.devices[0];
  try {
    const test = dev
      ? await api.testDevice(dev.host, dev.port, dev.unit_id, dev.timeout_ms)
      : { ok: false, message: "No device" };
    if (!test.ok) {
      log(`Device test failed: ${test.message}`, "warn");
    } else {
      log(`Device reachable ${dev?.host}:${dev?.port}`, "ok");
    }
    await api.startPolling(dev?.id);
    log(`Polling started → ${dev?.name ?? "device"}`, "ok");
  } catch (e) {
    log(`Connect error: ${e}`, "err");
  }
}

export async function disconnectDevice() {
  await api.stopPolling();
  log("Polling stopped", "warn");
}

export async function switchMode(m: AppMode) {
  mode.set(m);
  await api.setMode(m);
  log(m === "runtime" ? "Runtime mode (operator)" : "Designer mode (engineering)", "info");
}

export function updateWidget(patch: Partial<WidgetDef> & { id: string }) {
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      const target = f.widgets.find((w) => w.id === patch.id);
      if (!target) return f;

      const dx = patch.x !== undefined ? patch.x - target.x : 0;
      const dy = patch.y !== undefined ? patch.y - target.y : 0;

      // Grouped movement
      if (target.group_id && (dx !== 0 || dy !== 0) && patch.w === undefined && patch.h === undefined) {
        return {
          ...f,
          widgets: f.widgets.map((w) =>
            w.group_id === target.group_id
              ? { ...w, x: w.x + dx, y: w.y + dy }
              : w.id === patch.id
                ? { ...w, ...patch }
                : w,
          ),
        };
      }

      return {
        ...f,
        widgets: f.widgets.map((w) => (w.id === patch.id ? { ...w, ...patch } : w)),
      };
    });
    dirty.set(true);
    return { ...p, forms };
  });
}

export function generateUniqueWidgetId(): string {
  const ts = Date.now().toString(36);
  const rand = Math.random().toString(36).slice(2, 7);
  return `w_${ts}_${rand}`;
}

export function addWidget(w: WidgetDef) {
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
    config: { ...cat.defaultConfig },
  });
  log(`Added widget: ${cat.label}`, "ok");
}

export function deleteSelectedWidget() {
  const id = get(selectedWidgetId);
  if (!id) return;
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      return { ...f, widgets: f.widgets.filter((w) => w.id !== id) };
    });
    dirty.set(true);
    return { ...p, forms };
  });
  selectedWidgetId.set(null);
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
      const idx = sorted.findIndex((w) => w.id === id);
      if (idx === -1) return f;

      if (action === "bring_to_front") {
        const maxZ = Math.max(0, ...sorted.map((w) => w.z ?? 0));
        sorted[idx] = { ...sorted[idx], z: maxZ + 10 };
      } else if (action === "send_to_back") {
        const minZ = Math.min(0, ...sorted.map((w) => w.z ?? 0));
        sorted[idx] = { ...sorted[idx], z: Math.min(-10, minZ - 10) };
      } else if (action === "bring_forward" && idx < sorted.length - 1) {
        const nextZ = sorted[idx + 1].z ?? 0;
        sorted[idx] = { ...sorted[idx], z: nextZ + 1 };
      } else if (action === "send_backward" && idx > 0) {
        const prevZ = sorted[idx - 1].z ?? 0;
        sorted[idx] = { ...sorted[idx], z: Math.max(-10, prevZ - 1) };
      }

      const reindexed = [...sorted]
        .sort((a, b) => (a.z ?? 0) - (b.z ?? 0))
        .map((w, i) => ({ ...w, z: i + 1 }));

      return { ...f, widgets: reindexed };
    });
    dirty.set(true);
    return { ...p, forms };
  });
}

export function groupSelectedWidgets() {
  const ids = get(selectedWidgetIds);
  const single = get(selectedWidgetId);
  const targetIds = Array.from(new Set([...ids, ...(single ? [single] : [])]));
  if (targetIds.length < 2) return;

  const newGroupId = `grp_${Date.now().toString(36)}`;
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => {
      if (f.id !== formId) return f;
      return {
        ...f,
        widgets: f.widgets.map((w) =>
          targetIds.includes(w.id) ? { ...w, group_id: newGroupId } : w,
        ),
      };
    });
    dirty.set(true);
    return { ...p, forms };
  });
  log(`Grouped ${targetIds.length} widgets into ${newGroupId}`, "ok");
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

export async function persistProject() {
  const p = get(project);
  if (!p) return;
  try {
    const saved = await api.saveProject(p);
    project.set(saved);
    dirty.set(false);
    log("Project saved (in-memory + hash recomputed)", "ok");
  } catch (e) {
    log(`Save failed: ${e}`, "err");
  }
}

export function updateFormMeta(patch: Partial<FormDef>) {
  project.update((p) => {
    if (!p) return p;
    const formId = get(selectedFormId) ?? p.forms[0]?.id;
    const forms = p.forms.map((f) => (f.id === formId ? { ...f, ...patch } : f));
    dirty.set(true);
    return { ...p, forms };
  });
}

export function addNewForm(
  name?: string,
  width = 1040,
  height = 700,
  background = "#F4F5F7",
  grid = 8,
) {
  let newFormId = "";
  let formName = "";
  project.update((p) => {
    if (!p) return p;
    const count = p.forms.length + 1;
    newFormId = `form_${Date.now().toString(36)}`;
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
    dirty.set(true);
    return { ...p, forms: [...p.forms, newForm] };
  });
  if (newFormId) {
    selectedFormId.set(newFormId);
    selectedWidgetId.set(null);
    selectedWidgetIds.set([]);
    log(`Created new screen: ${formName}`, "ok");
  }
}

export function deleteForm(formId: string) {
  let deletedName = "";
  let nextFormId: string | null = null;
  project.update((p) => {
    if (!p) return p;
    if (p.forms.length <= 1) {
      log("Cannot delete the only screen in project", "warn");
      return p;
    }
    const targetIdx = p.forms.findIndex((f) => f.id === formId);
    if (targetIdx === -1) return p;

    deletedName = p.forms[targetIdx].name;
    const remaining = p.forms.filter((f) => f.id !== formId);
    const nextIdx = Math.max(0, targetIdx - 1);
    nextFormId = remaining[nextIdx]?.id ?? remaining[0]?.id ?? null;

    dirty.set(true);
    return { ...p, forms: remaining };
  });

  if (deletedName && nextFormId) {
    selectedFormId.set(nextFormId);
    selectedWidgetId.set(null);
    selectedWidgetIds.set([]);
    log(`Deleted screen: ${deletedName}`, "warn");
  }
}

export async function refreshAudit() {
  try {
    audit.set(await api.getAudit(100));
  } catch {
    /* */
  }
}

export function exportProjectJson() {
  const p = get(project);
  if (!p) return;
  const blob = new Blob([JSON.stringify(p, null, 2)], { type: "application/json" });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = `${p.id}.proscada.json`;
  a.click();
  URL.revokeObjectURL(a.href);
  log("Project exported as JSON", "ok");
}
