<script lang="ts">
  import type { ProjectNode, ProjectNodeKind, ScadaProject } from "$lib/types";
  import {
    selectedFormId,
    selectedNodeId,
    selectSolutionNode,
    addNewForm,
    addProjectFolder,
    addProjectDocument,
    importImageFiles,
    deleteProjectNode,
    renameProjectNode,
    toggleFolderCollapsed,
    moveProjectNode,
    newBlankProject,
    importProjectFile,
    exportProjectJson,
    persistProject,
    addDeviceModalOpen,
    addAlarmModalOpen,
    addVariableModalOpen,
    openAddDeviceModal,
    openEditDeviceModal,
    deleteDeviceFromProject,
    isMainScreen,
  } from "$lib/stores/app";
  import { childrenOf, iconFor } from "$lib/utils/projectTree";

  interface Props {
    project: ScadaProject;
    design: boolean;
  }

  let { project, design }: Props = $props();

  type Ctx = {
    open: boolean;
    x: number;
    y: number;
    node: ProjectNode | null;
    /** true when opened on empty area / project root */
    onRoot: boolean;
  };

  let ctx = $state<Ctx>({ open: false, x: 0, y: 0, node: null, onRoot: false });
  let renamingId = $state<string | null>(null);
  let renameValue = $state("");
  let dragId = $state<string | null>(null);
  let fileInputEl = $state<HTMLInputElement | null>(null);
  let importTargetFolderId = $state<string | null>(null);

  type HoverImage = {
    node: ProjectNode;
    x: number;
    y: number;
  };
  let hoverImage = $state<HoverImage | null>(null);

  const tree = $derived(project.tree ?? []);

  function closeCtx() {
    ctx = { ...ctx, open: false };
  }

  function openCtx(e: MouseEvent, node: ProjectNode | null, onRoot = false) {
    e.preventDefault();
    e.stopPropagation();
    ctx = { open: true, x: e.clientX, y: e.clientY, node, onRoot };
  }

  function parentForAdd(): string | null {
    if (ctx.node?.kind === "folder") return ctx.node.id;
    if (ctx.node?.parent_id) return ctx.node.parent_id;
    return null;
  }

  function triggerImportImage(parentId: string | null = null) {
    importTargetFolderId = parentId;
    fileInputEl?.click();
  }

  async function handleImageFileInput(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      await importImageFiles(input.files, importTargetFolderId);
      input.value = "";
    }
  }

  function handleNodeMouseEnter(node: ProjectNode, e: MouseEvent) {
    if (node.kind === "image" && node.content) {
      hoverImage = { node, x: e.clientX, y: e.clientY };
    }
  }

  function handleNodeMouseMove(node: ProjectNode, e: MouseEvent) {
    if (hoverImage && hoverImage.node.id === node.id) {
      hoverImage = { node, x: e.clientX, y: e.clientY };
    }
  }

  function handleNodeMouseLeave(node: ProjectNode) {
    if (hoverImage?.node.id === node.id) {
      hoverImage = null;
    }
  }

  function startRename(node: ProjectNode) {
    if (node.kind === "screen" && isMainScreen(node)) return;
    renamingId = node.id;
    renameValue = node.name;
    closeCtx();
    queueMicrotask(() => {
      const el = document.getElementById(`rename-${node.id}`) as HTMLInputElement | null;
      el?.focus();
      el?.select();
    });
  }

  function commitRename() {
    if (renamingId && renameValue.trim()) {
      renameProjectNode(renamingId, renameValue.trim());
    }
    renamingId = null;
  }

  function onSelect(node: ProjectNode) {
    selectSolutionNode(node.id);
  }

  function onFolderClick(node: ProjectNode, e: MouseEvent) {
    e.stopPropagation();
    toggleFolderCollapsed(node.id);
    selectSolutionNode(node.id);
  }

  function runAdd(kind: "folder" | "screen" | ProjectNodeKind) {
    const parent = parentForAdd();
    closeCtx();
    if (kind === "folder") addProjectFolder(parent);
    else if (kind === "screen") addNewForm(undefined, undefined, undefined, undefined, undefined, parent);
    else addProjectDocument(kind as Exclude<ProjectNodeKind, "folder" | "screen">, parent);
  }

  function onDragStart(e: DragEvent, node: ProjectNode) {
    if (!design) return;
    dragId = node.id;
    e.dataTransfer?.setData("text/proscada-node", node.id);
    e.dataTransfer!.effectAllowed = "move";
  }

  function onDragOver(e: DragEvent) {
    if (!design) return;
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = "move";
    }
  }

  async function onDrop(e: DragEvent, target: ProjectNode | null) {
    e.preventDefault();
    e.stopPropagation();
    if (!design) return;

    if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
      const parentId = target?.kind === "folder" ? target.id : target?.parent_id ?? null;
      await importImageFiles(e.dataTransfer.files, parentId);
      return;
    }

    const id = e.dataTransfer?.getData("text/proscada-node") || dragId;
    dragId = null;
    if (!id) return;
    if (target?.kind === "folder") moveProjectNode(id, target.id);
    else if (target) moveProjectNode(id, target.parent_id);
    else moveProjectNode(id, null);
  }

  function renderBranch(parentId: string | null, depth: number): ProjectNode[] {
    return childrenOf(tree, parentId);
  }
</script>

<input
  type="file"
  accept="image/*,.svg,.png,.jpg,.jpeg,.gif,.webp"
  multiple
  bind:this={fileInputEl}
  onchange={handleImageFileInput}
  style="display:none;"
/>

{#snippet treeRows(parentId: string | null, depth: number)}
  {#each renderBranch(parentId, depth) as node (node.id)}
    {@const collapsed = node.kind === "folder" && (node.collapsed ?? false)}
    {@const active =
      $selectedNodeId === node.id ||
      (node.kind === "screen" && node.ref_id === $selectedFormId)}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="tree-item tree-row"
      class:active
      class:folder={node.kind === "folder"}
      style:padding-left="{10 + depth * 14}px"
      role="treeitem"
      tabindex="0"
      aria-selected={active}
      draggable={design}
      ondragstart={(e) => onDragStart(e, node)}
      ondragover={onDragOver}
      ondrop={(e) => onDrop(e, node)}
      onmouseenter={(e) => handleNodeMouseEnter(node, e)}
      onmousemove={(e) => handleNodeMouseMove(node, e)}
      onmouseleave={() => handleNodeMouseLeave(node)}
      onclick={(e) => {
        if (node.kind === "folder") onFolderClick(node, e);
        else onSelect(node);
      }}
      ondblclick={() => design && startRename(node)}
      oncontextmenu={(e) => openCtx(e, node)}
      onkeydown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          if (node.kind === "folder") toggleFolderCollapsed(node.id);
          else onSelect(node);
        }
        if (e.key === "F2" && design) startRename(node);
        if ((e.key === "Delete" || e.key === "Backspace") && design && !renamingId) {
          e.preventDefault();
          deleteProjectNode(node.id);
        }
      }}
    >
      {#if node.kind === "folder"}
        <span class="twist">{collapsed ? "▸" : "▾"}</span>
      {:else}
        <span class="twist spacer"></span>
      {/if}
      <span class="ico">{iconFor(node.kind)}</span>
      {#if renamingId === node.id}
        <input
          id="rename-{node.id}"
          class="rename-input"
          bind:value={renameValue}
          onclick={(e) => e.stopPropagation()}
          onkeydown={(e) => {
            if (e.key === "Enter") commitRename();
            if (e.key === "Escape") renamingId = null;
            e.stopPropagation();
          }}
          onblur={commitRename}
        />
      {:else}
        <span class="label">{node.name}{node.kind === "screen" && !node.name.endsWith(".form") ? ".form" : ""}</span>
      {/if}
    </div>
    {#if node.kind === "folder" && !collapsed}
      {@render treeRows(node.id, depth + 1)}
    {/if}
  {/each}
{/snippet}

<div class="panel se-panel" style:height="100%;border:none;border-right:1px solid var(--vs-border)">
  <div class="panel-header se-header">
    <span>Solution Explorer</span>
    {#if design}
      <div class="se-actions">
        <button type="button" title="New folder" onclick={() => addProjectFolder(null)}>📁+</button>
        <button type="button" title="Add image from disk..." onclick={() => triggerImportImage(null)}>🖼️+</button>
        <button type="button" title="New style sheet" onclick={() => addProjectDocument("style")}>🎨+</button>
        <button type="button" title="New screen" onclick={() => addNewForm()}>🗂+</button>
        <button type="button" title="New script" onclick={() => addProjectDocument("script")}>📜+</button>
      </div>
    {/if}
  </div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="panel-body se-body"
    role="tree"
    tabindex="0"
    oncontextmenu={(e) => openCtx(e, null, true)}
    ondragover={onDragOver}
    ondrop={(e) => onDrop(e, null)}
  >
    <!-- Project root -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="tree-item tree-row project-root"
      class:active={!$selectedNodeId}
      role="treeitem"
      tabindex="0"
      aria-selected={!$selectedNodeId}
      onclick={() => selectSolutionNode(null)}
      onkeydown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          selectSolutionNode(null);
        }
      }}
      oncontextmenu={(e) => openCtx(e, null, true)}
    >
      <span class="ico">📦</span>
      <span class="label">{project.name}</span>
    </div>

    {@render treeRows(null, 1)}

    <div class="tree-group-header">
      <span class="tree-group">Devices ({project.devices.length})</span>
      {#if design}
        <button type="button" class="btn-group-add" title="Dodaj Nowe Urządzenie..." onclick={() => openAddDeviceModal()}>🔌+</button>
      {/if}
    </div>
    {#each project.devices as d}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="tree-item device-row"
        style:padding-left="24px"
        role="button"
        tabindex="0"
        title="Kliknij, aby edytować urządzenie {d.name}"
        onclick={() => design && openEditDeviceModal(d.id)}
        onkeydown={(e) => (e.key === "Enter" || e.key === " ") && design && openEditDeviceModal(d.id)}
      >
        <span class="ico">🔌</span>
        <span class="label">{d.name} · {d.host}:{d.port}</span>
        {#if design}
          <button
            type="button"
            class="btn-device-edit"
            title="Edytuj urządzenia..."
            onclick={(e) => {
              e.stopPropagation();
              openEditDeviceModal(d.id);
            }}
          >
            ✏️
          </button>
        {/if}
      </div>
    {/each}

    <div class="tree-group-header">
      <span class="tree-group">Alarms ({project.alarms.length})</span>
      {#if design}
        <button type="button" class="btn-group-add" title="Dodaj Nowy Alarm..." onclick={() => addAlarmModalOpen.set(true)}>🔔+</button>
      {/if}
    </div>
    {#each project.alarms as a}
      <div class="tree-item" style:padding-left="24px">
        <span>🔔</span>
        <span>{a.name}</span>
      </div>
    {/each}

    <div class="meta">
      Schema v{project.schema_version}<br />
      Hash: {(project.content_hash || "—").slice(0, 12)}…
    </div>
  </div>
</div>

{#if ctx.open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="ctx-backdrop" onpointerdown={closeCtx}></div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="ctx-menu"
    style:left="{ctx.x}px"
    style:top="{ctx.y}px"
    role="menu"
    tabindex="-1"
    onpointerdown={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === "Escape" && closeCtx()}
  >
    {#if design}
      <div class="ctx-label">Add</div>
      <button type="button" role="menuitem" onclick={() => runAdd("folder")}>New Folder</button>
      <button
        type="button"
        role="menuitem"
        onclick={() => {
          const p = parentForAdd();
          closeCtx();
          triggerImportImage(p);
        }}
      >
        🖼️ Import Image from Disk...
      </button>
      <button type="button" role="menuitem" onclick={() => runAdd("style")}>New Style Sheet (.css)</button>
      <button type="button" role="menuitem" onclick={() => runAdd("screen")}>New HMI Screen</button>
      <button type="button" role="menuitem" onclick={() => runAdd("script")}>New Script (.js)</button>
      <button type="button" role="menuitem" onclick={() => runAdd("variables")}>New Variables List</button>
      <button
        type="button"
        role="menuitem"
        onclick={() => {
          closeCtx();
          addDeviceModalOpen.set(true);
        }}
      >
        🔌 New Modbus Device…
      </button>
      <button
        type="button"
        role="menuitem"
        onclick={() => {
          closeCtx();
          addAlarmModalOpen.set(true);
        }}
      >
        🔔 New Alarm Rules / List…
      </button>
      <button
        type="button"
        role="menuitem"
        onclick={() => {
          closeCtx();
          addVariableModalOpen.set(true);
        }}
      >
        🏷️ New Variables / Tag List…
      </button>
      <button type="button" role="menuitem" onclick={() => runAdd("note")}>New Note</button>
      <button type="button" role="menuitem" onclick={() => runAdd("markdown")}>New Markdown</button>
      <div class="sep"></div>
      {#if ctx.node}
        <button
          type="button"
          role="menuitem"
          disabled={ctx.node.kind === "screen" && isMainScreen(ctx.node)}
          onclick={() => ctx.node && startRename(ctx.node)}
        >
          Rename<span class="kbd">F2</span>
        </button>
        <button
          type="button"
          role="menuitem"
          disabled={ctx.node.kind === "screen" && isMainScreen(ctx.node)}
          onclick={() => {
            if (ctx.node) deleteProjectNode(ctx.node.id);
            closeCtx();
          }}
        >
          Delete<span class="kbd">Del</span>
        </button>
        <div class="sep"></div>
      {/if}
      <button
        type="button"
        role="menuitem"
        onclick={() => {
          closeCtx();
          const name = prompt("Project name", "New Project");
          if (name) newBlankProject(name);
        }}
      >
        New Project…
      </button>
      <button
        type="button"
        role="menuitem"
        onclick={() => {
          closeCtx();
          importProjectFile();
        }}
      >
        Import Project…
      </button>
      <button
        type="button"
        role="menuitem"
        onclick={() => {
          closeCtx();
          exportProjectJson();
        }}
      >
        Export Project…
      </button>
      <button
        type="button"
        role="menuitem"
        onclick={() => {
          closeCtx();
          persistProject();
        }}
      >
        Save Project<span class="kbd">Ctrl+S</span>
      </button>
    {:else}
      <button type="button" role="menuitem" disabled>Read-only in Runtime</button>
    {/if}
  </div>
{/if}

{#if hoverImage && hoverImage.node.content}
  <div
    class="image-hover-popover"
    style:left="{Math.min(hoverImage.x + 16, (typeof window !== 'undefined' ? window.innerWidth : 800) - 220)}px"
    style:top="{Math.min(hoverImage.y + 12, (typeof window !== 'undefined' ? window.innerHeight : 600) - 220)}px"
  >
    <div class="hover-img-wrap">
      <img src={hoverImage.node.content} alt={hoverImage.node.name} />
    </div>
    <div class="hover-img-meta">
      <span class="hover-img-name">{hoverImage.node.name}</span>
      <span class="hover-img-size">
        {Math.round((hoverImage.node.content.length * 0.75) / 1024)} KB
      </span>
    </div>
  </div>
{/if}

<style>
  .se-panel {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .se-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
  }
  .se-actions {
    display: flex;
    gap: 2px;
  }
  .se-actions button {
    background: transparent;
    border: 1px solid transparent;
    color: var(--vs-text);
    font-size: 11px;
    padding: 0 4px;
    cursor: pointer;
    border-radius: 3px;
    line-height: 18px;
  }
  .se-actions button:hover {
    background: var(--vs-bg-4);
    border-color: var(--vs-border);
  }
  .se-body {
    overflow: auto;
    flex: 1;
  }
  .tree-row {
    user-select: none;
    gap: 4px;
  }
  .project-root {
    font-weight: 700;
    color: var(--vs-text-bright);
  }
  .twist {
    width: 12px;
    flex-shrink: 0;
    font-size: 10px;
    color: var(--vs-text-dim);
    text-align: center;
  }
  .twist.spacer {
    visibility: hidden;
  }
  .ico {
    flex-shrink: 0;
    width: 16px;
    text-align: center;
    font-size: 12px;
    font-weight: 800;
    color: #89d185;
  }
  .label {
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }
  .rename-input {
    flex: 1;
    min-width: 0;
    background: #3c3c3c;
    border: 1px solid var(--vs-accent);
    color: #fff;
    font-size: 12px;
    padding: 1px 4px;
    border-radius: 2px;
  }
  .meta {
    padding: 10px;
    font-size: 10px;
    color: var(--vs-text-dim);
  }
  .ctx-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9998;
  }
  .ctx-menu {
    position: fixed;
    z-index: 9999;
    min-width: 220px;
    max-height: min(80vh, 520px);
    overflow: auto;
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
    padding: 6px 14px;
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
  .ctx-label {
    padding: 4px 14px 2px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    color: var(--vs-text-dim);
    letter-spacing: 0.04em;
  }
  .kbd {
    font-size: 10px;
    color: #9d9d9d;
  }
  .sep {
    height: 1px;
    background: #3e3e42;
    margin: 4px 0;
  }
  .image-hover-popover {
    position: fixed;
    z-index: 10000;
    pointer-events: none;
    background: #1e1e24;
    border: 1px solid #3b82f6;
    box-shadow: 0 10px 28px rgba(0, 0, 0, 0.6);
    border-radius: 6px;
    padding: 8px;
    width: 200px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    backdrop-filter: blur(8px);
    animation: popover-fade-in 0.12s ease-out;
  }
  .hover-img-wrap {
    width: 100%;
    max-height: 160px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: radial-gradient(circle, #2a2a30 0%, #121214 100%);
    border-radius: 4px;
    overflow: hidden;
    padding: 4px;
  }
  .hover-img-wrap img {
    max-width: 100%;
    max-height: 150px;
    object-fit: contain;
    display: block;
  }
  .hover-img-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    font-size: 10px;
    color: #ccc;
  }
  .hover-img-name {
    font-weight: 700;
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .hover-img-size {
    color: #93c5fd;
    flex-shrink: 0;
  }
  @keyframes popover-fade-in {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }
  .tree-group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-right: 8px;
  }
  .btn-group-add {
    background: transparent;
    border: 1px solid transparent;
    color: var(--vs-text-dim);
    font-size: 11px;
    cursor: pointer;
    border-radius: 3px;
    padding: 0 4px;
    line-height: 16px;
  }
  .btn-group-add:hover {
    background: var(--vs-bg-4);
    border-color: var(--vs-border);
    color: #fff;
  }
  .device-row {
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-right: 8px;
  }
  .device-row:hover .btn-device-edit {
    opacity: 1;
  }
  .btn-device-edit {
    opacity: 0.4;
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 11px;
    padding: 0 4px;
    transition: opacity 0.12s;
  }
  .btn-device-edit:hover {
    opacity: 1;
  }
</style>
