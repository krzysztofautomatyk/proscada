<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const num = (k: string, d = 0) => Number(cfg[k] ?? d);

  const borderRadius = $derived(num("borderRadius", 8));
  const bgColor = $derived(str("bgColor", "#ffffff"));
  const borderColor = $derived(str("borderColor", "#e5e7eb"));
  const borderWidth = $derived(num("borderWidth", 1));
  const borderStyle = $derived(str("borderStyle", "solid"));
  const shadow = $derived(str("shadow", "0 1px 3px rgba(0,0,0,0.05)"));
</script>

<div
  class="shape-box"
  style:border-radius="{borderRadius}px"
  style:background={bgColor}
  style:border="{borderWidth}px {borderStyle} {borderColor}"
  style:box-shadow={shadow}
>
  {#if str("title")}
    <div class="shape-title">{str("title")}</div>
  {/if}
</div>

<style>
  .shape-box {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    position: relative;
    overflow: hidden;
    padding: 6px 8px;
  }
  .shape-title {
    font-size: 11px;
    font-weight: 800;
    color: #6b7280;
  }
</style>
