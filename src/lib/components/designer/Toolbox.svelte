<script lang="ts">
  import {
    CANONICAL_WIDGETS,
    TOOLBOX_CATEGORIES,
    WIDGET_CATALOG,
  } from "$lib/components/widgets/registry";
  import {
    addCatalogWidget,
    instantiateComponentTemplate,
    project,
  } from "$lib/stores/app";
  import VerticalScrollControls from "./VerticalScrollControls.svelte";

  const FAVORITES_KEY = "proscada.toolbox.favorites";
  const COLLAPSED_KEY = "proscada.toolbox.collapsed";

  const categories = TOOLBOX_CATEGORIES.filter((category) =>
    WIDGET_CATALOG.some((widget) => widget.category === category),
  );

  function loadJson<T>(key: string, fallback: T): T {
    try {
      const raw = localStorage.getItem(key);
      if (!raw) return fallback;
      return JSON.parse(raw) as T;
    } catch {
      return fallback;
    }
  }

  let favorites = $state<string[]>(loadJson<string[]>(FAVORITES_KEY, []));
  let collapsed = $state<Record<string, boolean>>(
    loadJson<Record<string, boolean>>(COLLAPSED_KEY, {})
  );

  // Manual drag implementation - NO draggable="true" attribute!
  // macOS WKWebView (Tauri) intercepts ALL mouse events on draggable elements
  // at the native NSView level before DOM handlers fire.
  let dragType = $state<string | null>(null);
  let dragStartPos = $state({ x: 0, y: 0 });
  let isDragging = $state(false);
  let dragGhostEl = $state<HTMLDivElement | null>(null);
  let scrollContainer = $state<HTMLDivElement | null>(null);

  const favoriteItems = $derived(
    favorites
      .map((type) => WIDGET_CATALOG.find((w) => w.type === type))
      .filter((w): w is (typeof WIDGET_CATALOG)[number] => !!w)
  );
  const componentTemplates = $derived($project?.component_templates ?? []);

  function persistFavorites() {
    localStorage.setItem(FAVORITES_KEY, JSON.stringify(favorites));
  }

  function persistCollapsed() {
    localStorage.setItem(COLLAPSED_KEY, JSON.stringify(collapsed));
  }

  function isFavorite(type: string) {
    return favorites.includes(type);
  }

  function toggleFavorite(e: MouseEvent, type: string) {
    e.preventDefault();
    e.stopPropagation();
    if (favorites.includes(type)) {
      favorites = favorites.filter((t) => t !== type);
    } else {
      favorites = [...favorites, type];
    }
    persistFavorites();
  }

  function isCollapsed(key: string) {
    return !!collapsed[key];
  }

  function toggleCollapsed(key: string) {
    collapsed = { ...collapsed, [key]: !collapsed[key] };
    persistCollapsed();
  }

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

  /** Stop parent pointer-drag handlers from treating fav/+ clicks as item clicks. */
  function stopItemPointer(e: PointerEvent) {
    e.stopPropagation();
  }
</script>

<svelte:window onpointermove={onPointerMove} />

<div class="panel" style:height="100%;border:none">
  <div class="panel-header">
    <span>Toolbox (Click + or Drag)</span>
    <div class="header-actions">
      <span class="catalog-total">{CANONICAL_WIDGETS.length}/35</span>
      <VerticalScrollControls target={scrollContainer} />
    </div>
  </div>
  <div class="panel-body scrollable-panel-body" bind:this={scrollContainer}>
    <!-- Favorites (always first) -->
    <button
      type="button"
      class="tree-group collapsible"
      class:collapsed={isCollapsed("Favorites")}
      onclick={() => toggleCollapsed("Favorites")}
      aria-expanded={!isCollapsed("Favorites")}
    >
      <span class="chevron">{isCollapsed("Favorites") ? "▸" : "▾"}</span>
      <span>★ Favorites</span>
      <span class="count">{favoriteItems.length}</span>
    </button>
    {#if !isCollapsed("Favorites")}
      {#if favoriteItems.length === 0}
        <div class="empty-favorites">Star widgets to pin them here</div>
      {:else}
        {#each favoriteItems as item}
          <div
            class="toolbox-item"
            onpointerdown={(e) => onPointerDown(e, item.type)}
            onpointerup={onPointerUp}
            title={`${item.canonicalId} · ${item.description}`}
            role="group"
            aria-label={item.label}
          >
            <span class="icon">{item.icon}</span>
            <span class="item-label">{item.label}</span>
            <button
              type="button"
              class="btn-fav active"
              title="Remove from favorites"
              onpointerdown={stopItemPointer}
              onpointerup={stopItemPointer}
              onmousedown={(e) => e.stopPropagation()}
              onclick={(e) => toggleFavorite(e, item.type)}
            >
              ★
            </button>
            <button
              type="button"
              class="btn-quick-add"
              title="Add {item.label}"
              onpointerdown={stopItemPointer}
              onpointerup={stopItemPointer}
              onmousedown={(e) => e.stopPropagation()}
              onclick={(e) => handleQuickAdd(e, item.type)}
            >
              +
            </button>
          </div>
        {/each}
      {/if}
    {/if}

    <button
      type="button"
      class="tree-group collapsible"
      class:collapsed={isCollapsed("Custom Components")}
      onclick={() => toggleCollapsed("Custom Components")}
      aria-expanded={!isCollapsed("Custom Components")}
    >
      <span class="chevron">{isCollapsed("Custom Components") ? "▸" : "▾"}</span>
      <span>◆ Custom Components</span>
      <span class="count">{componentTemplates.length}</span>
    </button>
    {#if !isCollapsed("Custom Components")}
      {#each componentTemplates as component (component.id)}
        <div class="toolbox-item custom-component" title={`${component.description} · v${component.version}`}>
          <span class="icon">◆</span>
          <span class="item-label">{component.name}</span>
          <button
            type="button"
            class="btn-quick-add"
            title="Add {component.name}"
            onclick={() => instantiateComponentTemplate(component.id)}
          >
            +
          </button>
        </div>
      {:else}
        <div class="empty-favorites">Use Components to create or import reusable controls</div>
      {/each}
    {/if}

    {#each categories as cat}
      <button
        type="button"
        class="tree-group collapsible"
        class:collapsed={isCollapsed(cat)}
        onclick={() => toggleCollapsed(cat)}
        aria-expanded={!isCollapsed(cat)}
      >
        <span class="chevron">{isCollapsed(cat) ? "▸" : "▾"}</span>
        <span>{cat}</span>
        <span class="count">{WIDGET_CATALOG.filter((w) => w.category === cat).length}</span>
      </button>
      {#if !isCollapsed(cat)}
        {#each WIDGET_CATALOG.filter((w) => w.category === cat) as item}
          <div
            class="toolbox-item"
            onpointerdown={(e) => onPointerDown(e, item.type)}
            onpointerup={onPointerUp}
            title={`${item.canonicalId} · ${item.description}`}
            role="group"
            aria-label={item.label}
          >
            <span class="icon">{item.icon}</span>
            <span class="item-label">{item.label}</span>
            <button
              type="button"
              class="btn-fav"
              class:active={isFavorite(item.type)}
              title={isFavorite(item.type) ? "Remove from favorites" : "Add to favorites"}
              onpointerdown={stopItemPointer}
              onpointerup={stopItemPointer}
              onmousedown={(e) => e.stopPropagation()}
              onclick={(e) => toggleFavorite(e, item.type)}
            >
              {isFavorite(item.type) ? "★" : "☆"}
            </button>
            <button
              type="button"
              class="btn-quick-add"
              title="Add {item.label}"
              onpointerdown={stopItemPointer}
              onpointerup={stopItemPointer}
              onmousedown={(e) => e.stopPropagation()}
              onclick={(e) => handleQuickAdd(e, item.type)}
            >
              +
            </button>
          </div>
        {/each}
      {/if}
    {/each}
  </div>
</div>

<style>
  .tree-group.collapsible {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    border: none;
    background: transparent;
    padding: 6px 8px 2px;
    color: var(--vs-text-dim);
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    cursor: pointer;
    text-align: left;
  }
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .catalog-total {
    color: #86efac;
    font-size: 9px;
    font-weight: 800;
    letter-spacing: 0.04em;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .tree-group.collapsible:hover {
    color: var(--vs-text, #cccccc);
  }
  .chevron {
    width: 10px;
    flex-shrink: 0;
    font-size: 11px;
    line-height: 1;
  }
  .count {
    margin-left: auto;
    opacity: 0.55;
    font-weight: 600;
  }
  .empty-favorites {
    padding: 4px 8px 8px 22px;
    font-size: 10px;
    color: var(--vs-text-dim);
    opacity: 0.7;
  }
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
  .btn-fav {
    background: transparent;
    border: none;
    color: var(--vs-text-dim, #888);
    font-size: 12px;
    width: 20px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    opacity: 0;
    flex-shrink: 0;
    padding: 0;
  }
  .toolbox-item:hover .btn-fav,
  .btn-fav.active {
    opacity: 1;
  }
  .btn-fav.active {
    color: #eab308;
  }
  .btn-fav:hover {
    color: #eab308;
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
