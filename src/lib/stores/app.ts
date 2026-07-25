import { writable, derived, get } from "svelte/store";
import type {
  EngineSnapshot,
  ProjectNode,
  ProjectNodeKind,
  ScadaProject,
  WidgetDef,
  FormDef,
  AuditEntry,
} from "$lib/types";
import { WIDGET_CATALOG } from "$lib/types";
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

export type AppMode = "designer" | "runtime";

export const project = writable<ScadaProject | null>(null);
export const snapshot = writable<EngineSnapshot | null>(null);
export const mode = writable<AppMode>("designer");
export const selectedWidgetId = writable<string | null>(null);
export const selectedWidgetIds = writable<string[]>([]);
export const selectedFormId = writable<string | null>(null);
/** Solution Explorer selection (folder / screen / script / note / …). */
export const selectedNodeId = writable<string | null>(null);
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

let pollTimer: ReturnType<typeof setInterval> | null = null;

function applyLoadedProject(p: ScadaProject, msg?: string) {
  const normalized = ensureProjectTree(p);
  project.set(normalized);
  selectedFormId.set(normalized.forms[0]?.id ?? null);
  selectedNodeId.set(null);
  selectedWidgetId.set(null);
  selectedWidgetIds.set([]);
  dirty.set(false);
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
    const chainOk = await api.verifyAudit();
    log(chainOk ? "Audit chain verified" : "Audit chain BROKEN", chainOk ? "ok" : "err");
    startUiPoll();
  } catch (e) {
    log(`Init error: ${e}`, "err");
    try {
      const p = await api.loadBuiltinWaterTank();
      applyLoadedProject(p);
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
  const form = get(activeForm);
  const screen = form?.name ?? "(no screen)";
  if (m === "runtime") {
    log(`Runtime · running screen: ${screen}`, "ok");
  } else {
    log(`Designer · editing screen: ${screen}`, "info");
  }
}

export function updateWidget(patch: Partial<WidgetDef> & { id: string }) {
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

  if (node.kind === "screen" && node.ref_id) {
    deleteForm(node.ref_id);
    return;
  }

  const removeIds = new Set(collectDescendantIds(p.tree, nodeId));
  // If deleting folder that contains screens, also remove those forms (keep ≥1 form)
  const screenFormIds = p.tree
    .filter((n) => removeIds.has(n.id) && n.kind === "screen" && n.ref_id)
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
  updateProjectNode(nodeId, { name: trimmed });
  log(`Renamed → ${trimmed}`, "ok");
}

export function moveProjectNode(nodeId: string, newParentId: string | null) {
  project.update((p) => {
    if (!p?.tree) return p;
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

export async function newBlankProject(name?: string) {
  const p = createEmptyProject(name || "New Project");
  try {
    await api.loadProject(p);
  } catch {
    /* browser mock */
  }
  applyLoadedProject(p, `Created project: ${p.name}`);
  dirty.set(true);
}

export async function importProjectFromJson(text: string) {
  const raw = JSON.parse(text) as unknown;
  const p = normalizeImportedProject(raw);
  try {
    await api.loadProject(p);
  } catch {
    /* browser mock — keep in UI store */
  }
  applyLoadedProject(p, `Imported project: ${p.name}`);
  dirty.set(true);
}

export async function importProjectFile() {
  try {
    const { openTextFile } = await import("$lib/services/fileIo");
    const picked = await openTextFile();
    if (!picked) {
      log("Import cancelled", "warn");
      return;
    }
    await importProjectFromJson(picked.text);
    log(`Imported from ${picked.path}`, "ok");
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
