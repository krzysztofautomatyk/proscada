<script lang="ts">
  import type { WidgetRendererProps } from "../../shared/types";
  import { clamp, configOf, readNumber, readString, tagNumber } from "../../shared/config";
  import QualityBadge from "../../shared/QualityBadge.svelte";

  let { widget, tag = null }: WidgetRendererProps = $props();

  const config = $derived(configOf(widget));
  const variant = $derived(readString(config, "variant", "bar"));
  const title = $derived(readString(config, "title", "METER"));
  const unit = $derived(readString(config, "unit", ""));
  const min = $derived(readNumber(config, "min", 0));
  const max = $derived(Math.max(min + 0.0001, readNumber(config, "max", 100)));
  const value = $derived(tagNumber(tag, readNumber(config, "value", 65)));
  const percent = $derived(clamp(((value - min) / (max - min)) * 100, 0, 100));
  const warningAt = $derived(readNumber(config, "warningAt", 75));
  const alarmAt = $derived(readNumber(config, "alarmAt", 90));
  const color = $derived(
    tag?.quality === "bad"
      ? "#b91c1c"
      : percent >= alarmAt
        ? "#dc2626"
        : percent >= warningAt
          ? "#d97706"
          : readString(config, "fillColor", "#2563eb"),
  );
</script>

<div class="meter-card" class:bad={tag?.quality === "bad"} style:--meter-color={color}>
  <div class="header">
    <span>{title}</span>
    <QualityBadge {tag} />
  </div>
  {#if variant === "gauge"}
    <div class="gauge">
      <div class="arc" style:--percent="{percent * 1.8}deg"></div>
      <div class="gauge-value">
        <strong>{value.toFixed(readNumber(config, "decimals", 0, 0, 6))}</strong>
        <small>{unit}</small>
      </div>
    </div>
  {:else}
    <div class:vertical={variant === "vertical"} class="track">
      <div class="fill" style:--percent="{percent}%"></div>
      <span>{value.toFixed(readNumber(config, "decimals", 0, 0, 6))} {unit}</span>
    </div>
  {/if}
  <div class="range"><span>{min}</span><span>{max}</span></div>
</div>

<style>
  .meter-card {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 7px;
    border: 1px solid #d8dee8;
    border-radius: 7px;
    background: #fff;
    color: #334155;
  }
  .header,
  .range {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .header > span {
    font-size: 9px;
    font-weight: 800;
    letter-spacing: 0.04em;
  }
  .track {
    position: relative;
    min-height: 18px;
    flex: 1;
    overflow: hidden;
    border: 1px solid #cbd5e1;
    border-radius: 4px;
    background: #e2e8f0;
  }
  .fill {
    width: var(--percent);
    height: 100%;
    background: var(--meter-color);
    transition: width 180ms ease-out;
  }
  .track > span {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    color: #0f172a;
    font-size: 10px;
    font-weight: 800;
    text-shadow: 0 1px 0 #fff;
  }
  .vertical .fill {
    position: absolute;
    right: 0;
    bottom: 0;
    left: 0;
    width: 100%;
    height: var(--percent);
  }
  .gauge {
    position: relative;
    min-height: 54px;
    flex: 1;
    display: grid;
    place-items: end center;
    overflow: hidden;
  }
  .arc {
    position: absolute;
    bottom: -38px;
    width: 104px;
    height: 104px;
    border-radius: 50%;
    background: conic-gradient(from 270deg, var(--meter-color) 0deg var(--percent), #e2e8f0 var(--percent) 180deg, transparent 180deg);
  }
  .arc::after {
    position: absolute;
    inset: 13px;
    content: "";
    border-radius: 50%;
    background: #fff;
  }
  .gauge-value {
    position: relative;
    z-index: 1;
    padding-bottom: 4px;
    text-align: center;
  }
  .gauge-value strong {
    display: block;
    color: var(--meter-color);
    font-size: 17px;
  }
  .gauge-value small {
    color: #64748b;
    font-size: 9px;
  }
  .range {
    color: #64748b;
    font-size: 8px;
  }
  .bad {
    background: repeating-linear-gradient(135deg, #fff, #fff 6px, #fee2e2 6px, #fee2e2 12px);
  }
</style>

