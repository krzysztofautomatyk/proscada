<script lang="ts">
  /**
   * Generic LINE widget — free endpoints inside bounding box.
   * Styles: solid | dashed | dotted | dashdot | longdash
   * Caps: none | arrow | open-arrow | circle | square | diamond | bar
   */
  import type { WidgetDef, TagValue } from "$lib/types";
  import { updateWidget } from "$lib/stores/app";

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

  function clamp(v: number, a: number, b: number) {
    return Math.max(a, Math.min(b, v));
  }

  // Endpoints in % of box → pixels
  const x1p = $derived(clamp(num("x1", 5), 0, 100));
  const y1p = $derived(clamp(num("y1", 50), 0, 100));
  const x2p = $derived(clamp(num("x2", 95), 0, 100));
  const y2p = $derived(clamp(num("y2", 50), 0, 100));

  const W = $derived(Math.max(1, widget.w));
  const H = $derived(Math.max(1, widget.h));
  const x1 = $derived((x1p / 100) * W);
  const y1 = $derived((y1p / 100) * H);
  const x2 = $derived((x2p / 100) * W);
  const y2 = $derived((y2p / 100) * H);

  const stroke = $derived(str("stroke", "#1f2937"));
  const strokeWidth = $derived(Math.max(0.5, num("strokeWidth", 2.5)));
  const lineStyle = $derived(str("lineStyle", "solid"));
  const startCap = $derived(str("startCap", "none"));
  const endCap = $derived(str("endCap", "arrow"));
  const capSize = $derived(Math.max(4, num("capSize", 12)));

  function dashArray(style: string, w: number): string | undefined {
    switch (style) {
      case "dashed":
        return `${w * 4} ${w * 3}`;
      case "dotted":
        return `0.1 ${w * 2.4}`;
      case "dashdot":
        return `${w * 5} ${w * 2} 0.1 ${w * 2}`;
      case "longdash":
        return `${w * 9} ${w * 3}`;
      default:
        return undefined;
    }
  }

  const dash = $derived(dashArray(lineStyle, strokeWidth));
  const angle = $derived((Math.atan2(y2 - y1, x2 - x1) * 180) / Math.PI);

  let dragEnd = $state<"start" | "end" | null>(null);
  let rootEl = $state<HTMLDivElement | null>(null);

  function setEndpoint(which: "start" | "end", clientX: number, clientY: number) {
    if (!rootEl || widget.locked) return;
    const r = rootEl.getBoundingClientRect();
    if (r.width < 1 || r.height < 1) return;
    const px = clamp(((clientX - r.left) / r.width) * 100, 0, 100);
    const py = clamp(((clientY - r.top) / r.height) * 100, 0, 100);
    const patch =
      which === "start"
        ? { x1: Math.round(px * 10) / 10, y1: Math.round(py * 10) / 10 }
        : { x2: Math.round(px * 10) / 10, y2: Math.round(py * 10) / 10 };
    updateWidget({
      id: widget.id,
      config: { ...widget.config, ...patch },
    });
  }

  function onCapDown(e: PointerEvent, which: "start" | "end") {
    if (!design || widget.locked) return;
    e.stopPropagation();
    e.preventDefault();
    dragEnd = which;
    (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
  }

  function onCapMove(e: PointerEvent) {
    if (!dragEnd) return;
    setEndpoint(dragEnd, e.clientX, e.clientY);
  }

  function onCapUp() {
    dragEnd = null;
  }

  function capPath(kind: string, s: number): string | null {
    switch (kind) {
      case "arrow":
        return `M ${-s} ${-s * 0.55} L 0 0 L ${-s} ${s * 0.55} Z`;
      case "open-arrow":
        return `M ${-s} ${-s * 0.55} L 0 0 L ${-s} ${s * 0.55}`;
      case "diamond":
        return `M 0 ${-s * 0.55} L ${s * 0.55} 0 L 0 ${s * 0.55} L ${-s * 0.55} 0 Z`;
      case "bar":
        return `M 0 ${-s * 0.75} L 0 ${s * 0.75}`;
      default:
        return null;
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="line-root"
    class:design
    bind:this={rootEl}
    onpointermove={onCapMove}
    onpointerup={onCapUp}
    onpointercancel={onCapUp}
  >
    <svg class="line-svg" width={W} height={H} viewBox="0 0 {W} {H}" overflow="visible">
      <line
        x1={x1}
        y1={y1}
        x2={x2}
        y2={y2}
        stroke={stroke}
        stroke-width={strokeWidth}
        stroke-linecap={lineStyle === "dotted" ? "round" : "butt"}
        stroke-dasharray={dash}
      />

      {#if startCap !== "none"}
        <g transform="translate({x1},{y1}) rotate({angle + 180})">
          {#if startCap === "circle"}
            <circle r={capSize * 0.4} fill={stroke} />
          {:else if startCap === "square"}
            <rect
              x={-capSize * 0.35}
              y={-capSize * 0.35}
              width={capSize * 0.7}
              height={capSize * 0.7}
              fill={stroke}
            />
          {:else if capPath(startCap, capSize)}
            <path
              d={capPath(startCap, capSize) ?? ""}
              fill={startCap === "open-arrow" || startCap === "bar" ? "none" : stroke}
              stroke={stroke}
              stroke-width={startCap === "open-arrow" || startCap === "bar" ? strokeWidth : 0}
              stroke-linejoin="round"
              stroke-linecap="round"
            />
          {/if}
        </g>
      {/if}

      {#if endCap !== "none"}
        <g transform="translate({x2},{y2}) rotate({angle})">
          {#if endCap === "circle"}
            <circle r={capSize * 0.4} fill={stroke} />
          {:else if endCap === "square"}
            <rect
              x={-capSize * 0.35}
              y={-capSize * 0.35}
              width={capSize * 0.7}
              height={capSize * 0.7}
              fill={stroke}
            />
          {:else if capPath(endCap, capSize)}
            <path
              d={capPath(endCap, capSize) ?? ""}
              fill={endCap === "open-arrow" || endCap === "bar" ? "none" : stroke}
              stroke={stroke}
              stroke-width={endCap === "open-arrow" || endCap === "bar" ? strokeWidth : 0}
              stroke-linejoin="round"
              stroke-linecap="round"
            />
          {/if}
        </g>
      {/if}
    </svg>

    {#if design && !widget.locked}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="ep start"
        style:left="{x1p}%"
        style:top="{y1p}%"
        title="Start — drag"
        onpointerdown={(e) => onCapDown(e, "start")}
      ></div>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="ep end"
        style:left="{x2p}%"
        style:top="{y2p}%"
        title="End — drag"
        onpointerdown={(e) => onCapDown(e, "end")}
      ></div>
    {/if}

  </div>

<style>
  .line-root {
    width: 100%;
    height: 100%;
    position: relative;
    box-sizing: border-box;
    overflow: visible;
    touch-action: none;
  }
  .line-svg {
    display: block;
    overflow: visible;
  }
  .ep {
    position: absolute;
    width: 12px;
    height: 12px;
    margin-left: -6px;
    margin-top: -6px;
    border-radius: 50%;
    background: #fff;
    border: 2px solid #007acc;
    cursor: crosshair;
    z-index: 5;
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.25);
  }
  .ep.end {
    border-color: #16a34a;
  }
</style>
