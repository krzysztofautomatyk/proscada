<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, tag = null, design = false }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const num = (k: string, d = 0) => Number(cfg[k] ?? d);

  const title = $derived(str("title", widget.tag_id ?? "Value"));
  const decimals = $derived(num("decimals", 0));
  const unit = $derived(str("unit", ""));
  const fontSize = $derived(num("fontSize", 18));
  const fontWeight = $derived(str("fontWeight", "bold"));
  const textColor = $derived(str("textColor", "#1F2937"));
  const bgColor = $derived(str("bgColor", "#FFFFFF"));
  const borderRadius = $derived(num("borderRadius", 8));
  const borderColor = $derived(str("borderColor", "#E5E7EB"));

  const quality = $derived(tag?.quality ?? "bad");
  const value = $derived(tag?.value ?? 0);
</script>

<div
  class="numeric-card"
  style:background={bgColor}
  style:border="1px solid {borderColor}"
  style:border-radius="{borderRadius}px"
>
  {#if title}
    <div class="header">
      <span class="quality {quality}"></span>
      <span class="title-text">{title}</span>
    </div>
  {/if}
  <div
    class="val-body"
    style:color={textColor}
    style:font-size="{fontSize}px"
    style:font-weight={fontWeight}
  >
    {#if design && !tag}
      — —
    {:else}
      {value.toFixed(decimals)}
      {#if unit}
        <span class="unit-text">{unit}</span>
      {/if}
    {/if}
  </div>
</div>

<style>
  .numeric-card {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    overflow: hidden;
    padding: 6px 8px;
    justify-content: space-between;
  }
  .header {
    font-size: 11px;
    font-weight: 700;
    color: #6b7280;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .val-body {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .unit-text {
    font-size: 0.75em;
    opacity: 0.75;
  }
  .quality {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #dc2626;
  }
  .quality.good { background: #16a34a; }
  .quality.uncertain { background: #eab308; }
</style>
