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
  import { SYSTEM_TAG_DEFINITIONS } from "$lib/services/systemTagsService";

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

  // Per-PLC device expand/collapse state (default expanded)
  let expandedDevices = $state<Record<string, boolean>>({});

  type HoverImage = {
    node: ProjectNode;
    x: number;
    y: number;
  };
  let hoverImage = $state<HoverImage | null>(null);

  const tree = $derived(project.tree ?? []);

  const plcTagsCount = $derived((project.tags ?? []).filter((t) => t.binding.table !== "memory" && t.binding.table !== "system").length);
  const memoryTagsCount = $derived((project.tags ?? []).filter((t) => t.binding.table === "memory").length);
  const systemTagsCount = $derived(SYSTEM_TAG_DEFINITIONS.length);
  const totalTagsCount = $derived((project.tags ?? []).length + systemTagsCount);

  function toggleDevice(id: string) {
    expandedDevices[id] = !(expandedDevices[id] ?? true);
  }

  function isDeviceExpanded(id: string): boolean {
    return expandedDevices[id] ?? true;
  }

  function openVariablesManager() {
    const varsNode = tree.find((n) => n.kind === "variables");
    if (varsNode) {
      selectSolutionNode(varsNode.id);
    } else {
      addVariableModalOpen.set(true);
    }
  }

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
      style:padding-left="{depth * 12 + 12}px"
      draggable={design && node.kind !== "folder"}
      ondragstart={(e) => onDragStart(e, node)}
      ondragover={onDragOver}
      ondrop={(e) => onDrop(e, node)}
      onclick={() => (node.kind === "folder" ? toggleFolderCollapsed(node.id) : onSelect(node))}
      onmouseenter={(e) => handleNodeMouseEnter(node, e)}
      onmousemove={(e) => handleNodeMouseMove(node, e)}
      onmouseleave={() => handleNodeMouseLeave(node)}
      oncontextmenu={(e) => openCtx(e, node)}
    >
      {#if node.kind === "folder"}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <span class="twist" onclick={(e) => onFolderClick(node, e)} role="button" tabindex="0">
          {collapsed ? "▶" : "▼"}
        </span>
      {:else}
        <span class="twist spacer"></span>
      {/if}

      <span class="ico">{iconFor(node.kind)}</span>

      {#if renamingId === node.id}
        <input
          id="rename-{node.id}"
          class="rename-input"
          bind:value={renameValue}
          onblur={commitRename}
          onkeydown={(e) => {
            if (e.key === "Enter") commitRename();
            if (e.key === "Escape") renamingId = null;
          }}
        />
      {:else}
        <span class="label">{node.name}{node.kind === "screen" ? ".form" : ""}</span>
      {/if}
    </div>

    {#if node.kind === "folder" && !collapsed}
      {@render treeRows(node.id, depth + 1)}
    {/if}
  {/each}
{/snippet}

<div class="panel solution-explorer">
  <div class="header">
    <span>Solution Explorer</span>
    {#if design}
      <div class="header-actions">
        <button type="button" title="New folder" onclick={() => addProjectFolder()}>📁+</button>
        <button type="button" title="Import image(s)" onclick={() => triggerImportImage()}>🖼️+</button>
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

    <!-- Expandable PLC Devices Hierarchy -->
    <div class="tree-group-header">
      <span class="tree-group">Sterowniki PLC / Devices ({project.devices.length})</span>
      {#if design}
        <button type="button" class="btn-group-add" title="Dodaj Nowy Sterownik PLC..." onclick={() => openAddDeviceModal()}>🔌+</button>
      {/if}
    </div>

    {#each project.devices as d}
      {@const expanded = isDeviceExpanded(d.id)}
      {@const queriesList = d.queries ?? []}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="tree-item device-row"
        style:padding-left="16px"
        role="button"
        tabindex="0"
        title="Sterownik PLC: {d.name} ({d.host}:{d.port})"
        onclick={() => design && openEditDeviceModal(d.id, "params")}
        onkeydown={(e) => (e.key === "Enter" || e.key === " ") && design && openEditDeviceModal(d.id, "params")}
      >
        <button
          type="button"
          class="tree-toggle"
          onclick={(e) => {
            e.stopPropagation();
            toggleDevice(d.id);
          }}
        >
          {expanded ? "▼" : "▶"}
        </button>
        <span class="ico">🔌</span>
        <span class="label">{d.name} ({d.host}:{d.port})</span>
        <span class="unit-badge">U:{d.unit_id}</span>
        {#if design}
          <div class="device-actions">
            <button
              type="button"
              class="btn-device-act"
              title="Dodaj / Edytuj zapytania Modbus..."
              onclick={(e) => {
                e.stopPropagation();
                openEditDeviceModal(d.id, "queries");
              }}
            >
              📡+
            </button>
            <button
              type="button"
              class="btn-device-act"
              title="Edytuj parametry połączenia PLC..."
              onclick={(e) => {
                e.stopPropagation();
                openEditDeviceModal(d.id, "params");
              }}
            >
              ✏️
            </button>
            <button
              type="button"
              class="btn-device-act danger"
              title="Usuń sterownik..."
              onclick={(e) => {
                e.stopPropagation();
                if (confirm(`Czy na pewno usunąć sterownik ${d.name}?`)) {
                  deleteDeviceFromProject(d.id);
                }
              }}
            >
              🗑️
            </button>
          </div>
        {/if}
      </div>

      <!-- Nested Modbus Poll Queries under PLC Device -->
      {#if expanded}
        {#each queriesList as q}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="tree-item query-row"
            style:padding-left="38px"
            role="button"
            tabindex="0"
            title="Zapytanie Modbus: {q.name} ({q.table} R{q.start_address}..{q.start_address + Math.max(1, q.count) - 1})"
            onclick={() => design && openEditDeviceModal(d.id, "queries")}
            onkeydown={(e) => (e.key === "Enter" || e.key === " ") && design && openEditDeviceModal(d.id, "queries")}
          >
            <span class="ico">📡</span>
            <span class="label">{q.name}</span>
            <span class="query-meta">
              ({q.table === 'holding' ? '4x' : q.table === 'input' ? '3x' : q.table === 'coil' ? '0x' : '1x'} R{q.start_address}..{q.start_address + Math.max(1, q.count) - 1})
            </span>
          </div>
        {:else}
          <div class="tree-item empty-query-row" style:padding-left="38px">
            <span class="hint">Brak zapytań (Kliknij 📡+, aby dodać)</span>
          </div>
        {/each}
      {/if}
    {/each}

    <!-- Central Variables Management Group -->
    <div class="tree-group-header">
      <span class="tree-group">Zmienne / Variables ({totalTagsCount})</span>
      <button type="button" class="btn-group-add" title="Otwórz Menedżer Zmiennych / Dodaj..." onclick={() => openVariablesManager()}>🏷️+</button>
    </div>

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="tree-item device-row"
      style:padding-left="24px"
      role="button"
      tabindex="0"
      title="Otwórz Centralną Baza Zmiennych SCADA"
      onclick={() => openVariablesManager()}
      onkeydown={(e) => (e.key === "Enter" || e.key === " ") && openVariablesManager()}
    >
      <span class="ico">🏷️</span>
      <span class="label">Centralna Baza Zmiennych ({totalTagsCount})</span>
    </div>

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="tree-item device-row"
      style:padding-left="24px"
      role="button"
      tabindex="0"
      title="Zmienne Pamięci Wewnętrznej SCADA"
      onclick={() => openVariablesManager()}
      onkeydown={(e) => (e.key === "Enter" || e.key === " ") && openVariablesManager()}
    >
      <span class="ico">🧠</span>
      <span class="label">Zmienne Pamięci ({memoryTagsCount})</span>
    </div>

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="tree-item device-row"
      style:padding-left="24px"
      role="button"
      tabindex="0"
      title="Zmienne Systemowe SCADA (Czas, Uptime, Dysk, CPU, Rola)"
      onclick={() => openVariablesManager()}
      onkeydown={(e) => (e.key === "Enter" || e.key === " ") && openVariablesManager()}
    >
      <span class="ico">⚙️</span>
      <span class="label">Zmienne Systemowe ({systemTagsCount})</span>
    </div>

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
      Schema v{project.schema_version ?? 3} · Hash: {(project.content_hash ?? "").slice(0, 12)}…
    </div>
  </div>
</div>

{#if ctx.open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="ctx-backdrop" onclick={closeCtx}></div>
  <div class="ctx-menu" style:left="{ctx.x}px" style:top="{ctx.y}px">
    {#if design}
      {#if ctx.node}
        <div class="ctx-label">{ctx.node.name}</div>
        <button
          type="button"
          role="menuitem"
          onclick={() => {
            if (ctx.node) startRename(ctx.node);
          }}
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

      <div class="ctx-label">Add New Item</div>
      <button type="button" role="menuitem" onclick={() => runAdd("screen")}>
        🗂 New Screen…
      </button>
      <button type="button" role="menuitem" onclick={() => runAdd("folder")}>
        📁 New Folder
      </button>
      <button type="button" role="menuitem" onclick={() => triggerImportImage(parentForAdd())}>
        🖼️ Import Image(s)…
      </button>
      <button type="button" role="menuitem" onclick={() => runAdd("script")}>
        📜 New Script (.js)…
      </button>
      <button type="button" role="menuitem" onclick={() => runAdd("style")}>
        🎨 New Style (.css)…
      </button>
      <button type="button" role="menuitem" onclick={() => runAdd("note")}>
        📝 New Text Note…
      </button>
      <button type="button" role="menuitem" onclick={() => runAdd("markdown")}>
        MD New Markdown Doc…
      </button>
      <div class="sep"></div>
      <button
        type="button"
        role="menuitem"
        onclick={() => {
          closeCtx();
          openAddDeviceModal();
        }}
      >
        🔌 Add PLC Device…
      </button>

      <div class="sep"></div>
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
    style:left="{Math.min(hoverImage.x + 16, window.innerWidth - 220)}px"
    style:top="{Math.min(hoverImage.y + 16, window.innerHeight - 200)}px"
  >
    <div class="hover-img-wrap">
      <img src={hoverImage.node.content} alt={hoverImage.node.name} />
    </div>
    <div class="hover-img-meta">
      <span class="hover-img-name">{hoverImage.node.name}</span>
    </div>
  </div>
{/if}

<style>
  .solution-explorer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--vs-bg-2);
    font-size: 12px;
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    background: var(--vs-bg-3);
    border-bottom: 1px solid var(--vs-border);
    font-weight: 700;
    text-transform: uppercase;
    font-size: 11px;
    letter-spacing: 0.05em;
  }
  .header-actions {
    display: flex;
    gap: 2px;
  }
  .header-actions button {
    background: transparent;
    border: none;
    color: var(--vs-text-dim);
    cursor: pointer;
    font-size: 11px;
    padding: 1px 4px;
  }
  .header-actions button:hover {
    color: var(--vs-text-bright);
  }
  .se-body {
    flex: 1;
    overflow: auto;
    padding: 4px 0;
  }
  .tree-group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 8px 4px 10px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    color: var(--vs-text-dim);
    letter-spacing: 0.05em;
    margin-top: 6px;
    border-top: 1px solid var(--vs-border-soft);
  }
  .btn-group-add {
    background: transparent;
    border: none;
    color: var(--vs-text-dim);
    font-size: 10px;
    cursor: pointer;
    padding: 0 4px;
  }
  .btn-group-add:hover {
    color: var(--vs-text-bright);
  }
  .tree-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    cursor: default;
    color: var(--vs-text);
  }
  .tree-item:hover {
    background: rgba(255, 255, 255, 0.06);
  }
  .tree-item.active {
    background: var(--vs-selection);
    color: var(--vs-text-bright);
  }
  .device-row {
    font-weight: 600;
  }
  .tree-toggle {
    background: transparent;
    border: none;
    color: var(--vs-text-dim, #9d9d9d);
    font-size: 9px;
    width: 14px;
    height: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
  }
  .tree-toggle:hover {
    color: #ffffff;
  }
  .device-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.12s ease;
  }
  .device-row:hover .device-actions {
    opacity: 1;
  }
  .btn-device-act {
    background: transparent;
    border: none;
    font-size: 10px;
    padding: 1px 3px;
    cursor: pointer;
    border-radius: 2px;
    opacity: 0.8;
  }
  .btn-device-act:hover {
    opacity: 1;
    background: rgba(255, 255, 255, 0.15);
  }
  .btn-device-act.danger:hover {
    background: rgba(220, 38, 38, 0.3);
  }
  .query-row {
    font-size: 11.5px;
    color: var(--vs-text-dim, #9d9d9d);
  }
  .query-row:hover {
    color: var(--vs-text-bright, #ffffff);
  }
  .query-meta {
    font-size: 10px;
    color: #64748b;
    font-family: var(--font-mono, monospace);
  }
  .unit-badge {
    font-size: 9px;
    background: var(--vs-bg-3, #2d2d30);
    border: 1px solid var(--vs-border, #3e3e42);
    color: #9cdcfe;
    padding: 0 4px;
    border-radius: 3px;
  }
  .empty-query-row {
    font-size: 11px;
    color: #64748b;
    font-style: italic;
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
    white-space: nowrap;
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
  }
  .hover-img-name {
    font-size: 11px;
    font-weight: 600;
    color: #f1f5f9;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .hint {
    font-size: 10px;
    color: #64748b;
  }
</style>
