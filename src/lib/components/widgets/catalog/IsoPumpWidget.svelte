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
  const pumpName = $derived(str("pumpName", "PUMP 1"));

  const isRunning = $derived(tag?.bool_value ?? false);
  const isFault = $derived(Boolean(cfg.fault ?? false));
</script>

<div class="iso-pump-card" class:running={isRunning} class:fault={isFault}>
  <div class="pump-header">
    <span class="dot" class:run={isRunning} class:flt={isFault}></span>
    <span class="pump-title">{pumpName}</span>
    <span class="badge" class:green={isRunning} class:red={isFault}>
      {isFault ? "FAULT" : isRunning ? "RUNNING" : "STOPPED"}
    </span>
  </div>

  <div class="pump-visual">
    <svg viewBox="0 0 160 100" class="pump-svg">
      <!-- Base casing -->
      <rect x="20" y="70" width="120" height="16" rx="4" fill="#4b5563" />
      <rect x="35" y="30" width="90" height="42" rx="8" fill="#374151" stroke="#6b7280" stroke-width="2" />
      
      <!-- Impeller Housing -->
      <circle cx="80" cy="50" r="22" fill={isRunning ? "#15803d" : isFault ? "#b91c1c" : "#1f2937"} stroke="#9ca3af" stroke-width="2" />
      
      <!-- Impeller Blades -->
      <g class="impeller" class:spinning={isRunning} transform-origin="80 50">
        <line x1="80" y1="34" x2="80" y2="66" stroke="#ffffff" stroke-width="3" stroke-linecap="round" />
        <line x1="64" y1="50" x2="96" y2="50" stroke="#ffffff" stroke-width="3" stroke-linecap="round" />
      </g>

      <!-- Connection pipes -->
      <rect x="5" y="42" width="30" height="16" fill="#6b7280" />
      <rect x="125" y="42" width="30" height="16" fill="#6b7280" />
    </svg>
  </div>
</div>

<style>
  .iso-pump-card {
    width: 100%;
    height: 100%;
    background: #ffffff;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 8px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }
  .pump-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    font-size: 11px;
    font-weight: 700;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #9ca3af;
  }
  .dot.run { background: #16a34a; }
  .dot.flt { background: #dc2626; }
  .badge {
    font-size: 9px;
    font-weight: 800;
    padding: 2px 6px;
    border-radius: 99px;
    background: #f3f4f6;
    color: #4b5563;
  }
  .badge.green { background: #dcfce7; color: #15803d; }
  .badge.red { background: #fee2e2; color: #b91c1c; }

  .pump-visual {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .pump-svg {
    width: 100%;
    height: auto;
    max-height: 90px;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .spinning {
    animation: spin 0.6s linear infinite;
  }
</style>
