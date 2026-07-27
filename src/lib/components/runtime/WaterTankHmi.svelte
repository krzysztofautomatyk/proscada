<script lang="ts">
  /**
   * Water Tank SCADA board (light panel inside dark VS chrome).
   * Designer ≡ Runtime (1:1). Isometric SVG + operator controls.
   * Data path: Modbus tags → projectWaterTank() → IsometricScene.
   */
  import type { EngineSnapshot, TagValue } from "$lib/types";
  import { log, refreshAudit } from "$lib/stores/app";
  import { api } from "$lib/services/api";
  import { projectWaterTank } from "$lib/stores/waterTank.svelte";
  import IsometricScene from "./isometric/IsometricScene.svelte";
  import MetricsPanel from "./isometric/MetricsPanel.svelte";
  import AlarmPanel from "./isometric/AlarmPanel.svelte";
  import "$lib/styles/scada-theme.css";

  interface Props {
    snapshot: EngineSnapshot | null;
    tagMap: Map<string, TagValue>;
    onWrite: (tagId: string, value: number | string) => Promise<void> | void;
    designMode?: boolean;
  }

  let { snapshot, tagMap, onWrite, designMode = false }: Props = $props();

  const vm = $derived(
    projectWaterTank(tagMap, snapshot?.connected ?? false, snapshot?.alarms ?? []),
  );

  const G = "#16A34A";
  const Y = "#EAB308";
  const R = "#DC2626";

  let editStop = $state(200);
  let editP1 = $state(700);
  let editP2 = $state(800);
  let editK = $state(150);
  let lastFill = $state(10);
  let frozen = $state(false);
  let busy = $state(false);
  let synced = $state(false);

  $effect(() => {
    if (!synced && vm.qualityLevel === "good") {
      editStop = Math.round(vm.spStop);
      editP1 = Math.round(vm.spP1);
      editP2 = Math.round(vm.spP2);
      editK = Math.round(vm.kX100);
      if (vm.fillStep > 0) lastFill = Math.round(vm.fillStep);
      frozen = vm.fillStep === 0;
      synced = true;
    }
  });

  $effect(() => {
    if (synced) frozen = vm.fillStep === 0;
  });

  async function write(tagId: string, value: number) {
    await onWrite(tagId, value);
  }

  async function applySetpoints() {
    if (!(editStop >= 0 && editStop < editP1 && editP1 <= editP2 && editP2 <= 1000)) {
      log("Setpoints invalid: 0 ≤ STOP < P1 ≤ P2 ≤ 1000", "err");
      return;
    }
    if (
      !confirm(
        `APPLY SETPOINTS\nSP_STOP=${editStop}\nSP_P1_ON=${editP1}\nSP_P2_ON=${editP2}\n\nAudited · PLC Allow SCADA writes required.`,
      )
    )
      return;
    busy = true;
    try {
      await write("wt.sp_stop", editStop);
      await write("wt.sp_p1_on", editP1);
      await write("wt.sp_p2_on", editP2);
      log(`Setpoints applied STOP=${editStop} P1=${editP1} P2=${editP2}`, "ok");
      await refreshAudit();
    } catch (e) {
      log(`Setpoints failed: ${e}`, "err");
    } finally {
      busy = false;
    }
  }

  async function applyK() {
    if (editK < 1 || editK > 500) {
      log("K×100 must be 1…500", "err");
      return;
    }
    if (!confirm(`Write K_x100 = ${editK}?`)) return;
    busy = true;
    try {
      await write("wt.k_x100", editK);
      log(`K_x100 = ${editK}`, "ok");
    } catch (e) {
      log(`K write failed: ${e}`, "err");
    } finally {
      busy = false;
    }
  }

  async function toggleFreeze() {
    busy = true;
    try {
      if (!frozen) {
        if (vm.fillStep > 0) lastFill = Math.round(vm.fillStep);
        await write("wt.fill_step", 0);
        frozen = true;
        log("FREEZE — FILL_STEP=0", "warn");
      } else {
        const restore = lastFill > 0 ? lastFill : 10;
        await write("wt.fill_step", restore);
        frozen = false;
        log(`RESUME — FILL_STEP=${restore}`, "ok");
      }
    } catch (e) {
      log(`Freeze failed: ${e}`, "err");
    } finally {
      busy = false;
    }
  }

  async function ackOne(id: string) {
    try {
      await api.ackAlarm(id);
      log(`Alarm ACK ${id}`, "ok");
      await refreshAudit();
    } catch (e) {
      log(`Ack failed: ${e}`, "err");
    }
  }

  async function ackAll() {
    for (const a of snapshot?.alarms ?? []) {
      if (a.state !== "inactive") {
        try {
          await api.ackAlarm(a.def_id);
        } catch {
          /* */
        }
      }
    }
    log("Alarms acknowledged", "ok");
    await refreshAudit();
  }
</script>

<div class="scada-board board">
  <header class="board-bar">
    <div>
      <h1>Water Tank · Dual-Pump Station</h1>
      <p>
        Isometric SCADA board · greyscale + G/Y/R
        {#if designMode}
          · <strong>Designer = Runtime (1:1)</strong>
        {:else}
          · Runtime
        {/if}
      </p>
    </div>
    <div class="board-badges" role="status" aria-live="polite">
      <span class="pill" class:g={vm.simEn}>{vm.simEn ? "SIM ON" : "SIM OFF"}</span>
      <span class="pill" class:y={frozen}>{frozen ? "FROZEN" : "LIVE"}</span>
      <span class="pill" class:g={vm.connected} class:r={!vm.connected}>
        {vm.connected ? "MODBUS OK" : "OFFLINE"}
      </span>
    </div>
  </header>

  <MetricsPanel {vm} />

  <div class="main">
    <section class="card synoptic">
      <div class="card-h">Synoptic · tank / water / soil / grass / inlet</div>
      <div class="syn-pad">
        <IsometricScene {vm} animated={true} />
      </div>
    </section>

    <aside class="side">
      <section class="card">
        <div class="card-h">Operator controls</div>
        <div class="body">
          <div class="row">
            <div>
              <strong>Simulation SIM_EN</strong>
              <p class="hint">I0 read-only on map — toggle in PLC Watch</p>
            </div>
            <span class="state" style:background={vm.simEn ? G : "#9CA3AF"}>{vm.simEn ? "ON" : "OFF"}</span>
          </div>
          <div class="row">
            <div>
              <strong>Process freeze</strong>
              <p class="hint">FILL_STEP → 0 / restore</p>
            </div>
            <button class="btn" class:danger={frozen} disabled={busy} onclick={toggleFreeze}>
              {frozen ? "▶ Resume" : "⏸ Freeze"}
            </button>
          </div>
        </div>
      </section>

      <AlarmPanel {vm} onAck={ackOne} onAckAll={ackAll} />

      <section class="card">
        <div class="card-h">Operating levels (cm)</div>
        <div class="body form">
          <label>
            <span style:color={G}>SP_STOP</span>
            <div class="step">
              <button type="button" onclick={() => (editStop = Math.max(0, editStop - 50))}>−</button>
              <input type="number" bind:value={editStop} />
              <button type="button" onclick={() => (editStop = Math.min(1000, editStop + 50))}>+</button>
            </div>
            <em>Live {vm.spStop.toFixed(0)}</em>
          </label>
          <label>
            <span style:color={Y}>SP_P1_ON</span>
            <div class="step">
              <button type="button" onclick={() => (editP1 = Math.max(0, editP1 - 50))}>−</button>
              <input type="number" bind:value={editP1} />
              <button type="button" onclick={() => (editP1 = Math.min(1000, editP1 + 50))}>+</button>
            </div>
            <em>Live {vm.spP1.toFixed(0)}</em>
          </label>
          <label>
            <span style:color={R}>SP_P2_ON</span>
            <div class="step">
              <button type="button" onclick={() => (editP2 = Math.max(0, editP2 - 50))}>−</button>
              <input type="number" bind:value={editP2} />
              <button type="button" onclick={() => (editP2 = Math.min(1000, editP2 + 50))}>+</button>
            </div>
            <em>Live {vm.spP2.toFixed(0)}</em>
          </label>
          <button class="btn primary wide" disabled={busy} onclick={applySetpoints}>Apply setpoints</button>
          <p class="hint">0 ≤ STOP &lt; P1 ≤ P2 ≤ 1000 · HR108–110</p>
        </div>
      </section>

      <section class="card">
        <div class="card-h">Inflow K</div>
        <div class="body form">
          <label>
            <span>K ×100</span>
            <div class="step">
              <button type="button" onclick={() => (editK = Math.max(1, editK - 10))}>−</button>
              <input type="number" bind:value={editK} />
              <button type="button" onclick={() => (editK = Math.min(500, editK + 10))}>+</button>
            </div>
            <em>Live {vm.kX100.toFixed(0)} · FILL {vm.fillStep.toFixed(0)}</em>
          </label>
          <div class="btn-row">
            <button class="btn" disabled={busy} onclick={() => (editK = 50)}>0.5</button>
            <button class="btn" disabled={busy} onclick={() => (editK = 100)}>1.0</button>
            <button class="btn" disabled={busy} onclick={() => (editK = 150)}>1.5</button>
            <button class="btn primary" disabled={busy} onclick={applyK}>Write K</button>
          </div>
        </div>
      </section>
    </aside>
  </div>
</div>

<style>
  .board {
    height: 100%;
    overflow: auto;
    background: var(--scada-bg, #f4f5f7);
    padding: 12px 14px 18px;
  }
  .board-bar {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
    margin-bottom: 12px;
  }
  .board-bar h1 {
    margin: 0;
    font-size: 16px;
    font-weight: 800;
  }
  .board-bar p {
    margin: 2px 0 0;
    font-size: 11px;
    color: var(--scada-muted, #6b7280);
  }
  .board-badges {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .pill {
    font-size: 10px;
    font-weight: 800;
    padding: 4px 10px;
    border-radius: 99px;
    background: #e5e7eb;
    color: #4b5563;
  }
  .pill.g {
    background: #dcfce7;
    color: #16a34a;
  }
  .pill.y {
    background: #fef9c3;
    color: #a16207;
  }
  .pill.r {
    background: #fee2e2;
    color: #dc2626;
  }

  .main {
    display: grid;
    grid-template-columns: 1.3fr 0.7fr;
    gap: 12px;
    align-items: start;
    margin-top: 12px;
  }
  .side {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .card {
    background: var(--scada-panel, #fff);
    border: 1px solid var(--scada-border, #e5e7eb);
    border-radius: 10px;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
    overflow: hidden;
  }
  .card-h {
    padding: 10px 12px;
    border-bottom: 1px solid var(--scada-border, #e5e7eb);
    font-size: 12px;
    font-weight: 800;
  }
  .syn-pad {
    padding: 8px 10px 12px;
  }
  .body {
    padding: 12px;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 8px 0;
    border-bottom: 1px solid #f3f4f6;
  }
  .row:last-child {
    border-bottom: none;
  }
  .row strong {
    font-size: 12px;
  }
  .hint {
    margin: 2px 0 0;
    font-size: 10px;
    color: #9ca3af;
  }
  .state {
    color: #fff;
    font-size: 11px;
    font-weight: 800;
    padding: 6px 12px;
    border-radius: 99px;
    min-width: 48px;
    text-align: center;
  }
  .btn {
    border: 1px solid #e5e7eb;
    background: #fff;
    color: #1f2937;
    border-radius: 8px;
    padding: 7px 12px;
    font-weight: 700;
    font-size: 12px;
    cursor: pointer;
  }
  .btn:hover:not(:disabled) {
    background: #f3f4f6;
  }
  .btn.primary {
    background: #1f2937;
    color: #fff;
    border-color: #1f2937;
  }
  .btn.danger {
    border-color: #fca5a5;
    color: #dc2626;
    background: #fef2f2;
  }
  .btn.wide {
    width: 100%;
  }
  .btn-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .form {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .form label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 11px;
    font-weight: 700;
  }
  .form em {
    font-style: normal;
    font-weight: 500;
    font-size: 10px;
    color: #9ca3af;
  }
  .step {
    display: grid;
    grid-template-columns: 36px 1fr 36px;
    gap: 6px;
  }
  .step input {
    text-align: center;
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 8px;
    color: #1f2937;
    font-weight: 700;
  }
  .step button {
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    background: #fff;
    font-weight: 800;
    color: #1f2937;
    cursor: pointer;
  }
  @media (max-width: 1100px) {
    .main {
      grid-template-columns: 1fr;
    }
  }
</style>
