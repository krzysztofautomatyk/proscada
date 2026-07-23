<script lang="ts">
  import { onMount } from "svelte";
  import {
    project,
    snapshot,
    mode,
    activeForm,
    selectedWidget,
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
        <div class="tabstrip">
          <div class="tab active">
            {#if isWaterTankBoard}
              Main_Synoptic · SCADA board
              {$mode === "designer" ? " [Design = Run 1:1]" : " [Running]"}
            {:else}
              {$activeForm?.name ?? "Form"}.form
              {$mode === "designer" ? " [Design]" : " [Running]"}
            {/if}
          </div>
        </div>
        {#if isWaterTankBoard}
          <!-- Light board only; VS chrome stays dark. Designer ≡ Runtime. -->
          <WaterTankHmi
            snapshot={$snapshot}
            tagMap={$tagMap}
            {onWrite}
            designMode={$mode === "designer"}
          />
        {:else if $activeForm}
          <DesignerCanvas
            form={$activeForm}
            tagMap={$tagMap}
            design={$mode === "designer"}
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
