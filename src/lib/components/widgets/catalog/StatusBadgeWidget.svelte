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
  const bool = (k: string, d = false) => Boolean(cfg[k] ?? d);

  const simOn = $derived(bool("simEn", true));
  const isFrozen = $derived(bool("frozen", false));
  const isConnected = $derived(tag ? tag.quality === "good" : true);
</script>

<div class="status-badges-wrap">
  <span class="pill" class:g={simOn}>{simOn ? "SIM ON" : "SIM OFF"}</span>
  <span class="pill" class:y={isFrozen}>{isFrozen ? "FROZEN" : "LIVE"}</span>
  <span class="pill" class:g={isConnected} class:r={!isConnected}>
    {isConnected ? "MODBUS OK" : "OFFLINE"}
  </span>
</div>

<style>
  .status-badges-wrap {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    gap: 6px;
    box-sizing: border-box;
  }
  .pill {
    font-size: 10px;
    font-weight: 800;
    padding: 4px 10px;
    border-radius: 99px;
    background: #e5e7eb;
    color: #4b5563;
  }
  .pill.g {
    background: #dcfce7;
    color: #16a34a;
  }
  .pill.y {
    background: #fef9c3;
    color: #a16207;
  }
  .pill.r {
    background: #fee2e2;
    color: #dc2626;
  }
</style>
