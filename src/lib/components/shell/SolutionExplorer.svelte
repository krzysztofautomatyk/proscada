<script lang="ts">
  import type { ProjectNode, ProjectNodeKind, ScadaProject } from "$lib/types";
  import {
    selectedFormId,
    selectedNodeId,
    selectSolutionNode,
    addNewForm,
    addProjectFolder,
    addProjectDocument,
    deleteProjectNode,
    renameProjectNode,
    toggleFolderCollapsed,
    moveProjectNode,
    newBlankProject,
    importProjectFile,
    exportProjectJson,
    persistProject,
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

  function startRename(node: ProjectNode) {
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
    if (!design || !dragId) return;
    e.preventDefault();
    e.dataTransfer!.dropEffect = "move";
  }

  function onDrop(e: DragEvent, target: ProjectNode | null) {
    e.preventDefault();
    e.stopPropagation();
    const id = e.dataTransfer?.getData("text/proscada-node") || dragId;
    dragId = null;
    if (!id || !design) return;
    if (target?.kind === "folder") moveProjectNode(id, target.id);
    else if (target) moveProjectNode(id, target.parent_id);
    else moveProjectNode(id, null);
  }

  function renderBranch(parentId: string | null, depth: number): ProjectNode[] {
    return childrenOf(tree, parentId);
  }
</script>

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
        <button type="button" title="New screen" onclick={() => addNewForm()}>🗂+</button>
        <button type="button" title="New script" onclick={() => addProjectDocument("script")}>📜+</button>
      </div>
    {/if}
  </div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="panel-body se-body"
    role="tree"
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

    <div class="tree-group">Devices ({project.devices.length})</div>
    {#each project.devices as d}
      <div class="tree-item" style:padding-left="24px">
        <span>🔌</span>
        <span>{d.name} · {d.host}:{d.port}</span>
      </div>
    {/each}

    <div class="tree-group">Alarms ({project.alarms.length})</div>
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
      <button type="button" role="menuitem" onclick={() => runAdd("screen")}>New HMI Screen</button>
      <button type="button" role="menuitem" onclick={() => runAdd("script")}>New Script (.js)</button>
      <button type="button" role="menuitem" onclick={() => runAdd("variables")}>New Variables List</button>
      <button type="button" role="menuitem" onclick={() => runAdd("note")}>New Note</button>
      <button type="button" role="menuitem" onclick={() => runAdd("markdown")}>New Markdown</button>
      <div class="sep"></div>
      {#if ctx.node}
        <button type="button" role="menuitem" onclick={() => ctx.node && startRename(ctx.node)}>
          Rename<span class="kbd">F2</span>
        </button>
        <button
          type="button"
          role="menuitem"
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
</style>
