<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";
  import { tagMap } from "$lib/stores/app";
  import {
    formatTrustedValue,
    NO_VALUE_PLACEHOLDER,
    resolveTagQuality,
  } from "../../shared/quality";

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

  const quality = $derived(resolveTagQuality(widget, tag, design));
  const levelCm = $derived.by(() => {
    if (tag?.value !== undefined && tag.value !== null) return tag.value;
    if (typeof quality.lastValidValue === "number") return quality.lastValidValue;
    return null;
  });
  const levelLabel = $derived(formatTrustedValue(quality, levelCm, 0, { showLastKnown: true }));
  const spanLabel = $derived(
    levelCm === null ? NO_VALUE_PLACEHOLDER : (levelCm / 10).toFixed(1),
  );

  // Pump states come from configured tags; retain last state when degraded
  const p1TagId = $derived(str("pump1TagId"));
  const p2TagId = $derived(str("pump2TagId"));
  const p1 = $derived(p1TagId ? ($tagMap.get(p1TagId) ?? null) : null);
  const p2 = $derived(p2TagId ? ($tagMap.get(p2TagId) ?? null) : null);
  const p1Known = $derived(design || !!p1);
  const p2Known = $derived(design || !!p2);
  const p1Run = $derived(p1Known && (p1?.bool_value ?? (p1?.value !== undefined ? p1.value > 0 : false)));
  const p2Run = $derived(p2Known && (p2?.bool_value ?? (p2?.value !== undefined ? p2.value > 0 : false)));
  const inflowTagId = $derived(str("inflowTagId"));
  const inflow = $derived(inflowTagId ? ($tagMap.get(inflowTagId) ?? null) : null);
  const inflowLabel = $derived(
    inflow
      ? inflow.value.toFixed(2)
      : NO_VALUE_PLACEHOLDER,
  );
  const borderRadius = $derived(num("borderRadius", 10));
</script>

<div class="metrics-panel-box" style:border-radius="{borderRadius}px">
  <div class="card-header">{str("title", "METRICS OVERVIEW")}</div>
  <div class="metrics-grid">
    <!-- Analog Readout: Level -->
    <div class="metric-card">
      <span class="m-label">TANK LEVEL (ANALOG)</span>
      <div class="m-val blue">{levelLabel} <span class="u">cm</span></div>
      <div class="m-sub">{spanLabel} % Span</div>
    </div>

    <!-- Bool Indicator: Pump 1 -->
    <div class="metric-card">
      <div class="m-head">
        <span class="bool-dot" class:on={p1Run}></span>
        <span class="m-label">PUMP 1 (BOOL)</span>
      </div>
      <div class="m-val" class:green={p1Run}>
        {p1Known ? (p1Run ? "RUNNING" : "STOPPED") : NO_VALUE_PLACEHOLDER}
      </div>
      <div class="m-sub">Lead Pump</div>
    </div>

    <!-- Bool Indicator: Pump 2 -->
    <div class="metric-card">
      <div class="m-head">
        <span class="bool-dot" class:on={p2Run}></span>
        <span class="m-label">PUMP 2 (BOOL)</span>
      </div>
      <div class="m-val" class:green={p2Run}>
        {p2Known ? (p2Run ? "RUNNING" : "STOPPED") : NO_VALUE_PLACEHOLDER}
      </div>
      <div class="m-sub">Lag Pump</div>
    </div>

    <!-- Analog Readout: Inflow Factor -->
    <div class="metric-card">
      <span class="m-label">INFLOW K (ANALOG)</span>
      <div class="m-val">{inflowLabel} <span class="u">×100</span></div>
      <div class="m-sub">Multiplier</div>
    </div>
  </div>
</div>

<style>
  .metrics-panel-box {
    width: 100%;
    height: 100%;
    background: #ffffff;
    border: 1px solid #e5e7eb;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 1px 3px rgba(0,0,0,0.05);
  }
  .card-header {
    background: #f9fafb;
    padding: 6px 10px;
    font-size: 11px;
    font-weight: 800;
    color: #374151;
    border-bottom: 1px solid #e5e7eb;
  }
  .metrics-grid {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    padding: 8px;
  }
  .metric-card {
    background: #f9fafb;
    border: 1px solid #f3f4f6;
    border-radius: 8px;
    padding: 6px 8px;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
  .m-head {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .bool-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #9ca3af;
  }
  .bool-dot.on {
    background: #16a34a;
    box-shadow: 0 0 4px #16a34a;
  }
  .m-label {
    font-size: 9px;
    font-weight: 800;
    color: #6b7280;
  }
  .m-val {
    font-size: 16px;
    font-weight: 800;
    color: #1f2937;
    margin: 2px 0;
  }
  .m-val.blue { color: #0284c7; }
  .m-val.green { color: #16a34a; }
  .m-sub {
    font-size: 9px;
    color: #9ca3af;
  }
  .u {
    font-size: 10px;
    font-weight: 600;
  }
</style>
