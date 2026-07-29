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

  let draft = $state("");
  let edited = $state(false);
  let status = $state("");
  const disabled = $derived(design || !onWrite || !widget.tag_id || tag?.quality !== "good");

  $effect(() => {
    if (!edited) draft = tag?.quality === "good" ? String(tag.value) : "";
  });

  function adjust(delta: number) {
    const current = Number(draft);
    edited = true;
    draft = String(Math.min(500, Math.max(1, (Number.isFinite(current) ? current : 1) + delta)));
    status = "";
  }

  async function writeK(candidate: string) {
    const value = Number(candidate);
    if (disabled || !onWrite || !widget.tag_id || candidate.trim() === "" || !Number.isFinite(value)) return;
    status = "WRITE REQUESTED";
    try {
      const result = await onWrite(widget.tag_id, value);
      status = writeResultLabel(result);
    } catch (error) {
      status = `WRITE REJECTED: ${error instanceof Error ? error.message : String(error)}`;
    }
  }
</script>

<div class="inflow-card">
  <div class="card-h">Inflow K Factor</div>
  <div class="body">
    <div class="step">
      <button type="button" disabled={design} onclick={() => adjust(-10)}>−</button>
      <input aria-label="Inflow K factor" type="number" value={draft} disabled={design} oninput={(event) => { edited = true; draft = event.currentTarget.value; status = ""; }} />
      <button type="button" disabled={design} onclick={() => adjust(10)}>+</button>
    </div>
    <div class="btn-presets">
      <button class="btn-quick" disabled={disabled} onclick={() => { edited = true; draft = "50"; void writeK(draft); }}>0.5</button>
      <button class="btn-quick" disabled={disabled} onclick={() => { edited = true; draft = "100"; void writeK(draft); }}>1.0</button>
      <button class="btn-quick" disabled={disabled} onclick={() => { edited = true; draft = "150"; void writeK(draft); }}>1.5</button>
      <button class="btn-write" disabled={disabled || draft.trim() === "" || !Number.isFinite(Number(draft))} onclick={() => void writeK(draft)}>Write K</button>
    </div>
    {#if disabled && !design}<p class="write-state error">WRITE INHIBITED: bind a GOOD-quality tag</p>
    {:else if status}<p class="write-state" role="status">{status}</p>{/if}
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
  .write-state { margin: 0; color: #0369a1; font-size: 8px; font-weight: 800; text-align: center; }
  .write-state.error { color: #b91c1c; }
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
