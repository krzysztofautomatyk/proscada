<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, tag = null, design = false, onWrite }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const num = (k: string, d = 0) => Number(cfg[k] ?? d);

  const title = $derived(str("title", "SP VALUE"));
  const step = $derived(num("step", 10));
  const min = $derived(num("min", 0));
  const max = $derived(num("max", 1000));
  const unit = $derived(str("unit", "cm"));
  const labelColor = $derived(str("labelColor", "#16A34A"));

  let val = $state(200);

  $effect(() => {
    if (tag && tag.value !== undefined) {
      val = tag.value;
    } else {
      val = num("defaultValue", 200);
    }
  });

  function applyValue(newV: number) {
    val = Math.max(min, Math.min(max, newV));
    if (!design && widget.tag_id && onWrite) {
      onWrite(widget.tag_id, val);
    }
  }
</script>

<div class="numeric-input-card">
  <div class="header">
    <span style:color={labelColor}>{title}</span>
    <span class="live">{val.toFixed(0)} {unit}</span>
  </div>
  <div class="step-controls">
    <button
      type="button"
      class="btn-step"
      disabled={design}
      onclick={() => applyValue(val - step)}>−</button
    >
    <input
      type="number"
      class="num-field"
      bind:value={val}
      disabled={design}
      onchange={(e) => applyValue(Number(e.currentTarget.value))}
    />
    <button
      type="button"
      class="btn-step"
      disabled={design}
      onclick={() => applyValue(val + step)}>+</button
    >
  </div>
</div>

<style>
  .numeric-input-card {
    width: 100%;
    height: 100%;
    background: #ffffff;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 6px 8px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 11px;
    font-weight: 700;
  }
  .live {
    font-size: 10px;
    font-weight: 500;
    color: #6b7280;
  }
  .step-controls {
    display: grid;
    grid-template-columns: 28px 1fr 28px;
    gap: 4px;
    margin-top: 4px;
  }
  .num-field {
    text-align: center;
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    padding: 4px;
    font-size: 11px;
    font-weight: 800;
    color: #1f2937;
  }
  .btn-step {
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    background: #ffffff;
    font-weight: 800;
    color: #1f2937;
    cursor: pointer;
  }
  .btn-step:disabled,
  .num-field:disabled {
    opacity: 0.6;
    cursor: default;
  }
</style>
