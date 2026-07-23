<script lang="ts">
  import { WIDGET_CATALOG } from "$lib/types";
  import { addCatalogWidget } from "$lib/stores/app";

  const categories = [...new Set(WIDGET_CATALOG.map((w) => w.category))];

  // Manual drag implementation - NO draggable="true" attribute!
  // macOS WKWebView (Tauri) intercepts ALL mouse events on draggable elements
  // at the native NSView level before DOM handlers fire.
  let dragType = $state<string | null>(null);
  let dragStartPos = $state({ x: 0, y: 0 });
  let isDragging = $state(false);
  let dragGhostEl = $state<HTMLDivElement | null>(null);

  function onPointerDown(e: PointerEvent, type: string) {
    dragType = type;
    dragStartPos = { x: e.clientX, y: e.clientY };
    isDragging = false;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragType) return;
    const dist = Math.hypot(e.clientX - dragStartPos.x, e.clientY - dragStartPos.y);
    if (dist > 8 && !isDragging) {
      isDragging = true;
      // Create drag ghost
      if (!dragGhostEl) {
        const ghost = document.createElement("div");
        ghost.style.cssText = `
          position: fixed; z-index: 99999; pointer-events: none;
          background: rgba(37,99,235,0.85); color: #fff; padding: 4px 10px;
          border-radius: 6px; font-size: 11px; font-weight: 700;
          box-shadow: 0 4px 12px rgba(0,0,0,0.4);
        `;
        const cat = WIDGET_CATALOG.find((c) => c.type === dragType);
        ghost.textContent = `${cat?.icon ?? ""} ${cat?.label ?? dragType}`;
        document.body.appendChild(ghost);
        dragGhostEl = ghost;
      }
    }
    if (isDragging && dragGhostEl) {
      dragGhostEl.style.left = `${e.clientX + 12}px`;
      dragGhostEl.style.top = `${e.clientY + 12}px`;
    }
  }

  function onPointerUp(e: PointerEvent) {
    const wasDragging = isDragging;
    const type = dragType;

    // Clean up drag ghost
    if (dragGhostEl) {
      dragGhostEl.remove();
      dragGhostEl = null;
    }
    isDragging = false;
    dragType = null;

    if (!type) return;

    if (!wasDragging) {
      // It was a click — add widget at default position
      addCatalogWidget(type);
    } else {
      // It was a drag — find canvas and add at drop position
      const dropTarget = document.elementFromPoint(e.clientX, e.clientY);
      const canvas = dropTarget?.closest(".form-surface") as HTMLElement | null;
      if (canvas) {
        const rect = canvas.getBoundingClientRect();
        const x = Math.round((e.clientX - rect.left) / 8) * 8;
        const y = Math.round((e.clientY - rect.top) / 8) * 8;
        addCatalogWidget(type, x, y);
      }
      // If dropped outside canvas, do nothing
    }
  }

  function handleQuickAdd(e: MouseEvent, type: string) {
    e.preventDefault();
    e.stopPropagation();
    addCatalogWidget(type);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="panel" style:height="100%;border:none" onpointermove={onPointerMove}>
  <div class="panel-header">Toolbox (Click + or Drag)</div>
  <div class="panel-body">
    {#each categories as cat}
      <div class="tree-group">{cat}</div>
      {#each WIDGET_CATALOG.filter((w) => w.category === cat) as item}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="toolbox-item"
          onpointerdown={(e) => onPointerDown(e, item.type)}
          onpointerup={onPointerUp}
          title="Click to add or drag onto canvas"
          role="button"
          tabindex="0"
        >
          <span class="icon">{item.icon}</span>
          <span class="item-label">{item.label}</span>
          <button
            type="button"
            class="btn-quick-add"
            title="Add {item.label}"
            onmousedown={(e) => e.stopPropagation()}
            onclick={(e) => handleQuickAdd(e, item.type)}
          >
            +
          </button>
        </div>
      {/each}
    {/each}
  </div>
</div>

<style>
  .toolbox-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    cursor: pointer;
    user-select: none;
    border-radius: 4px;
    transition: background 0.15s ease;
    touch-action: none;
  }
  .toolbox-item:hover {
    background: var(--vs-hover, #2a2d2e);
  }
  .item-label {
    flex: 1;
    margin-left: 6px;
    font-size: 11px;
  }
  .btn-quick-add {
    background: transparent;
    border: 1px solid var(--vs-border, #444444);
    color: var(--vs-text, #cccccc);
    border-radius: 3px;
    font-size: 11px;
    font-weight: 800;
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    opacity: 0.5;
    flex-shrink: 0;
  }
  .toolbox-item:hover .btn-quick-add {
    opacity: 1;
    background: #2563eb;
    color: #ffffff;
    border-color: #3b82f6;
  }
</style>
