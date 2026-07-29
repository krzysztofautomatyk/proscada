<script lang="ts">
  import type { TagValue } from "$lib/types";

  interface Props {
    tagMap: Map<string, TagValue>;
  }

  let { tagMap }: Props = $props();

  const degradedTags = $derived.by(() => {
    const list: TagValue[] = [];
    for (const tag of tagMap.values()) {
      if (tag.quality === "comm_lost" || tag.quality === "bad" || tag.quality === "uncertain") {
        list.push(tag);
      }
    }
    return list;
  });

  const commLostCount = $derived(
    degradedTags.filter((t) => t.quality === "comm_lost" || t.quality === "bad").length
  );
  const staleCount = $derived(
    degradedTags.filter((t) => t.quality === "uncertain").length
  );

  const hasDegradation = $derived(degradedTags.length > 0);

  const latestTs = $derived.by(() => {
    if (degradedTags.length === 0) return null;
    const sorted = [...degradedTags].sort((a, b) => (b.ts ?? "").localeCompare(a.ts ?? ""));
    return sorted[0]?.ts ?? null;
  });

  let dismissed = $state(false);

  function resetDismiss() {
    dismissed = false;
  }
</script>

{#if hasDegradation && !dismissed}
  <div class="comm-banner" class:comm-lost-severity={commLostCount > 0} role="status" aria-live="polite">
    <div class="banner-main">
      <span class="banner-icon">⚡</span>
      <div class="banner-text">
        <strong>
          SYSTEM COMMUNICATION DEGRADED
          {#if commLostCount > 0}
            · {commLostCount} tag{commLostCount > 1 ? 's' : ''} comm lost
          {/if}
          {#if staleCount > 0}
            · {staleCount} tag{staleCount > 1 ? 's' : ''} stale
          {/if}
        </strong>
        <span class="banner-detail">
          PLC/RTU data transport degraded. Process graphics remain visible with last known values.
          {#if latestTs}
            Last valid sync: {latestTs.split("T")[1]?.slice(0, 8) ?? latestTs}
          {/if}
        </span>
      </div>
    </div>
    <div class="banner-actions">
      <button type="button" class="btn-dismiss" onclick={() => (dismissed = true)} title="Ukryj powiadomienie">
        ✕
      </button>
    </div>
  </div>
{/if}

<style>
  .comm-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 14px;
    background: #3b0764;
    color: #f5d0fe;
    border-bottom: 1px solid #a855f7;
    font-size: 11px;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.25);
    z-index: 100;
  }
  .comm-banner.comm-lost-severity {
    background: #4c1d95;
    border-bottom-color: #c084fc;
  }
  .banner-main {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .banner-icon {
    font-size: 14px;
    color: #f472b6;
  }
  .banner-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .banner-text strong {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.03em;
    color: #ffffff;
  }
  .banner-detail {
    font-size: 10px;
    color: #e9d5ff;
  }
  .banner-actions {
    display: flex;
    align-items: center;
  }
  .btn-dismiss {
    background: transparent;
    border: 0;
    color: #e9d5ff;
    font-size: 12px;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 3px;
  }
  .btn-dismiss:hover {
    background: rgba(255, 255, 255, 0.15);
    color: #ffffff;
  }
</style>
