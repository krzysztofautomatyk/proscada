<script lang="ts">
  /**
   * Label — text chrome + marquee. Blink/visibility come from DynamicShell parent.
   */
  import type { WidgetDef, TagValue } from "$lib/types";
  import { tagMap } from "$lib/stores/app";
  import { isWidgetScrolling } from "$lib/utils/dynamics";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, design = false }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const num = (k: string, d = 0) => Number(cfg[k] ?? d);

  const text = $derived(str("text", "Label"));
  const fontFamily = $derived(str("fontFamily", "Segoe UI, system-ui, sans-serif"));
  const fontSize = $derived(num("fontSize", 14));
  const fontWeight = $derived(str("fontWeight", "normal"));
  const fontStyle = $derived(str("fontStyle", "normal"));
  const textColor = $derived(str("textColor", "#1f2937"));
  const bgColor = $derived(str("bgColor", "transparent"));
  const borderColor = $derived(str("borderColor", "transparent"));
  const borderWidth = $derived(num("borderWidth", 0));
  const borderRadius = $derived(num("borderRadius", 0));
  const align = $derived(str("align", "left"));
  const vAlign = $derived(str("vAlign", "center"));

  const scrolling = $derived(isWidgetScrolling(cfg, $tagMap, widget.tag_id));
  const scrollSpeedSec = $derived(Math.max(1, num("scrollSpeedSec", 8)));
  const scrollDir = $derived(str("scrollDir", "left"));
</script>

<div
  class="label-widget"
  style:background={bgColor}
  style:color={textColor}
  style:border="{borderWidth}px solid {borderColor}"
  style:border-radius="{borderRadius}px"
  style:font-family={fontFamily}
  style:font-size="{fontSize}px"
  style:font-weight={fontWeight}
  style:font-style={fontStyle}
  style:text-align={align}
  style:align-items={vAlign === "top" ? "flex-start" : vAlign === "bottom" ? "flex-end" : "center"}
  style:justify-content={align === "center" ? "center" : align === "right" ? "flex-end" : "flex-start"}
>
  {#if scrolling}
    <div class="marquee-viewport" style:text-align={align}>
      <div
        class="marquee-track"
        class:dir-right={scrollDir === "right"}
        style:animation-duration="{scrollSpeedSec}s"
      >
        <span class="marquee-text">{text}</span>
        <span class="marquee-text gap" aria-hidden="true">{text}</span>
      </div>
    </div>
  {:else}
    <span class="static-text">{text}</span>
  {/if}
</div>

<style>
  .label-widget {
    width: 100%;
    height: 100%;
    display: flex;
    box-sizing: border-box;
    overflow: hidden;
    position: relative;
    padding: 2px 6px;
    white-space: nowrap;
  }
  .static-text {
    width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .marquee-viewport {
    width: 100%;
    overflow: hidden;
  }
  .marquee-track {
    display: inline-flex;
    white-space: nowrap;
    animation: scada-marquee-left 8s linear infinite;
  }
  .marquee-track.dir-right {
    animation-name: scada-marquee-right;
  }
  .marquee-text.gap {
    padding-left: 3em;
  }
  @keyframes scada-marquee-left {
    0% {
      transform: translateX(0);
    }
    100% {
      transform: translateX(-50%);
    }
  }
  @keyframes scada-marquee-right {
    0% {
      transform: translateX(-50%);
    }
    100% {
      transform: translateX(0);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .marquee-track {
      animation: none !important;
    }
  }
</style>
