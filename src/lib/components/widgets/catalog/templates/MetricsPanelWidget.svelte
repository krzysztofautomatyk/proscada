<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, tag = null }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const num = (k: string, d = 0) => Number(cfg[k] ?? d);

  const levelCm = $derived(tag?.value ?? 420);
  const p1Run = $derived(true);
  const p2Run = $derived(false);
  const borderRadius = $derived(num("borderRadius", 10));
</script>

<div class="metrics-panel-box" style:border-radius="{borderRadius}px">
  <div class="card-header">{str("title", "METRICS OVERVIEW")}</div>
  <div class="metrics-grid">
    <!-- Analog Readout: Level -->
    <div class="metric-card">
      <span class="m-label">TANK LEVEL (ANALOG)</span>
      <div class="m-val blue">{levelCm.toFixed(0)} <span class="u">cm</span></div>
      <div class="m-sub">{(levelCm / 10).toFixed(1)} % Span</div>
    </div>

    <!-- Bool Indicator: Pump 1 -->
    <div class="metric-card">
      <div class="m-head">
        <span class="bool-dot" class:on={p1Run}></span>
        <span class="m-label">PUMP 1 (BOOL)</span>
      </div>
      <div class="m-val" class:green={p1Run}>{p1Run ? "RUNNING" : "STOPPED"}</div>
      <div class="m-sub">Lead Pump</div>
    </div>

    <!-- Bool Indicator: Pump 2 -->
    <div class="metric-card">
      <div class="m-head">
        <span class="bool-dot" class:on={p2Run}></span>
        <span class="m-label">PUMP 2 (BOOL)</span>
      </div>
      <div class="m-val" class:green={p2Run}>{p2Run ? "RUNNING" : "STOPPED"}</div>
      <div class="m-sub">Lag Pump</div>
    </div>

    <!-- Analog Readout: Inflow Factor -->
    <div class="metric-card">
      <span class="m-label">INFLOW K (ANALOG)</span>
      <div class="m-val">1.50 <span class="u">×100</span></div>
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
