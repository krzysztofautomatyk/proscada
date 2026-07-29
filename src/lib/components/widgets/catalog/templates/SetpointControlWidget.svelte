<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";
  import type { ProcessWrite } from "$lib/components/widgets/shared/types";
  import { tagMap } from "$lib/stores/app";
  import { writeResultLabel } from "$lib/components/widgets/shared/config";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: ProcessWrite;
  }

  let { widget, design = false, onWrite }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const num = (k: string, d = 0) => Number(cfg[k] ?? d);

  let stopDraft = $state("");
  let p1Draft = $state("");
  let p2Draft = $state("");
  let stopEdited = $state(false);
  let p1Edited = $state(false);
  let p2Edited = $state(false);
  let stopStatus = $state("");
  let p1Status = $state("");
  let p2Status = $state("");
  const borderRadius = $derived(num("borderRadius", 10));
  const stopTagId = $derived(str("stopTagId"));
  const p1TagId = $derived(str("p1TagId"));
  const p2TagId = $derived(str("p2TagId"));
  const targetIds = $derived([stopTagId, p1TagId, p2TagId]);

  $effect(() => {
    const stopTag = stopTagId ? $tagMap.get(stopTagId) : undefined;
    const p1Tag = p1TagId ? $tagMap.get(p1TagId) : undefined;
    const p2Tag = p2TagId ? $tagMap.get(p2TagId) : undefined;
    if (!stopEdited) stopDraft = stopTag?.quality === "good" ? String(stopTag.value) : "";
    if (!p1Edited) p1Draft = p1Tag?.quality === "good" ? String(p1Tag.value) : "";
    if (!p2Edited) p2Draft = p2Tag?.quality === "good" ? String(p2Tag.value) : "";
  });

  function targetGood(tagId: string): boolean {
    return Boolean(tagId && $tagMap.get(tagId)?.quality === "good");
  }

  function validDraft(draft: string): boolean {
    return draft.trim() !== "" && Number.isFinite(Number(draft));
  }

  function adjust(
    draft: string,
    delta: number,
    assign: (value: string) => void,
  ) {
    const current = Number(draft);
    assign(String(Math.min(1000, Math.max(0, (Number.isFinite(current) ? current : 0) + delta))));
  }

  async function writeOne(
    tagId: string,
    draft: string,
    setStatus: (status: string) => void,
  ) {
    const value = Number(draft);
    if (design || !onWrite || !targetGood(tagId) || !validDraft(draft)) return;
    setStatus("WRITE REQUESTED");
    try {
      const result = await onWrite(tagId, value);
      setStatus(writeResultLabel(result));
    } catch (error) {
      setStatus(`WRITE REJECTED: ${error instanceof Error ? error.message : String(error)}`);
    }
  }
</script>

<div class="setpoint-card" style:border-radius="{borderRadius}px">
  <div class="card-h">{str("title", "OPERATING LEVELS SETPOINTS")}</div>
  <div class="body">
    <!-- SP_STOP Stepper -->
    <div class="stepper-item">
      <div class="stepper-head">
        <span style:color="#16A34A">SP_STOP</span>
        <span class="val-badge">{stopDraft || "––"} cm</span>
      </div>
      <div class="step-controls">
        <button type="button" disabled={design} onclick={() => { stopEdited = true; adjust(stopDraft, -50, (value) => (stopDraft = value)); }}>−</button>
        <input aria-label="SP_STOP setpoint" type="number" value={stopDraft} disabled={design} oninput={(event) => { stopEdited = true; stopDraft = event.currentTarget.value; stopStatus = ""; }} />
        <button type="button" disabled={design} onclick={() => { stopEdited = true; adjust(stopDraft, 50, (value) => (stopDraft = value)); }}>+</button>
        <button type="button" class="write-one" disabled={design || !onWrite || !targetGood(stopTagId) || !validDraft(stopDraft)} onclick={() => void writeOne(stopTagId, stopDraft, (value) => (stopStatus = value))}>WRITE</button>
      </div>
      {#if stopStatus}<p class="write-state" role="status">{stopStatus}</p>{/if}
    </div>

    <!-- SP_P1_ON Stepper -->
    <div class="stepper-item">
      <div class="stepper-head">
        <span style:color="#EAB308">SP_P1_ON</span>
        <span class="val-badge">{p1Draft || "––"} cm</span>
      </div>
      <div class="step-controls">
        <button type="button" disabled={design} onclick={() => { p1Edited = true; adjust(p1Draft, -50, (value) => (p1Draft = value)); }}>−</button>
        <input aria-label="SP_P1_ON setpoint" type="number" value={p1Draft} disabled={design} oninput={(event) => { p1Edited = true; p1Draft = event.currentTarget.value; p1Status = ""; }} />
        <button type="button" disabled={design} onclick={() => { p1Edited = true; adjust(p1Draft, 50, (value) => (p1Draft = value)); }}>+</button>
        <button type="button" class="write-one" disabled={design || !onWrite || !targetGood(p1TagId) || !validDraft(p1Draft)} onclick={() => void writeOne(p1TagId, p1Draft, (value) => (p1Status = value))}>WRITE</button>
      </div>
      {#if p1Status}<p class="write-state" role="status">{p1Status}</p>{/if}
    </div>

    <!-- SP_P2_ON Stepper -->
    <div class="stepper-item">
      <div class="stepper-head">
        <span style:color="#DC2626">SP_P2_ON</span>
        <span class="val-badge">{p2Draft || "––"} cm</span>
      </div>
      <div class="step-controls">
        <button type="button" disabled={design} onclick={() => { p2Edited = true; adjust(p2Draft, -50, (value) => (p2Draft = value)); }}>−</button>
        <input aria-label="SP_P2_ON setpoint" type="number" value={p2Draft} disabled={design} oninput={(event) => { p2Edited = true; p2Draft = event.currentTarget.value; p2Status = ""; }} />
        <button type="button" disabled={design} onclick={() => { p2Edited = true; adjust(p2Draft, 50, (value) => (p2Draft = value)); }}>+</button>
        <button type="button" class="write-one" disabled={design || !onWrite || !targetGood(p2TagId) || !validDraft(p2Draft)} onclick={() => void writeOne(p2TagId, p2Draft, (value) => (p2Status = value))}>WRITE</button>
      </div>
      {#if p2Status}<p class="write-state" role="status">{p2Status}</p>{/if}
    </div>

    {#if !targetIds.every(Boolean)}
      <p class="write-state error">CONFIG: stopTagId, p1TagId and p2TagId are required</p>
    {/if}
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
    grid-template-columns: 28px minmax(0, 1fr) 28px 48px;
    gap: 4px;
  }
  .write-state { margin: 0; color: #0369a1; font-size: 8px; font-weight: 800; text-align: center; }
  .write-state.error { color: #b91c1c; }
  .write-one { font-size: 8px; }
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
</style>
