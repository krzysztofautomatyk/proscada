<script lang="ts">
  import {
    cutSelectedWidgets,
    copySelectedWidgets,
    multiCopySelected,
    pasteWidgets,
    deleteSelectedWidget,
    duplicateSelected,
    toggleLockSelected,
    groupSelectedWidgets,
    ungroupSelectedWidgets,
    alignSelectedWidgets,
    bringSelectedToFront,
    sendSelectedToBack,
    bringSelectedForward,
    sendSelectedBackward,
    selectAllWidgets,
    openAttributesPanel,
    clipboard,
    selectedWidgetIds,
    selectedWidgetId,
  } from "$lib/stores/app";

  interface Props {
    x: number;
    y: number;
    open: boolean;
    onClose: () => void;
  }

  let { x, y, open, onClose }: Props = $props();

  const hasSel = $derived(
    ($selectedWidgetIds?.length ?? 0) > 0 || !!$selectedWidgetId,
  );
  const hasClip = $derived(($clipboard?.length ?? 0) > 0);
  const multi = $derived(($selectedWidgetIds?.length ?? 0) > 1);

  function run(fn: () => void) {
    fn();
    onClose();
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="ctx-backdrop" onpointerdown={onClose}></div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="ctx-menu"
    style:left="{x}px"
    style:top="{y}px"
    role="menu"
    tabindex="-1"
    onkeydown={onKey}
    onpointerdown={(e) => e.stopPropagation()}
  >
    <button type="button" role="menuitem" disabled={!hasSel} onclick={() => run(cutSelectedWidgets)}>
      <span>Cut</span><kbd>Ctrl+X</kbd>
    </button>
    <button type="button" role="menuitem" disabled={!hasSel} onclick={() => run(copySelectedWidgets)}>
      <span>Copy</span><kbd>Ctrl+C</kbd>
    </button>
    <button type="button" role="menuitem" disabled={!hasSel} onclick={() => run(duplicateSelected)}>
      <span>Duplicate</span><kbd>Ctrl+D</kbd>
    </button>
    <button
      type="button"
      role="menuitem"
      disabled={!hasSel}
      onclick={() => run(() => multiCopySelected(3, 24, 24))}
    >
      <span>Multi-copy ×3</span>
    </button>
    <button type="button" role="menuitem" disabled={!hasClip} onclick={() => run(() => pasteWidgets())}>
      <span>Paste</span><kbd>Ctrl+V</kbd>
    </button>
    <div class="sep"></div>
    <button type="button" role="menuitem" disabled={!hasSel} onclick={() => run(deleteSelectedWidget)}>
      <span>Delete</span><kbd>Del</kbd>
    </button>
    <div class="sep"></div>
    <button type="button" role="menuitem" disabled={!hasSel} onclick={() => run(toggleLockSelected)}>
      <span>Pin / Lock toggle</span>
    </button>
    <button
      type="button"
      role="menuitem"
      disabled={!multi}
      onclick={() => run(groupSelectedWidgets)}
    >
      <span>Group (2+ selected)</span><kbd>Ctrl+G</kbd>
    </button>
    <button type="button" role="menuitem" disabled={!hasSel} onclick={() => run(ungroupSelectedWidgets)}>
      <span>Ungroup</span><kbd>Ctrl+Shift+G</kbd>
    </button>
    {#if multi}
      <div class="sep"></div>
      <button type="button" role="menuitem" onclick={() => run(() => alignSelectedWidgets("left"))}>
        <span>Align Left (Do lewej)</span><kbd>⇤</kbd>
      </button>
      <button type="button" role="menuitem" onclick={() => run(() => alignSelectedWidgets("center"))}>
        <span>Align Center H (Do środka H)</span><kbd>↔</kbd>
      </button>
      <button type="button" role="menuitem" onclick={() => run(() => alignSelectedWidgets("right"))}>
        <span>Align Right (Do prawej)</span><kbd>⇥</kbd>
      </button>
      <button type="button" role="menuitem" onclick={() => run(() => alignSelectedWidgets("top"))}>
        <span>Align Top (Do góry)</span><kbd>⤒</kbd>
      </button>
      <button type="button" role="menuitem" onclick={() => run(() => alignSelectedWidgets("middle"))}>
        <span>Align Middle V (Do środka V)</span><kbd>↕</kbd>
      </button>
      <button type="button" role="menuitem" onclick={() => run(() => alignSelectedWidgets("bottom"))}>
        <span>Align Bottom (Do dołu)</span><kbd>⤓</kbd>
      </button>
    {/if}
    <div class="sep"></div>
    <button type="button" role="menuitem" disabled={!hasSel} onclick={() => run(bringSelectedToFront)}>
      <span>Bring to Front</span>
    </button>
    <button type="button" role="menuitem" disabled={!hasSel} onclick={() => run(bringSelectedForward)}>
      <span>Bring Forward</span>
    </button>
    <button type="button" role="menuitem" disabled={!hasSel} onclick={() => run(sendSelectedBackward)}>
      <span>Send Backward</span>
    </button>
    <button type="button" role="menuitem" disabled={!hasSel} onclick={() => run(sendSelectedToBack)}>
      <span>Send to Back</span>
    </button>
    <div class="sep"></div>
    <button type="button" role="menuitem" onclick={() => run(selectAllWidgets)}>
      <span>Select All</span><kbd>Ctrl+A</kbd>
    </button>
    <button
      type="button"
      role="menuitem"
      disabled={!hasSel}
      onclick={() => run(openAttributesPanel)}
    >
      <span>Attributes…</span><kbd>F4</kbd>
    </button>
  </div>
{/if}

<style>
  .ctx-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9998;
  }
  .ctx-menu {
    position: fixed;
    z-index: 9999;
    min-width: 220px;
    background: #2d2d30;
    border: 1px solid #3e3e42;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    padding: 4px 0;
    border-radius: 4px;
    font-size: 12px;
    color: #cccccc;
  }
  .ctx-menu button {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 6px 12px;
    border: none;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
    font-size: 12px;
  }
  .ctx-menu button:hover:not(:disabled) {
    background: #094771;
    color: #fff;
  }
  .ctx-menu button:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .ctx-menu kbd {
    font-size: 10px;
    color: #9d9d9d;
    font-family: inherit;
  }
  .sep {
    height: 1px;
    background: #3e3e42;
    margin: 4px 0;
  }
</style>
