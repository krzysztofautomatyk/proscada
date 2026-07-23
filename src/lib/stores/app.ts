import { writable, derived, get } from "svelte/store";
import type {
  EngineSnapshot,
  ScadaProject,
  WidgetDef,
  FormDef,
  AuditEntry,
} from "$lib/types";
import { api } from "$lib/services/api";

export type AppMode = "designer" | "runtime";

export const project = writable<ScadaProject | null>(null);
export const snapshot = writable<EngineSnapshot | null>(null);
export const mode = writable<AppMode>("designer");
export const selectedWidgetId = writable<string | null>(null);
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
      return {
        ...f,
        widgets: f.widgets.map((w) => (w.id === patch.id ? { ...w, ...patch } : w)),
      };
    });
    dirty.set(true);
    return { ...p, forms };
  });
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
