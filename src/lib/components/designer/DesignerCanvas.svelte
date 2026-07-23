<script lang="ts">
  import type { FormDef, WidgetDef, TagValue } from "$lib/types";
  import WidgetView from "$lib/components/widgets/WidgetView.svelte";
  import {
    selectedWidgetId,
    selectedWidgetIds,
    updateWidget,
    addWidget,
  } from "$lib/stores/app";
  import { WIDGET_CATALOG } from "$lib/types";

  interface Props {
    form: FormDef;
    tagMap: Map<string, TagValue>;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { form, tagMap, design = true, onWrite }: Props = $props();

  type Dir = "nw" | "n" | "ne" | "e" | "se" | "s" | "sw" | "w";
  interface Gesture {
    id: string;
    mode: "move" | "resize";
    dir?: Dir;
    px: number;
    py: number;
    geom: { x: number; y: number; w: number; h: number };
  }

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

  const currentSelectedWidget = $derived(
    form.widgets.find((w) => w.id === $selectedWidgetId),
  );
  const activeGroupId = $derived(currentSelectedWidget?.group_id ?? null);

  function onSurfacePointerDown(e: PointerEvent) {
    if (!design) return;
    if (e.target === surfaceEl || (e.target as HTMLElement).classList.contains("form-surface")) {
      selectedWidgetId.set(null);
      selectedWidgetIds.set([]);
    }
  }

  function startMove(e: PointerEvent, w: WidgetDef) {
    if (!design) return;
    e.stopPropagation();
    e.preventDefault();

    if (e.shiftKey) {
      selectedWidgetIds.update((ids) =>
        ids.includes(w.id) ? ids.filter((i) => i !== w.id) : [...ids, w.id],
      );
    } else {
      selectedWidgetId.set(w.id);
      if (w.group_id) {
        const groupMembers = form.widgets
          .filter((item) => item.group_id === w.group_id)
          .map((item) => item.id);
        selectedWidgetIds.set(groupMembers);
      } else {
        selectedWidgetIds.set([w.id]);
      }
    }

    gesture = {
      id: w.id,
      mode: "move",
      px: e.clientX,
      py: e.clientY,
      geom: { x: w.x, y: w.y, w: w.w, h: w.h },
    };
    (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
  }

  function startResize(e: PointerEvent, w: WidgetDef, dir: Dir) {
    if (!design) return;
    e.stopPropagation();
    e.preventDefault();
    selectedWidgetId.set(w.id);
    gesture = {
      id: w.id,
      mode: "resize",
      dir,
      px: e.clientX,
      py: e.clientY,
      geom: { x: w.x, y: w.y, w: w.w, h: w.h },
    };
  }

  function onPointerMove(e: PointerEvent) {
    if (!gesture) return;
    const dx = e.clientX - gesture.px;
    const dy = e.clientY - gesture.py;
    const g = gesture.geom;
    let x = g.x;
    let y = g.y;
    let w = g.w;
    let h = g.h;

    if (gesture.mode === "move") {
      x = snap(g.x + dx);
      y = snap(g.y + dy);
    } else {
      const dir = gesture.dir!;
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
    }
    updateWidget({ id: gesture.id, x, y, w, h });
  }

  function onPointerUp() {
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
      config: { ...cat.defaultConfig },
    });
  }
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
    role="presentation"
  >
    {#each sortedWidgets() as w (w.id)}
      {@const selected = design && $selectedWidgetId === w.id}
      {@const multiSelected = design && $selectedWidgetIds.includes(w.id)}
      {@const inActiveGroup = design && activeGroupId && w.group_id === activeGroupId}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="widget"
        class:selected
        class:multi-selected={multiSelected}
        class:in-group={inActiveGroup}
        style:left="{w.x}px"
        style:top="{w.y}px"
        style:width="{w.w}px"
        style:height="{w.h}px"
        style:z-index={selected ? 1000 : w.z ?? 0}
        onpointerdown={(e) => startMove(e, w)}
      >
        {#if inActiveGroup}
          <div class="group-tag">🔗 {w.group_id}</div>
        {/if}
        <WidgetView
          widget={w}
          tag={w.tag_id ? tagMap.get(w.tag_id) : null}
          {design}
          {onWrite}
        />
        {#if selected}
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
  </div>
</div>

<style>
  .widget.in-group:not(.selected) {
    outline: 1.5px dashed #3b82f6;
    outline-offset: 1px;
  }
  .widget.multi-selected:not(.selected) {
    outline: 2px solid #60a5fa;
  }
  .group-tag {
    position: absolute;
    top: -16px;
    left: 0;
    font-size: 9px;
    font-weight: 800;
    background: #3b82f6;
    color: #ffffff;
    padding: 1px 4px;
    border-radius: 3px;
    pointer-events: none;
    z-index: 10;
  }
</style>
