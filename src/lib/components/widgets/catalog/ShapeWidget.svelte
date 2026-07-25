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
  const titleColor = $derived(str("titleColor", "#6b7280"));
  const fontFamily = $derived(str("fontFamily", "Segoe UI, system-ui, sans-serif"));
  const fontSize = $derived(num("fontSize", 11));
  const fontWeight = $derived(str("fontWeight", "800"));
  const align = $derived(str("align", "left"));
</script>

<div
  class="shape-box"
  style:border-radius="{borderRadius}px"
  style:background={bgColor}
  style:border="{borderWidth}px {borderStyle} {borderColor}"
  style:box-shadow={shadow}
  style:font-family={fontFamily}
>
  {#if str("title")}
    <div
      class="shape-title"
      style:color={titleColor}
      style:font-size="{fontSize}px"
      style:font-weight={fontWeight}
      style:text-align={align}
    >
      {str("title")}
    </div>
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
    width: 100%;
  }
</style>
