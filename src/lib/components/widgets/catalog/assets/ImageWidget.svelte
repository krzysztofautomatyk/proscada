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

  const src = $derived(str("src", ""));
  const fit = $derived(str("fit", "contain"));
  const alt = $derived(str("alt", "Image"));

  // Default SCADA Centrifugal Pump SVG if src is empty
  const defaultPumpSvg = `data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" fill="none"><circle cx="50" cy="50" r="35" fill="%233B82F6" stroke="%231D4ED8" stroke-width="4"/><circle cx="50" cy="50" r="12" fill="%23FFFFFF" stroke="%231E40AF" stroke-width="3"/><path d="M50 15v20M50 65v20M15 50h20M65 50h20" stroke="%23FFFFFF" stroke-width="3" stroke-linecap="round"/><path d="M75 30h20v40H75z" fill="%232563EB" stroke="%231D4ED8" stroke-width="3"/></svg>`;
</script>

<div
  class="image-shell"
  style:background={str("bgColor", "transparent")}
  style:border="{num("borderWidth", 0)}px {str("borderStyle", "none")} {str("borderColor", "transparent")}"
  style:border-radius="{num("borderRadius", 0)}px"
>
  <img
    src={src || defaultPumpSvg}
    {alt}
    style:object-fit={fit as any}
    class="img-element"
  />
</div>

<style>
  .image-shell {
    width: 100%;
    height: 100%;
    position: relative;
    box-sizing: border-box;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .img-element {
    width: 100%;
    height: 100%;
    display: block;
    pointer-events: none;
  }
</style>
