<script lang="ts">
  import type { WidgetRendererProps } from "../../shared/types";
  import { configOf, readNumber, readString } from "../../shared/config";

  let { widget, tag = null }: WidgetRendererProps = $props();

  const config = $derived(configOf(widget));
  const environment = $derived(readString(config, "environment", "LIVE").toUpperCase());
  const staleAfter = $derived(readNumber(config, "staleAfterMs", 2000, 1));
  const status = $derived.by(() => {
    if (environment === "SIMULATION" || environment === "SIM") {
      return { key: "simulation", icon: "◆", label: "SIMULATION", detail: "Isolated data source" };
    }
    if (!tag) return { key: "disconnected", icon: "⛓", label: "DISCONNECTED", detail: "No sample" };
    if (tag.quality === "bad") return { key: "bad", icon: "▲", label: "BAD QUALITY", detail: "Source rejected" };
    if (tag.age_ms > staleAfter) return { key: "stale", icon: "◷", label: "STALE DATA", detail: `${Math.round(tag.age_ms)} ms old` };
    if (tag.quality === "uncertain") return { key: "uncertain", icon: "◆", label: "UNCERTAIN", detail: `${Math.round(tag.age_ms)} ms old` };
    return { key: "live", icon: "●", label: "LIVE", detail: `${Math.round(tag.age_ms)} ms old` };
  });
</script>

<div class="status {status.key}" role="status" aria-live="polite">
  <span class="icon" aria-hidden="true">{status.icon}</span>
  <div>
    <strong>{status.label}</strong>
    <small>{status.detail}</small>
  </div>
</div>

<style>
  .status {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 8px 10px;
    border: 1px solid #94a3b8;
    border-left: 5px solid #64748b;
    border-radius: 7px;
    background: #f8fafc;
    color: #334155;
  }
  .icon {
    font-size: 20px;
  }
  strong,
  small {
    display: block;
  }
  strong {
    font-size: 11px;
    letter-spacing: 0.04em;
  }
  small {
    margin-top: 2px;
    color: #64748b;
    font-size: 9px;
  }
  .live {
    border-left-color: #16a34a;
    color: #166534;
  }
  .simulation {
    border-left-color: #7c3aed;
    color: #5b21b6;
    background: repeating-linear-gradient(135deg, #faf5ff, #faf5ff 7px, #ede9fe 7px, #ede9fe 14px);
  }
  .uncertain,
  .stale {
    border-left-color: #d97706;
    color: #92400e;
    background: repeating-linear-gradient(135deg, #fffbeb, #fffbeb 7px, #fef3c7 7px, #fef3c7 14px);
  }
  .bad,
  .disconnected {
    border-left-color: #dc2626;
    color: #991b1b;
    background: repeating-linear-gradient(135deg, #fef2f2, #fef2f2 7px, #fee2e2 7px, #fee2e2 14px);
  }
</style>

