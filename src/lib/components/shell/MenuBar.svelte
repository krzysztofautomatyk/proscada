<script lang="ts">
  import { onMount } from "svelte";
  import type { LeftPanelTab } from "$lib/types";
  import {
    mode,
    activeForm,
    project,
    dirty,
    clipboard,
    selectedWidgetId,
    selectedWidgetIds,
    newBlankProject,
    importProjectFile,
    persistProject,
    exportProjectJson,
    addNewForm,
    deleteForm,
    switchMode,
    connectDevice,
    disconnectDevice,
    refreshAudit,
    cutSelectedWidgets,
    copySelectedWidgets,
    pasteWidgets,
    duplicateSelected,
    deleteSelectedWidget,
    selectAllWidgets,
    groupSelectedWidgets,
    ungroupSelectedWidgets,
    undoAction,
    redoAction,
    startWindowOpen,
    canUndo,
    canRedo,
    undoLabel,
    redoLabel,
    addDeviceModalOpen,
    addAlarmModalOpen,
    addVariableModalOpen,
  } from "$lib/stores/app";

  interface Props {
    leftTab: LeftPanelTab;
    onLeftTab: (tab: LeftPanelTab) => void;
    onNewWaterTank: () => void;
    onOpenSettings?: () => void;
  }

  let { leftTab, onLeftTab, onNewWaterTank, onOpenSettings }: Props = $props();

  type MenuId = "file" | "edit" | "view" | "project" | "debug" | "tools" | "help";

  let openMenu = $state<MenuId | null>(null);
  let rootEl = $state<HTMLDivElement | null>(null);

  const design = $derived($mode === "designer");
  const hasSel = $derived(
    ($selectedWidgetIds?.length ?? 0) > 0 || !!$selectedWidgetId,
  );
  const hasClip = $derived(($clipboard?.length ?? 0) > 0);
  const canDeleteScreen = $derived(
    !!$activeForm && ($project?.forms.length ?? 0) > 1,
  );

  const menus: { id: MenuId; label: string }[] = [
    { id: "file", label: "File" },
    { id: "edit", label: "Edit" },
    { id: "view", label: "View" },
    { id: "project", label: "Project" },
    { id: "debug", label: "Debug" },
    { id: "tools", label: "Tools" },
    { id: "help", label: "Help" },
  ];

  function close() {
    openMenu = null;
  }

  function toggle(id: MenuId) {
    openMenu = openMenu === id ? null : id;
  }

  function hoverOpen(id: MenuId) {
    if (openMenu !== null) openMenu = id;
  }

  function run(fn: () => unknown) {
    void Promise.resolve(fn());
    close();
  }

  function newProject() {
    const name = prompt("Project name", "New Project");
    if (name) newBlankProject(name);
  }

  function about() {
    alert(
      "ProScada v1.0 — Engineering Workstation\nVisual Studio–style SCADA designer & runtime\nModbus TCP master · Lab use only",
    );
  }

  onMount(() => {
    const onDoc = (e: PointerEvent) => {
      if (!rootEl?.contains(e.target as Node)) close();
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") close();
    };
    document.addEventListener("pointerdown", onDoc);
    window.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("pointerdown", onDoc);
      window.removeEventListener("keydown", onKey);
    };
  });
</script>

<div class="vs-menubar" bind:this={rootEl} role="menubar">
  {#each menus as m}
    <div class="vs-menu-root">
      <button
        type="button"
        class="vs-menu-top"
        class:open={openMenu === m.id}
        role="menuitem"
        aria-haspopup="true"
        aria-expanded={openMenu === m.id}
        onclick={() => toggle(m.id)}
        onmouseenter={() => hoverOpen(m.id)}
      >
        {m.label}
      </button>

      {#if openMenu === m.id}
        <div class="vs-menu-drop" role="menu">
          {#if m.id === "file"}
            <button type="button" role="menuitem" onclick={() => run(() => startWindowOpen.set(true))}>
              <span>Start Window / Ekran startowy…</span><kbd>Ctrl+Shift+W</kbd>
            </button>
            <div class="vs-menu-sep"></div>
            <button type="button" role="menuitem" onclick={() => run(newProject)}>
              <span>New Project…</span><kbd>Ctrl+Shift+N</kbd>
            </button>
            <button type="button" role="menuitem" onclick={() => run(onNewWaterTank)}>
              <span>New Water Tank Project</span>
            </button>
            <button type="button" role="menuitem" onclick={() => run(importProjectFile)}>
              <span>Open / Import…</span><kbd>Ctrl+O</kbd>
            </button>
            <div class="vs-menu-sep"></div>
            <button type="button" role="menuitem" onclick={() => run(persistProject)}>
              <span>Save</span><kbd>Ctrl+S</kbd>
            </button>
            <button type="button" role="menuitem" onclick={() => run(exportProjectJson)}>
              <span>Export JSON…</span>
            </button>
            <div class="vs-menu-sep"></div>
            <button type="button" role="menuitem" onclick={() => run(() => onOpenSettings?.())}>
              <span>⚙️ Application Settings…</span>
            </button>
            {#if $dirty}
              <div class="vs-menu-hint">Unsaved changes</div>
            {/if}

          {:else if m.id === "edit"}
            <button
              type="button"
              role="menuitem"
              disabled={!design || !$canUndo}
              onclick={() => run(undoAction)}
            >
              <span>Undo {$undoLabel ? `(${ $undoLabel })` : ""}</span><kbd>Ctrl+Z</kbd>
            </button>
            <button
              type="button"
              role="menuitem"
              disabled={!design || !$canRedo}
              onclick={() => run(redoAction)}
            >
              <span>Redo {$redoLabel ? `(${ $redoLabel })` : ""}</span><kbd>Ctrl+Y</kbd>
            </button>
            <div class="vs-menu-sep"></div>
            <button
              type="button"
              role="menuitem"
              disabled={!design || !hasSel}
              onclick={() => run(cutSelectedWidgets)}
            >
              <span>Cut</span><kbd>Ctrl+X</kbd>
            </button>
            <button
              type="button"
              role="menuitem"
              disabled={!design || !hasSel}
              onclick={() => run(copySelectedWidgets)}
            >
              <span>Copy</span><kbd>Ctrl+C</kbd>
            </button>
            <button
              type="button"
              role="menuitem"
              disabled={!design || !hasClip}
              onclick={() => run(() => pasteWidgets())}
            >
              <span>Paste</span><kbd>Ctrl+V</kbd>
            </button>
            <button
              type="button"
              role="menuitem"
              disabled={!design || !hasSel}
              onclick={() => run(duplicateSelected)}
            >
              <span>Duplicate</span><kbd>Ctrl+D</kbd>
            </button>
            <div class="vs-menu-sep"></div>
            <button
              type="button"
              role="menuitem"
              disabled={!design}
              onclick={() => run(selectAllWidgets)}
            >
              <span>Select All</span><kbd>Ctrl+A</kbd>
            </button>
            <button
              type="button"
              role="menuitem"
              disabled={!design || !hasSel}
              onclick={() => run(deleteSelectedWidget)}
            >
              <span>Delete</span><kbd>Del</kbd>
            </button>
            <div class="vs-menu-sep"></div>
            <button
              type="button"
              role="menuitem"
              disabled={!design || !hasSel}
              onclick={() => run(groupSelectedWidgets)}
            >
              <span>Group</span><kbd>Ctrl+G</kbd>
            </button>
            <button
              type="button"
              role="menuitem"
              disabled={!design || !hasSel}
              onclick={() => run(ungroupSelectedWidgets)}
            >
              <span>Ungroup</span><kbd>Ctrl+Shift+G</kbd>
            </button>

          {:else if m.id === "view"}
            <button
              type="button"
              role="menuitem"
              class:checked={design && leftTab === "solution"}
              disabled={!design}
              onclick={() => run(() => onLeftTab("solution"))}
            >
              <span>Solution Explorer</span>
            </button>
            <button
              type="button"
              role="menuitem"
              class:checked={design && leftTab === "toolbox"}
              disabled={!design}
              onclick={() => run(() => onLeftTab("toolbox"))}
            >
              <span>Toolbox</span>
            </button>
            <button
              type="button"
              role="menuitem"
              class:checked={design && leftTab === "objects"}
              disabled={!design}
              onclick={() => run(() => onLeftTab("objects"))}
            >
              <span>Document Outline</span>
            </button>
            <button
              type="button"
              role="menuitem"
              class:checked={design && leftTab === "designSystem"}
              disabled={!design}
              onclick={() => run(() => onLeftTab("designSystem"))}
            >
              <span>Project Styles / Fonts / Animations</span>
            </button>
            <button
              type="button"
              role="menuitem"
              class:checked={design && leftTab === "components"}
              disabled={!design}
              onclick={() => run(() => onLeftTab("components"))}
            >
              <span>Component Library</span>
            </button>
            <button
              type="button"
              role="menuitem"
              class:checked={design && leftTab === "alarms"}
              disabled={!design}
              onclick={() => run(() => onLeftTab("alarms"))}
            >
              <span>Central Alarm Manager</span>
            </button>
            <div class="vs-menu-sep"></div>
            <button
              type="button"
              role="menuitem"
              class:checked={design}
              onclick={() => run(() => switchMode("designer"))}
            >
              <span>Design</span>
            </button>
            <button
              type="button"
              role="menuitem"
              class:checked={!design}
              onclick={() => run(() => switchMode("runtime"))}
            >
              <span>Runtime</span>
            </button>

          {:else if m.id === "project"}
            <button
              type="button"
              role="menuitem"
              disabled={!design}
              onclick={() => run(addNewForm)}
            >
              <span>🗂 Add New Screen…</span>
            </button>

            <div class="vs-menu-sep"></div>

            <button
              type="button"
              role="menuitem"
              disabled={!design}
              onclick={() => run(() => addDeviceModalOpen.set(true))}
            >
              <span>🔌 Add Modbus Device…</span>
            </button>

            <button
              type="button"
              role="menuitem"
              disabled={!design}
              onclick={() => run(() => addAlarmModalOpen.set(true))}
            >
              <span>🔔 Add Alarm Rules / List…</span>
            </button>

            <button
              type="button"
              role="menuitem"
              disabled={!design}
              onclick={() => run(() => addVariableModalOpen.set(true))}
            >
              <span>🏷️ Add Variables / Tag List…</span>
            </button>

            <div class="vs-menu-sep"></div>

            <button
              type="button"
              role="menuitem"
              disabled={!design || !canDeleteScreen}
              onclick={() =>
                run(() => {
                  if ($activeForm) deleteForm($activeForm.id);
                })}
            >
              <span>Delete Current Screen</span>
            </button>

          {:else if m.id === "debug"}
            {#if design}
              <button
                type="button"
                role="menuitem"
                class="accent"
                onclick={() => run(() => switchMode("runtime"))}
              >
                <span>Start Runtime</span><kbd>F5</kbd>
              </button>
            {:else}
              <button
                type="button"
                role="menuitem"
                onclick={() => run(() => switchMode("designer"))}
              >
                <span>Stop Debugging</span><kbd>Shift+F5</kbd>
              </button>
            {/if}

          {:else if m.id === "tools"}
            <button type="button" role="menuitem" onclick={() => run(connectDevice)}>
              <span>Connect Modbus / Start Poll</span>
            </button>
            <button type="button" role="menuitem" onclick={() => run(disconnectDevice)}>
              <span>Disconnect / Stop Poll</span>
            </button>
            <div class="vs-menu-sep"></div>
            <button type="button" role="menuitem" onclick={() => run(refreshAudit)}>
              <span>Refresh Audit Trail</span>
            </button>

          {:else if m.id === "help"}
            <button type="button" role="menuitem" onclick={() => run(about)}>
              <span>About ProScada</span>
            </button>
          {/if}
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .vs-menubar {
    display: flex;
    align-items: stretch;
    height: 100%;
    padding: 0 4px;
    background: var(--vs-bg-2);
    border-bottom: 1px solid var(--vs-border);
    user-select: none;
  }

  .vs-menu-root {
    position: relative;
    display: flex;
    align-items: stretch;
  }

  .vs-menu-top {
    padding: 0 10px;
    border: none;
    border-radius: 0;
    background: transparent;
    color: var(--vs-text);
    font: inherit;
    font-size: 12px;
    cursor: default;
  }

  .vs-menu-top:hover,
  .vs-menu-top.open {
    background: var(--vs-selection);
    color: var(--vs-text-bright);
  }

  .vs-menu-drop {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 5000;
    min-width: 240px;
    padding: 4px 0;
    background: var(--vs-menu, #2c2c2c);
    border: 1px solid var(--vs-border);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.45);
  }

  .vs-menu-drop button {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    width: 100%;
    padding: 5px 12px 5px 28px;
    border: none;
    border-radius: 0;
    background: transparent;
    color: var(--vs-text);
    font: inherit;
    font-size: 12px;
    text-align: left;
    cursor: default;
  }

  .vs-menu-drop button:hover:not(:disabled) {
    background: var(--vs-selection);
    color: #fff;
  }

  .vs-menu-drop button:disabled {
    opacity: 0.4;
  }

  .vs-menu-drop button.checked::before {
    content: "✓";
    position: absolute;
    left: 10px;
    font-size: 11px;
  }

  .vs-menu-drop button.checked {
    position: relative;
  }

  .vs-menu-drop button.accent {
    color: #89d185;
  }

  .vs-menu-drop kbd {
    margin-left: auto;
    color: var(--vs-text-dim);
    font: inherit;
    font-size: 11px;
  }

  .vs-menu-drop button:hover:not(:disabled) kbd {
    color: #ddd;
  }

  .vs-menu-sep {
    height: 1px;
    margin: 4px 8px;
    background: var(--vs-border);
  }

  .vs-menu-hint {
    padding: 4px 12px 4px 28px;
    color: var(--vs-warn, #eab308);
    font-size: 11px;
  }
</style>
