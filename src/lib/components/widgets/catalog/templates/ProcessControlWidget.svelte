<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";
  import type { ProcessWrite } from "$lib/components/widgets/shared/types";
  import { writeResultLabel } from "$lib/components/widgets/shared/config";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: ProcessWrite;
  }

  let { widget, tag = null, design = false, onWrite }: Props = $props();

  let frozen = $state(false);
  let status = $state("");
  const restoreValue = $derived(Number(widget.config?.restoreValue));
  const disabled = $derived(
    design || !onWrite || !widget.tag_id || tag?.quality !== "good" || !Number.isFinite(restoreValue),
  );

  async function toggleFreeze() {
    if (disabled || !onWrite || !widget.tag_id) return;
    const next = !frozen;
    status = "COMMAND REQUESTED";
    try {
      const result = await onWrite(widget.tag_id, next ? 0 : restoreValue);
      frozen = next;
      status = writeResultLabel(result, "COMMAND");
    } catch (error) {
      status = `COMMAND REJECTED: ${error instanceof Error ? error.message : String(error)}`;
    }
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
      <span class="state" class:on={tag?.quality === "good" && tag.bool_value === true}>
        {tag?.quality === "good" ? (tag.bool_value ? "ON" : "OFF") : "NO DATA"}
      </span>
    </div>
    <div class="row">
      <div>
        <strong>Process Freeze</strong>
        <p class="hint">FILL_STEP → 0 / Restore</p>
      </div>
      <button class="btn-freeze" class:frozen disabled={disabled} onclick={() => void toggleFreeze()}>
        {frozen ? "▶ Resume" : "⏸ Freeze"}
      </button>
    </div>
    {#if status}<p class="write-state" role="status">{status}</p>{/if}
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
  .write-state { margin: 0; color: #0369a1; font-size: 8px; font-weight: 800; text-align: center; }
  .btn-freeze:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
