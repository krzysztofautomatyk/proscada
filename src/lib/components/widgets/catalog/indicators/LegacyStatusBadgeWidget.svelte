<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";
  import { snapshot, tagMap } from "$lib/stores/app";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, tag = null }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);
  const simTagId = $derived(str("simTagId"));
  const frozenTagId = $derived(str("frozenTagId"));
  const simTag = $derived(simTagId ? ($tagMap.get(simTagId) ?? null) : tag);
  const frozenTag = $derived(frozenTagId ? ($tagMap.get(frozenTagId) ?? null) : null);
  const simKnown = $derived(simTag?.quality === "good");
  const frozenKnown = $derived(frozenTag?.quality === "good");
  const isConnected = $derived($snapshot?.connected === true);
</script>

<div class="status-badges-wrap">
  <span class="pill" class:g={simKnown && simTag?.bool_value} class:r={!simKnown}>
    {simKnown ? (simTag?.bool_value ? "SIM ON" : "SIM OFF") : "SIM NO DATA"}
  </span>
  {#if frozenTagId}
    <span class="pill" class:y={frozenKnown && frozenTag?.bool_value} class:r={!frozenKnown}>
      {frozenKnown ? (frozenTag?.bool_value ? "FROZEN" : "LIVE") : "STATE NO DATA"}
    </span>
  {/if}
  <span class="pill" class:g={isConnected} class:r={!isConnected}>
    {isConnected ? "PLC ONLINE" : "PLC OFFLINE"}
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
