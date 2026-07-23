<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, tag = null, design = false, onWrite }: Props = $props();

  let frozen = $state(false);

  function toggleFreeze() {
    if (design || !onWrite) return;
    frozen = !frozen;
    onWrite("wt.fill_step", frozen ? 0 : 10);
  }
</script>

<div class="process-card">
  <div class="card-h">Operator Process Controls</div>
  <div class="body">
    <div class="row">
      <div>
        <strong>Simulation SIM_EN</strong>
        <p class="hint">Modbus I0 Status</p>
      </div>
      <span class="state" class:on={tag?.bool_value ?? true}>
        {tag?.bool_value ?? true ? "ON" : "OFF"}
      </span>
    </div>
    <div class="row">
      <div>
        <strong>Process Freeze</strong>
        <p class="hint">FILL_STEP → 0 / Restore</p>
      </div>
      <button class="btn-freeze" class:frozen disabled={design} onclick={toggleFreeze}>
        {frozen ? "▶ Resume" : "⏸ Freeze"}
      </button>
    </div>
  </div>
</div>

<style>
  .process-card {
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
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 4px 0;
  }
  strong {
    font-size: 11px;
    color: #1f2937;
  }
  .hint {
    margin: 1px 0 0;
    font-size: 9px;
    color: #9ca3af;
  }
  .state {
    font-size: 10px;
    font-weight: 800;
    padding: 3px 8px;
    border-radius: 99px;
    background: #9ca3af;
    color: #ffffff;
  }
  .state.on {
    background: #16a34a;
  }
  .btn-freeze {
    border: 1px solid #e5e7eb;
    background: #ffffff;
    color: #1f2937;
    border-radius: 6px;
    padding: 4px 10px;
    font-weight: 700;
    font-size: 11px;
    cursor: pointer;
  }
  .btn-freeze.frozen {
    border-color: #fca5a5;
    color: #dc2626;
    background: #fef2f2;
  }
  .btn-freeze:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
