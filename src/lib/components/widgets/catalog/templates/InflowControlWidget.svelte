<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, design = false, onWrite }: Props = $props();

  let kVal = $state(150);

  function writeK(val: number) {
    kVal = val;
    if (design || !onWrite) return;
    onWrite("wt.k_x100", val);
  }
</script>

<div class="inflow-card">
  <div class="card-h">Inflow K Factor</div>
  <div class="body">
    <div class="step">
      <button type="button" disabled={design} onclick={() => (kVal = Math.max(1, kVal - 10))}>−</button>
      <input type="number" bind:value={kVal} disabled={design} />
      <button type="button" disabled={design} onclick={() => (kVal = Math.min(500, kVal + 10))}>+</button>
    </div>
    <div class="btn-presets">
      <button class="btn-quick" disabled={design} onclick={() => writeK(50)}>0.5</button>
      <button class="btn-quick" disabled={design} onclick={() => writeK(100)}>1.0</button>
      <button class="btn-quick" disabled={design} onclick={() => writeK(150)}>1.5</button>
      <button class="btn-write" disabled={design} onclick={() => writeK(kVal)}>Write K</button>
    </div>
  </div>
</div>

<style>
  .inflow-card {
    width: 100%;
    height: 100%;
    background: #ffffff;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .card-h {
    padding: 8px 10px;
    background: #f9fafb;
    border-bottom: 1px solid #e5e7eb;
    font-size: 11px;
    font-weight: 800;
  }
  .body {
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .step {
    display: grid;
    grid-template-columns: 28px 1fr 28px;
    gap: 4px;
  }
  .step input {
    text-align: center;
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    padding: 4px;
    font-size: 11px;
    font-weight: 700;
  }
  .step button {
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    background: #ffffff;
    font-weight: 800;
    cursor: pointer;
  }
  .step button:disabled { opacity: 0.5; cursor: default; }

  .btn-presets {
    display: flex;
    gap: 4px;
  }
  .btn-quick {
    flex: 1;
    border: 1px solid #e5e7eb;
    background: #f9fafb;
    border-radius: 4px;
    padding: 4px;
    font-size: 10px;
    font-weight: 700;
    cursor: pointer;
  }
  .btn-quick:disabled { opacity: 0.5; cursor: default; }

  .btn-write {
    flex: 1.5;
    background: #1f2937;
    color: #ffffff;
    border: none;
    border-radius: 4px;
    padding: 4px;
    font-size: 10px;
    font-weight: 700;
    cursor: pointer;
  }
  .btn-write:disabled { opacity: 0.5; cursor: default; }
</style>
