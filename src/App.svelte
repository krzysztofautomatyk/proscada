<script lang="ts">
  import { onMount } from "svelte";
  import {
    project,
    snapshot,
    mode,
    activeForm,
    selectedWidget,
    selectedFormId,
    selectedWidgetId,
    tagMap,
    audit,
    dirty,
    initApp,
    connectDevice,
    disconnectDevice,
    switchMode,
    persistProject,
    exportProjectJson,
    deleteSelectedWidget,
    addNewForm,
    deleteForm,
    refreshAudit,
    log,
  } from "$lib/stores/app";
  import { api } from "$lib/services/api";
  import type { Role } from "$lib/types";
  import SolutionExplorer from "$lib/components/shell/SolutionExplorer.svelte";
  import Toolbox from "$lib/components/designer/Toolbox.svelte";
  import Properties from "$lib/components/designer/Properties.svelte";
  import DesignerCanvas from "$lib/components/designer/DesignerCanvas.svelte";
  import OutputPanel from "$lib/components/shell/OutputPanel.svelte";
  import WaterTankHmi from "$lib/components/runtime/WaterTankHmi.svelte";

  let role = $state<Role>("engineer");
  let leftTab = $state<"solution" | "toolbox">("solution");

  onMount(() => {
    initApp();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Delete" && $mode === "designer") {
        deleteSelectedWidget();
        log("Widget deleted", "warn");
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "s") {
        e.preventDefault();
        persistProject();
      }
    };
    window.addEventListener("keydown", onKey);
    const auditTimer = setInterval(() => refreshAudit(), 3000);
    return () => {
      window.removeEventListener("keydown", onKey);
      clearInterval(auditTimer);
    };
  });

  async function onRoleChange(r: Role) {
    role = r;
    await api.setRole(r, r === "administrator" ? "admin" : r);
    log(`Role set to ${r}`, "info");
  }

  async function onWrite(tagId: string, value: number) {
    try {
      await api.writeTag(tagId, value);
      log(`Write ${tagId} = ${value}`, "ok");
      await refreshAudit();
    } catch (e) {
      log(`Write failed: ${e}`, "err");
      throw e;
    }
  }

  /** Same SCADA board in Designer and Runtime (1:1). */
  const isWaterTankBoard = $derived(
    $project?.id === "water_tank_dual_pump" ||
      ($project?.name ?? "").toLowerCase().includes("water tank"),
  );

  async function reloadWaterTank() {
    try {
      const p = await api.loadBuiltinWaterTank();
      project.set(p);
      dirty.set(false);
      log("Reloaded factory Water Tank project", "ok");
    } catch (e) {
      log(`Reload failed: ${e}`, "err");
    }
  }

  const statusClass = $derived(
    !$snapshot
      ? "offline"
      : $snapshot.connected
        ? ""
        : $snapshot.last_error
          ? "offline"
          : "degraded",
  );
</script>

<div class="shell">
  <div class="titlebar">
    <span>ProScada</span>
    <span class="badge">v1.0</span>
    {#if $mode === "runtime"}
      <span class="badge warn">RUNTIME</span>
    {:else}
      <span class="badge">DESIGNER</span>
    {/if}
    {#if $dirty}
      <span class="badge danger">UNSAVED</span>
    {/if}
    <span class="spacer"></span>
    <span style:font-weight="400" style:opacity="0.85">
      {$project?.name ?? "No project"} — Visual Studio style SCADA Workstation
    </span>
  </div>

  <div class="menubar">
    <button onclick={() => reloadWaterTank()}>File · New Water Tank</button>
    <button onclick={() => persistProject()}>File · Save</button>
    <button onclick={() => exportProjectJson()}>File · Export JSON</button>
    <button onclick={() => addNewForm()}>Screen · New Screen</button>
    {#if $activeForm && ($project?.forms.length ?? 0) > 1}
      <button onclick={() => deleteForm($activeForm.id)}>Screen · Delete Screen</button>
    {/if}
    <button
      onclick={() => switchMode($mode === "designer" ? "runtime" : "designer")}
    >
      {$mode === "designer" ? "Debug · Start Runtime" : "Debug · Stop (Designer)"}
    </button>
    <button onclick={() => connectDevice()}>Device · Connect / Poll</button>
    <button onclick={() => disconnectDevice()}>Device · Disconnect</button>
    <button onclick={() => refreshAudit()}>Tools · Refresh Audit</button>
  </div>

  <div class="toolbar">
    <button class="primary" onclick={() => switchMode("designer")} disabled={$mode === "designer"}>
      Design
    </button>
    <button class="primary" onclick={() => switchMode("runtime")} disabled={$mode === "runtime"}>
      ▶ Run
    </button>
    <div class="sep"></div>
    <button onclick={() => connectDevice()}>Connect Modbus</button>
    <button onclick={() => disconnectDevice()}>Stop Poll</button>
    <div class="sep"></div>
    <label for="role-select">Role</label>
    <select
      id="role-select"
      value={role}
      onchange={(e) => onRoleChange(e.currentTarget.value as Role)}
    >
      <option value="viewer">Viewer</option>
      <option value="operator">Operator</option>
      <option value="engineer">Engineer</option>
      <option value="administrator">Administrator</option>
    </select>
    <div class="sep"></div>
    {#if $mode === "designer"}
      <button
        class:primary={leftTab === "solution"}
        onclick={() => (leftTab = "solution")}>Solution</button
      >
      <button
        class:primary={leftTab === "toolbox"}
        onclick={() => (leftTab = "toolbox")}>Toolbox</button
      >
      <button onclick={() => addNewForm()}>+ Screen</button>
      <button onclick={() => persistProject()}>Save Project</button>
    {/if}
    <span style:margin-left="auto" style:color="var(--vs-text-dim)">
      {$snapshot?.connected ? "ONLINE" : "OFFLINE"}
      · polls {$snapshot?.poll_count ?? 0}
      · {$snapshot?.last_poll_ms ?? 0} ms
      {#if $snapshot?.last_error}
        · {$snapshot.last_error}
      {/if}
    </span>
  </div>

  <div class="workspace" class:runtime-only={$mode === "runtime"}>
    {#if $project}
      {#if $mode === "designer"}
        <div class="solution" style:display="flex" style:flex-direction="column">
          {#if leftTab === "solution"}
            <SolutionExplorer project={$project} design={true} />
          {:else}
            <Toolbox />
          {/if}
        </div>
      {:else}
        <div class="solution">
          <SolutionExplorer project={$project} design={false} />
        </div>
      {/if}

      <div class="center">
        <!-- Interactive Multi-Screen Tabstrip -->
        <div class="tabstrip" style:display="flex" style:align-items="center">
          {#each $project.forms as f}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="tab"
              class:active={$selectedFormId === f.id}
              role="button"
              tabindex="0"
              onclick={() => {
                selectedFormId.set(f.id);
                selectedWidgetId.set(null);
              }}
              onkeydown={(e) => e.key === "Enter" && selectedFormId.set(f.id)}
            >
              <span>{f.name}.form</span>
              {#if $mode === "designer" && $project.forms.length > 1}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span
                  class="tab-close"
                  title="Close & Delete screen {f.name}"
                  onclick={(e) => {
                    e.stopPropagation();
                    deleteForm(f.id);
                  }}
                >
                  ✕
                </span>
              {/if}
            </div>
          {/each}
          {#if $mode === "designer"}
            <button
              class="btn-new-tab"
              title="Add New Screen"
              onclick={() => addNewForm()}
            >
              + New Screen
            </button>
          {/if}
        </div>
        {#if $mode === "designer" && $activeForm}
          <DesignerCanvas
            form={$activeForm}
            tagMap={$tagMap}
            design={true}
            {onWrite}
          />
        {:else if isWaterTankBoard}
          <!-- Dedicated Runtime HMI dashboard -->
          <WaterTankHmi
            snapshot={$snapshot}
            tagMap={$tagMap}
            {onWrite}
            designMode={false}
          />
        {:else if $activeForm}
          <DesignerCanvas
            form={$activeForm}
            tagMap={$tagMap}
            design={false}
            {onWrite}
          />
        {:else}
          <div
            class="canvas-wrap"
            style:display="flex"
            style:align-items="center"
            style:justify-content="center"
          >
            No form in project
          </div>
        {/if}
      </div>

      {#if $mode === "designer"}
        <div class="properties">
          <Properties
            widget={$selectedWidget}
            form={$activeForm}
            tags={$project.tags}
          />
        </div>
      {/if}

      <div class="output">
        <OutputPanel snapshot={$snapshot} audit={$audit} />
      </div>
    {:else}
      <div style:grid-column="1 / -1" style:padding="40px">Loading project…</div>
    {/if}
  </div>

  <div class="statusbar {statusClass}">
    <span class="item">ProScada Engineering Workstation</span>
    <span class="item">
      {$snapshot?.connected ? "● Modbus master connected" : "○ Disconnected"}
    </span>
    <span class="item">Role: {$snapshot?.role ?? role}</span>
    <span class="item">Mode: {$mode}</span>
    <span class="item" style:margin-left="auto">
      IEC 62443 / ISA-18.2 practices · Lab use only · Not certified
    </span>
  </div>
</div>

<style>
  .tab-close {
    margin-left: 6px;
    font-size: 11px;
    font-weight: 800;
    color: #ef4444;
    cursor: pointer;
    border-radius: 50%;
    padding: 0 4px;
    opacity: 0.7;
  }
  .tab-close:hover {
    opacity: 1;
    background: rgba(239, 68, 68, 0.2);
  }
  .btn-new-tab {
    background: var(--vs-panel-header-bg, #2d2d2d);
    color: var(--vs-text, #cccccc);
    border: 1px dashed var(--vs-border, #444444);
    border-radius: 4px;
    font-size: 11px;
    font-weight: 700;
    padding: 3px 8px;
    margin-left: 8px;
    cursor: pointer;
  }
  .btn-new-tab:hover {
    background: var(--vs-hover, #3e3e42);
    color: #ffffff;
  }
</style>
