<script lang="ts">
  import type { FormDef, WidgetDef, TagValue } from "$lib/types";
  import WidgetView from "$lib/components/widgets/WidgetView.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import {
    selectedWidgetId,
    selectedWidgetIds,
    updateWidget,
    addWidget,
    applyMultiMove,
    setSelection,
    toggleSelection,
  } from "$lib/stores/app";
  import { WIDGET_CATALOG } from "$lib/types";
  import { defaultDynamicsConfig } from "$lib/utils/dynamics";
  import {
    expandSelectionWithGroups,
    groupColor,
    groupLabel,
    widgetIntersectsRect,
  } from "$lib/stores/selection";

  interface Props {
    form: FormDef;
    tagMap: Map<string, TagValue>;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { form, tagMap, design = true, onWrite }: Props = $props();

  let ctxOpen = $state(false);
  let ctxX = $state(0);
  let ctxY = $state(0);

  type Dir = "nw" | "n" | "ne" | "e" | "se" | "s" | "sw" | "w";
  type Gesture =
    | {
        mode: "move";
        px: number;
        py: number;
        origins: Record<string, { x: number; y: number }>;
      }
    | {
        mode: "resize";
        id: string;
        dir: Dir;
        px: number;
        py: number;
        geom: { x: number; y: number; w: number; h: number };
      }
    | {
        mode: "marquee";
        x0: number;
        y0: number;
        x1: number;
        y1: number;
        additive: boolean;
      };

  let gesture = $state<Gesture | null>(null);
  let surfaceEl = $state<HTMLDivElement | null>(null);

  const HANDLES: { dir: Dir; cursor: string; style: string }[] = [
    { dir: "nw", cursor: "nwse-resize", style: "left:-4px;top:-4px" },
    { dir: "n", cursor: "ns-resize", style: "left:calc(50% - 4px);top:-4px" },
    { dir: "ne", cursor: "nesw-resize", style: "right:-4px;top:-4px" },
    { dir: "e", cursor: "ew-resize", style: "right:-4px;top:calc(50% - 4px)" },
    { dir: "se", cursor: "nwse-resize", style: "right:-4px;bottom:-4px" },
    { dir: "s", cursor: "ns-resize", style: "left:calc(50% - 4px);bottom:-4px" },
    { dir: "sw", cursor: "nesw-resize", style: "left:-4px;bottom:-4px" },
    { dir: "w", cursor: "ew-resize", style: "left:-4px;top:calc(50% - 4px)" },
  ];

  function snap(v: number) {
    const g = form.grid || 8;
    return Math.round(v / g) * g;
  }

  function sortedWidgets(): WidgetDef[] {
    return [...form.widgets].sort((a, b) => (a.z ?? 0) - (b.z ?? 0));
  }

  const allGroupIds = $derived(
    form.widgets.map((w) => w.group_id).filter((g): g is string => !!g),
  );

  function clientToForm(clientX: number, clientY: number) {
    if (!surfaceEl) return { x: 0, y: 0 };
    const rect = surfaceEl.getBoundingClientRect();
    return { x: clientX - rect.left, y: clientY - rect.top };
  }

  function openContextMenu(e: MouseEvent, w?: WidgetDef) {
    if (!design) return;
    e.preventDefault();
    e.stopPropagation();
    if (w) {
      const ids = $selectedWidgetIds;
      if (!ids.includes(w.id) && $selectedWidgetId !== w.id) {
        // select this widget / group
        if (w.group_id) {
          const members = form.widgets
            .filter((x) => x.group_id === w.group_id)
            .map((x) => x.id);
          setSelection(members, w.id);
        } else {
          setSelection([w.id], w.id);
        }
      }
    }
    ctxX = e.clientX;
    ctxY = e.clientY;
    ctxOpen = true;
  }

  function onSurfacePointerDown(e: PointerEvent) {
    if (!design) return;
    if (e.button !== 0) return;
    const t = e.target as HTMLElement;
    const onEmpty =
      t === surfaceEl || t.classList.contains("form-surface") || t.classList.contains("marquee");
    if (!onEmpty) return;

    const p = clientToForm(e.clientX, e.clientY);
    gesture = {
      mode: "marquee",
      x0: p.x,
      y0: p.y,
      x1: p.x,
      y1: p.y,
      additive: e.shiftKey || e.ctrlKey || e.metaKey,
    };
    if (!gesture.additive) {
      selectedWidgetId.set(null);
      selectedWidgetIds.set([]);
    }
    (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
  }

  function startMove(e: PointerEvent, w: WidgetDef) {
    if (!design) return;
    if (e.button !== 0) return;
    e.stopPropagation();
    e.preventDefault();

    const additive = e.shiftKey || e.ctrlKey || e.metaKey;

    if (additive) {
      toggleSelection(w.id);
    } else {
      // if already multi-selected and clicking one of them, keep multi
      const already =
        $selectedWidgetIds.includes(w.id) || $selectedWidgetId === w.id;
      if (already && $selectedWidgetIds.length > 1) {
        selectedWidgetId.set(w.id);
      } else if (w.group_id) {
        const members = form.widgets
          .filter((item) => item.group_id === w.group_id)
          .map((item) => item.id);
        setSelection(members, w.id);
      } else {
        setSelection([w.id], w.id);
      }
    }

    // Build move set: current selection expanded by groups
    const baseIds =
      $selectedWidgetIds.length > 0
        ? $selectedWidgetIds
        : $selectedWidgetId
          ? [$selectedWidgetId]
          : [w.id];
    const moveIds = expandSelectionWithGroups(form, baseIds);
    selectedWidgetIds.set(moveIds);
    if (!$selectedWidgetId) selectedWidgetId.set(w.id);

    const anyLocked = form.widgets.some((x) => moveIds.includes(x.id) && x.locked);
    if (anyLocked) return;

    const origins: Record<string, { x: number; y: number }> = {};
    for (const id of moveIds) {
      const item = form.widgets.find((x) => x.id === id);
      if (item) origins[id] = { x: item.x, y: item.y };
    }

    gesture = {
      mode: "move",
      px: e.clientX,
      py: e.clientY,
      origins,
    };
    (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
  }

  function startResize(e: PointerEvent, w: WidgetDef, dir: Dir) {
    if (!design || w.locked) return;
    e.stopPropagation();
    e.preventDefault();
    selectedWidgetId.set(w.id);
    gesture = {
      mode: "resize",
      id: w.id,
      dir,
      px: e.clientX,
      py: e.clientY,
      geom: { x: w.x, y: w.y, w: w.w, h: w.h },
    };
  }

  function onPointerMove(e: PointerEvent) {
    if (!gesture) return;

    if (gesture.mode === "marquee") {
      const p = clientToForm(e.clientX, e.clientY);
      gesture = { ...gesture, x1: p.x, y1: p.y };
      return;
    }

    if (gesture.mode === "move") {
      const dx = snap(e.clientX - gesture.px);
      const dy = snap(e.clientY - gesture.py);
      applyMultiMove(gesture.origins, dx, dy);
      return;
    }

    // resize
    const dx = e.clientX - gesture.px;
    const dy = e.clientY - gesture.py;
    const g = gesture.geom;
    let x = g.x;
    let y = g.y;
    let w = g.w;
    let h = g.h;
    const dir = gesture.dir;
    if (dir.includes("e")) w = Math.max(24, snap(g.w + dx));
    if (dir.includes("s")) h = Math.max(24, snap(g.h + dy));
    if (dir.includes("w")) {
      const nw = Math.max(24, snap(g.w - dx));
      x = snap(g.x + (g.w - nw));
      w = nw;
    }
    if (dir.includes("n")) {
      const nh = Math.max(24, snap(g.h - dy));
      y = snap(g.y + (g.h - nh));
      h = nh;
    }
    updateWidget({ id: gesture.id, x, y, w, h });
  }

  function onPointerUp() {
    if (gesture?.mode === "marquee") {
      const { x0, y0, x1, y1, additive } = gesture;
      const hit = form.widgets
        .filter((w) => widgetIntersectsRect(w, x0, y0, x1, y1))
        .map((w) => w.id);
      if (hit.length) {
        const base = additive ? [...$selectedWidgetIds, ...hit] : hit;
        setSelection(expandSelectionWithGroups(form, base), hit[hit.length - 1]);
      } else if (!additive) {
        selectedWidgetId.set(null);
        selectedWidgetIds.set([]);
      }
    }
    gesture = null;
  }

  function onDragOver(e: DragEvent) {
    if (!design) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
  }

  function onDrop(e: DragEvent) {
    if (!design || !surfaceEl) return;
    e.preventDefault();
    const rawType =
      e.dataTransfer?.getData("application/x-proscada-widget") ||
      e.dataTransfer?.getData("text/plain");
    const type = rawType?.trim();
    if (!type) return;
    const cat = WIDGET_CATALOG.find((c) => c.type === type);
    if (!cat) return;
    const rect = surfaceEl.getBoundingClientRect();
    const x = snap(e.clientX - rect.left);
    const y = snap(e.clientY - rect.top);
    const id = `w_${Date.now().toString(36)}`;
    addWidget({
      id,
      widget_type: type,
      x,
      y,
      w: cat.defaultW,
      h: cat.defaultH,
      z: form.widgets.length + 1,
      tag_id: null,
      group_id: null,
      locked: false,
      config: { ...defaultDynamicsConfig(), ...cat.defaultConfig },
    });
  }

  const marqueeStyle = $derived.by(() => {
    if (!gesture || gesture.mode !== "marquee") return null;
    const left = Math.min(gesture.x0, gesture.x1);
    const top = Math.min(gesture.y0, gesture.y1);
    const width = Math.abs(gesture.x1 - gesture.x0);
    const height = Math.abs(gesture.y1 - gesture.y0);
    return `left:${left}px;top:${top}px;width:${width}px;height:${height}px`;
  });
</script>

<svelte:window onpointermove={onPointerMove} onpointerup={onPointerUp} />

<div class="canvas-wrap">
  <div
    class="form-surface"
    bind:this={surfaceEl}
    style:width="{form.width}px"
    style:height="{form.height}px"
    style:background={form.background}
    onpointerdown={onSurfacePointerDown}
    ondragover={onDragOver}
    ondrop={onDrop}
    oncontextmenu={(e) => openContextMenu(e)}
    role="presentation"
  >
    {#each sortedWidgets() as w (w.id)}
      {@const selected = design && $selectedWidgetId === w.id}
      {@const multiSelected = design && $selectedWidgetIds.includes(w.id)}
      {@const showGroupChrome = design && !!w.group_id}
      {@const gLabel = w.group_id ? groupLabel(w.group_id, allGroupIds) : ""}
      {@const gColor = w.group_id ? groupColor(w.group_id) : ""}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="widget"
        class:design-mode={design}
        class:selected
        class:multi-selected={multiSelected}
        class:in-group={showGroupChrome}
        class:locked={design && !!w.locked}
        style:left="{w.x}px"
        style:top="{w.y}px"
        style:width="{w.w}px"
        style:height="{w.h}px"
        style:z-index={w.z ?? 0}
        style:--grp-color={showGroupChrome ? gColor : "transparent"}
        title={design
          ? w.locked
            ? `${w.id} (locked)`
            : w.group_id
              ? `${w.id} · ${gLabel}`
              : w.id
          : undefined}
        onpointerdown={(e) => startMove(e, w)}
        oncontextmenu={(e) => openContextMenu(e, w)}
      >
        {#if showGroupChrome}
          <div class="group-tag" style:background={gColor}>{gLabel}</div>
        {/if}
        {#if design && w.locked}
          <div class="lock-tag" title="Locked">🔒</div>
        {/if}
        <WidgetView
          widget={w}
          tag={w.tag_id ? tagMap.get(w.tag_id) : null}
          {design}
          {onWrite}
        />
        {#if selected && !w.locked && $selectedWidgetIds.length <= 1}
          {#each HANDLES as h}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="handle"
              style="{h.style};cursor:{h.cursor}"
              onpointerdown={(e) => startResize(e, w, h.dir)}
            ></div>
          {/each}
        {/if}
      </div>
    {/each}

    {#if marqueeStyle}
      <div class="marquee" style={marqueeStyle}></div>
    {/if}
  </div>
</div>

<ContextMenu
  open={ctxOpen}
  x={ctxX}
  y={ctxY}
  onClose={() => (ctxOpen = false)}
/>

<style>
  /* Selection / group chrome: Design mode only */
  .widget.design-mode.in-group {
    outline: 2px solid var(--grp-color, #3b82f6);
    outline-offset: 1px;
  }
  .widget.design-mode.multi-selected {
    outline: 2px solid #60a5fa;
    outline-offset: 0;
    box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.25);
  }
  .widget.design-mode.selected {
    outline: 2px solid #007acc;
    outline-offset: 0;
  }
  .widget.design-mode.in-group.multi-selected,
  .widget.design-mode.in-group.selected {
    outline: 2px solid var(--grp-color, #007acc);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--grp-color, #3b82f6) 35%, transparent);
  }
  .widget.design-mode.locked {
    outline: 1px dashed #eab308;
  }
  .group-tag {
    position: absolute;
    top: -16px;
    left: 0;
    font-size: 9px;
    font-weight: 800;
    color: #ffffff;
    padding: 1px 5px;
    border-radius: 3px;
    pointer-events: none;
    z-index: 10;
    letter-spacing: 0.04em;
  }
  .marquee {
    position: absolute;
    border: 1px solid #007acc;
    background: rgba(0, 122, 204, 0.15);
    pointer-events: none;
    z-index: 9990;
  }

  /* Runtime: pure operator view — no designer chrome */
  .widget:not(.design-mode) {
    outline: none !important;
    box-shadow: none !important;
  }
</style>
