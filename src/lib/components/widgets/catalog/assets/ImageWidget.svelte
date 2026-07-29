<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";
  import { tagMap } from "$lib/stores/app";
  import { evaluateCondition } from "$lib/utils/dynamics";

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

  const falseSrc = $derived(str("src", ""));
  const trueSrc = $derived(str("trueSrc", ""));
  const OBJECT_FITS = ["contain", "cover", "fill", "none", "scale-down"] as const;
  type ObjectFit = (typeof OBJECT_FITS)[number];
  const fit = $derived<ObjectFit>(
    (OBJECT_FITS as readonly string[]).includes(str("fit", "contain"))
      ? (str("fit", "contain") as ObjectFit)
      : "contain",
  );
  const alt = $derived(str("alt", "Image"));

  const stateMode = $derived(str("stateMode", "none"));
  const stateTagId = $derived(str("stateTagId", ""));
  const stateBit = $derived(num("stateBit", 0));
  const stateVal = $derived(num("stateVal", 1));

  const isStateTrue = $derived.by(() => {
    if (stateMode === "none" || !trueSrc) return false;
    const tagId = stateTagId || widget.tag_id;
    if (!tagId) return false;
    return evaluateCondition(stateMode, stateTagId, stateBit, stateVal, $tagMap, widget.tag_id);
  });

  const activeSrc = $derived(isStateTrue && trueSrc ? trueSrc : falseSrc);

  // Default SCADA Centrifugal Pump SVG if activeSrc is empty
  const defaultPumpSvg = `data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" fill="none"><circle cx="50" cy="50" r="35" fill="%233B82F6" stroke="%231D4ED8" stroke-width="4"/><circle cx="50" cy="50" r="12" fill="%23FFFFFF" stroke="%231E40AF" stroke-width="3"/><path d="M50 15v20M50 65v20M15 50h20M65 50h20" stroke="%23FFFFFF" stroke-width="3" stroke-linecap="round"/><path d="M75 30h20v40H75z" fill="%232563EB" stroke="%231D4ED8" stroke-width="3"/></svg>`;
</script>

<div
  class="image-shell"
  style:background={str("bgColor", "transparent")}
  style:border="{num("borderWidth", 0)}px {str("borderStyle", "none")} {str("borderColor", "transparent")}"
  style:border-radius="{num("borderRadius", 0)}px"
>
  <img
    src={activeSrc || defaultPumpSvg}
    {alt}
    style:object-fit={fit}
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
