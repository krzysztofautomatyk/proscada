<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, design = false, onWrite }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const num = (k: string, d = 0) => Number(cfg[k] ?? d);

  let stop = $state(200);
  let p1 = $state(700);
  let p2 = $state(800);
  const borderRadius = $derived(num("borderRadius", 10));

  function apply() {
    if (design || !onWrite) return;
    onWrite("wt.sp_stop", stop);
    onWrite("wt.sp_p1_on", p1);
    onWrite("wt.sp_p2_on", p2);
  }
</script>

<div class="setpoint-card" style:border-radius="{borderRadius}px">
  <div class="card-h">{str("title", "OPERATING LEVELS SETPOINTS")}</div>
  <div class="body">
    <!-- SP_STOP Stepper -->
    <div class="stepper-item">
      <div class="stepper-head">
        <span style:color="#16A34A">SP_STOP</span>
        <span class="val-badge">{stop} cm</span>
      </div>
      <div class="step-controls">
        <button type="button" disabled={design} onclick={() => (stop = Math.max(0, stop - 50))}>−</button>
        <input type="number" bind:value={stop} disabled={design} />
        <button type="button" disabled={design} onclick={() => (stop = Math.min(1000, stop + 50))}>+</button>
      </div>
    </div>

    <!-- SP_P1_ON Stepper -->
    <div class="stepper-item">
      <div class="stepper-head">
        <span style:color="#EAB308">SP_P1_ON</span>
        <span class="val-badge">{p1} cm</span>
      </div>
      <div class="step-controls">
        <button type="button" disabled={design} onclick={() => (p1 = Math.max(0, p1 - 50))}>−</button>
        <input type="number" bind:value={p1} disabled={design} />
        <button type="button" disabled={design} onclick={() => (p1 = Math.min(1000, p1 + 50))}>+</button>
      </div>
    </div>

    <!-- SP_P2_ON Stepper -->
    <div class="stepper-item">
      <div class="stepper-head">
        <span style:color="#DC2626">SP_P2_ON</span>
        <span class="val-badge">{p2} cm</span>
      </div>
      <div class="step-controls">
        <button type="button" disabled={design} onclick={() => (p2 = Math.max(0, p2 - 50))}>−</button>
        <input type="number" bind:value={p2} disabled={design} />
        <button type="button" disabled={design} onclick={() => (p2 = Math.min(1000, p2 + 50))}>+</button>
      </div>
    </div>

    <button class="btn-apply" disabled={design} onclick={apply}>Apply setpoints</button>
  </div>
</div>

<style>
  .setpoint-card {
    width: 100%;
    height: 100%;
    background: #ffffff;
    border: 1px solid #e5e7eb;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 1px 3px rgba(0,0,0,0.05);
  }
  .card-h {
    padding: 6px 10px;
    background: #f9fafb;
    border-bottom: 1px solid #e5e7eb;
    font-size: 11px;
    font-weight: 800;
    color: #374151;
  }
  .body {
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .stepper-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .stepper-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 10px;
    font-weight: 800;
  }
  .val-badge {
    font-size: 9px;
    color: #6b7280;
    font-weight: 600;
  }
  .step-controls {
    display: grid;
    grid-template-columns: 28px 1fr 28px;
    gap: 4px;
  }
  .step-controls input {
    text-align: center;
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    padding: 4px;
    font-size: 11px;
    font-weight: 800;
    color: #1f2937;
  }
  .step-controls button {
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    background: #ffffff;
    font-weight: 800;
    color: #1f2937;
    cursor: pointer;
  }
  .step-controls button:disabled,
  .step-controls input:disabled {
    opacity: 0.6;
    cursor: default;
  }
  .btn-apply {
    background: #1f2937;
    color: #ffffff;
    border: none;
    border-radius: 6px;
    padding: 6px;
    font-size: 11px;
    font-weight: 700;
    cursor: pointer;
    margin-top: 4px;
  }
  .btn-apply:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
