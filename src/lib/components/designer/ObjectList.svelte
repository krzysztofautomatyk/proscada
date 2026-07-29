<script lang="ts">
  import type { FormDef } from "$lib/types";
  import {
    selectedWidgetId,
    selectedWidgetIds,
    selectWidgetById,
    setSelection,
    deleteSelectedWidget,
    toggleLockSelected,
    reorderWidget,
    groupSelectedWidgets,
    ungroupSelectedWidgets,
    cutSelectedWidgets,
    copySelectedWidgets,
    pasteWidgets,
    duplicateSelected,
    openAttributesPanel,
  } from "$lib/stores/app";
  import {
    expandSelectionWithGroups,
    groupColor,
    groupLabel,
  } from "$lib/stores/selection";
  import ContextMenu from "./ContextMenu.svelte";

  interface Props {
    form: FormDef;
  }

  let { form }: Props = $props();

  let ctxOpen = $state(false);
  let ctxX = $state(0);
  let ctxY = $state(0);

  const allGroupIds = $derived(
    form.widgets.map((w) => w.group_id).filter((g): g is string => !!g),
  );

  const sorted = $derived(
    [...form.widgets].sort((a, b) => (b.z ?? 0) - (a.z ?? 0)),
  );

  /** Group sections for visual binding */
  const groupsSummary = $derived.by(() => {
    const map = new Map<string, string[]>();
    for (const w of form.widgets) {
      if (!w.group_id) continue;
      const arr = map.get(w.group_id) ?? [];
      arr.push(w.id);
      map.set(w.group_id, arr);
    }
    return [...map.entries()].map(([gid, ids]) => ({
      gid,
      label: groupLabel(gid, allGroupIds),
      color: groupColor(gid),
      count: ids.length,
      ids,
    }));
  });

  const multi = $derived(($selectedWidgetIds?.length ?? 0) > 1);
  const hasSel = $derived(!!$selectedWidgetId || multi);

  function onRowClick(e: MouseEvent, id: string) {
    if (e.shiftKey || e.metaKey || e.ctrlKey) {
      selectWidgetById(id, true);
      // expand groups in additive mode
      const w = form.widgets.find((x) => x.id === id);
      if (w?.group_id) {
        const members = form.widgets
          .filter((x) => x.group_id === w.group_id)
          .map((x) => x.id);
        const merged = expandSelectionWithGroups(form, [
          ...$selectedWidgetIds,
          ...members,
        ]);
        setSelection(merged, id);
      }
    } else {
      const w = form.widgets.find((x) => x.id === id);
      if (w?.group_id) {
        const members = form.widgets
          .filter((x) => x.group_id === w.group_id)
          .map((x) => x.id);
        setSelection(members, id);
      } else {
        setSelection([id], id);
      }
    }
  }

  function selectGroup(ids: string[]) {
    setSelection(ids, ids[0] ?? null);
  }

  function openRowMenu(e: MouseEvent, id: string) {
    e.preventDefault();
    e.stopPropagation();
    if (!$selectedWidgetIds.includes(id) && $selectedWidgetId !== id) {
      onRowClick({ shiftKey: false, metaKey: false, ctrlKey: false } as MouseEvent, id);
    }
    ctxX = e.clientX;
    ctxY = e.clientY;
    ctxOpen = true;
  }

  function typeIcon(t: string): string {
    const map: Record<string, string> = {
      label: "T",
      numeric: "#",
      lamp: "●",
      tank: "▣",
      bar: "▬",
      panel: "▭",
      write_button: "▶",
      shape: "▢",
      line: "／",
      iso_water_tank: "🛢️",
      iso_pump: "⚙",
      iso_pipe: "═",
      iso_terrain: "⛰",
    };
    return map[t] ?? "◇";
  }
</script>

<div class="obj-panel">
  <div class="panel-header">
    Objects on screen
    <span class="count">{form.widgets.length}</span>
  </div>
  <div class="hint">
    {form.name} · Shift/Ctrl+click multi · drag marquee on canvas · groups move as one
  </div>

  {#if groupsSummary.length > 0}
    <div class="groups-bar">
      <span class="groups-title">Linked groups</span>
      {#each groupsSummary as g}
        <button
          type="button"
          class="g-chip"
          style:background={g.color}
          title={g.gid}
          onclick={() => selectGroup(g.ids)}
        >
          {g.label} · {g.count}
        </button>
      {/each}
    </div>
  {/if}

  <div class="list">
    {#if sorted.length === 0}
      <div class="empty">No objects on this screen</div>
    {:else}
      {#each sorted as w (w.id)}
        {@const gLabel = w.group_id ? groupLabel(w.group_id, allGroupIds) : ""}
        {@const gColor = w.group_id ? groupColor(w.group_id) : ""}
        <div
          class="row"
          class:selected={$selectedWidgetId === w.id || $selectedWidgetIds.includes(w.id)}
          class:locked={!!w.locked}
          class:grouped={!!w.group_id}
          style:--grp-color={gColor}
          role="button"
          tabindex="0"
          onclick={(e) => onRowClick(e, w.id)}
          onkeydown={(e) => e.key === "Enter" && onRowClick(new MouseEvent("click"), w.id)}
          oncontextmenu={(e) => openRowMenu(e, w.id)}
        >
          <span class="icon">{typeIcon(w.widget_type)}</span>
          <div class="meta">
            <div class="name">
              {w.widget_type}
              {#if w.locked}<span title="Locked">🔒</span>{/if}
              {#if w.group_id}
                <span class="g-badge" style:background={gColor} title={w.group_id}>{gLabel}</span>
              {/if}
            </div>
            <div class="sub">
              {w.id}
              {#if w.tag_id}
                · {w.tag_id}
              {:else if w.config?.text}
                · {String(w.config.text).slice(0, 24)}
              {/if}
            </div>
          </div>
          <span class="z">z{w.z ?? 0}</span>
        </div>
      {/each}
    {/if}
  </div>

  <div class="toolbar group-actions">
    <button
      type="button"
      class="btn-grp"
      disabled={!hasSel || !multi}
      title="Group selected (need 2+)"
      onclick={() => groupSelectedWidgets()}
    >
      🔗 Group
    </button>
    <button
      type="button"
      class="btn-ungrp"
      disabled={!hasSel}
      title="Ungroup"
      onclick={() => ungroupSelectedWidgets()}
    >
      🔓 Ungroup
    </button>
  </div>
  <div class="toolbar">
    <button type="button" title="Bring front" disabled={!$selectedWidgetId} onclick={() => $selectedWidgetId && reorderWidget($selectedWidgetId, "bring_to_front")}>⇞</button>
    <button type="button" title="Up" disabled={!$selectedWidgetId} onclick={() => $selectedWidgetId && reorderWidget($selectedWidgetId, "bring_forward")}>↑</button>
    <button type="button" title="Down" disabled={!$selectedWidgetId} onclick={() => $selectedWidgetId && reorderWidget($selectedWidgetId, "send_backward")}>↓</button>
    <button type="button" title="To back" disabled={!$selectedWidgetId} onclick={() => $selectedWidgetId && reorderWidget($selectedWidgetId, "send_to_back")}>⇟</button>
    <button type="button" title="Lock" disabled={!hasSel} onclick={() => toggleLockSelected()}>🔒</button>
    <button type="button" title="Delete" disabled={!hasSel} onclick={() => deleteSelectedWidget()}>🗑</button>
  </div>
  <div class="toolbar">
    <button type="button" disabled={!hasSel} onclick={() => cutSelectedWidgets()}>Cut</button>
    <button type="button" disabled={!hasSel} onclick={() => copySelectedWidgets()}>Copy</button>
    <button type="button" onclick={() => pasteWidgets()}>Paste</button>
    <button type="button" disabled={!hasSel} onclick={() => duplicateSelected()}>Dup</button>
    <button type="button" disabled={!hasSel} onclick={() => openAttributesPanel()}>Attr</button>
  </div>
</div>

<ContextMenu open={ctxOpen} x={ctxX} y={ctxY} onClose={() => (ctxOpen = false)} />

<style>
  .obj-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background: var(--vs-bg-2, #252526);
    color: var(--vs-text, #ccc);
  }
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--vs-text-dim, #9d9d9d);
    border-bottom: 1px solid var(--vs-border, #3e3e42);
    background: var(--vs-bg-3, #2d2d30);
  }
  .count {
    background: #3e3e42;
    color: #fff;
    border-radius: 99px;
    padding: 0 7px;
    font-size: 10px;
  }
  .hint {
    padding: 4px 10px;
    font-size: 10px;
    color: #6b7280;
    border-bottom: 1px solid var(--vs-border-soft, #2b2b2b);
    line-height: 1.35;
  }
  .groups-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    align-items: center;
    padding: 6px 8px;
    border-bottom: 1px solid #2b2b2b;
  }
  .groups-title {
    font-size: 10px;
    font-weight: 700;
    color: #9d9d9d;
    margin-right: 4px;
  }
  .g-chip {
    border: none;
    color: #fff;
    font-size: 10px;
    font-weight: 800;
    padding: 2px 8px;
    border-radius: 99px;
    cursor: pointer;
  }
  .list {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }
  .empty {
    padding: 16px;
    font-size: 12px;
    color: #6b7280;
  }
  .row {
    display: grid;
    grid-template-columns: 22px 1fr 28px;
    gap: 6px;
    align-items: center;
    padding: 6px 8px;
    border-bottom: 1px solid #2b2b2b;
    cursor: pointer;
    border-left: 3px solid transparent;
  }
  .row.grouped {
    border-left-color: var(--grp-color, #3b82f6);
  }
  .row:hover {
    background: #2a2d2e;
  }
  .row.selected {
    background: #094771;
    color: #fff;
  }
  .row.locked .name {
    color: #eab308;
  }
  .icon {
    text-align: center;
    font-size: 12px;
    opacity: 0.9;
  }
  .name {
    font-size: 12px;
    font-weight: 700;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .g-badge {
    font-size: 9px;
    font-weight: 800;
    color: #fff;
    padding: 0 5px;
    border-radius: 3px;
  }
  .sub {
    font-size: 10px;
    opacity: 0.7;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .z {
    font-size: 10px;
    opacity: 0.6;
    text-align: right;
  }
  .toolbar {
    display: flex;
    gap: 4px;
    padding: 6px;
    border-top: 1px solid var(--vs-border, #3e3e42);
    background: var(--vs-bg-3, #2d2d30);
  }
  .toolbar button {
    flex: 1;
    background: #333;
    border: 1px solid #444;
    color: #ccc;
    border-radius: 3px;
    padding: 4px;
    cursor: pointer;
    font-size: 11px;
    font-weight: 600;
  }
  .toolbar button:hover:not(:disabled) {
    background: #444;
  }
  .toolbar button:disabled {
    opacity: 0.35;
  }
  .btn-grp {
    background: #1e3a8a !important;
    border-color: #3b82f6 !important;
    color: #fff !important;
  }
  .btn-ungrp {
    background: #3f3f46 !important;
  }
</style>
