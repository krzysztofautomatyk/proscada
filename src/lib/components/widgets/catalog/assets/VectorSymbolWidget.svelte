<script lang="ts">
  import type { WidgetRendererProps } from "../../shared/types";
  import { configOf, readString, tagNumber } from "../../shared/config";
  import QualityBadge from "../../shared/QualityBadge.svelte";

  let { widget, tag = null }: WidgetRendererProps = $props();

  const config = $derived(configOf(widget));
  const symbol = $derived(readString(config, "symbol", "pump"));
  const label = $derived(readString(config, "label", "VECTOR SYMBOL"));
  const active = $derived(tag ? tag.bool_value || tagNumber(tag) !== 0 : false);
  const color = $derived(
    tag?.quality === "bad"
      ? "#b91c1c"
      : active
        ? readString(config, "activeColor", "#2563eb")
        : readString(config, "idleColor", "#64748b"),
  );
</script>

<div class="symbol-card" class:bad={tag?.quality === "bad"}>
  <div class="symbol" style:--symbol-color={color}>
    {#if symbol === "valve"}
      <svg viewBox="0 0 100 70" role="img" aria-label="{label}: valve">
        <path d="M8 15 L50 35 L8 55 Z M92 15 L50 35 L92 55 Z" />
        <path d="M50 35 V8 H72" class="line" />
      </svg>
    {:else if symbol === "motor"}
      <svg viewBox="0 0 100 70" role="img" aria-label="{label}: motor">
        <circle cx="46" cy="35" r="26" />
        <path d="M72 27 H94 V43 H72 M24 17 H8 V53 H24" class="line" />
        <text x="46" y="43">M</text>
      </svg>
    {:else if symbol === "tank"}
      <svg viewBox="0 0 100 80" role="img" aria-label="{label}: tank">
        <path d="M20 12 H80 V64 Q50 78 20 64 Z" />
        <path d="M28 48 H72 V62 Q50 70 28 62 Z" class="fill" />
      </svg>
    {:else if symbol === "sensor"}
      <svg viewBox="0 0 100 70" role="img" aria-label="{label}: sensor">
        <circle cx="50" cy="35" r="12" />
        <path d="M50 5 V18 M50 52 V65 M20 35 H36 M64 35 H80 M29 14 L39 24 M61 46 L71 56 M71 14 L61 24 M39 46 L29 56" class="line" />
      </svg>
    {:else}
      <svg viewBox="0 0 110 80" role="img" aria-label="{label}: pump">
        <circle cx="45" cy="40" r="27" />
        <circle cx="45" cy="40" r="10" class="hub" />
        <path d="M72 26 H100 V54 H72 M18 40 H4" class="line" />
      </svg>
    {/if}
  </div>
  <div class="footer">
    <strong>{label}</strong>
    <QualityBadge {tag} />
  </div>
</div>

<style>
  .symbol-card {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    padding: 7px;
    border: 1px solid #d8dee8;
    border-radius: 7px;
    background: #fff;
  }
  .symbol {
    min-height: 0;
    flex: 1;
  }
  svg {
    width: 100%;
    height: 100%;
    overflow: visible;
  }
  svg :global(path),
  svg :global(circle) {
    fill: color-mix(in srgb, var(--symbol-color) 18%, white);
    stroke: var(--symbol-color);
    stroke-width: 4;
    vector-effect: non-scaling-stroke;
  }
  svg :global(.line) {
    fill: none;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  svg :global(.fill) {
    fill: #38bdf8;
    opacity: 0.7;
  }
  svg :global(.hub) {
    fill: #fff;
  }
  text {
    fill: var(--symbol-color);
    font: 800 24px "Segoe UI", sans-serif;
    text-anchor: middle;
  }
  .footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    padding-top: 4px;
  }
  strong {
    overflow: hidden;
    color: #334155;
    font-size: 9px;
    letter-spacing: 0.04em;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .bad {
    background: repeating-linear-gradient(135deg, #fff, #fff 8px, #fef2f2 8px, #fef2f2 16px);
  }
</style>

