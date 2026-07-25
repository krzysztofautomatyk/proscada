<script lang="ts">
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import { configOf, readString, readStringList } from "$lib/components/widgets/shared/config";
  import QualityBadge from "$lib/components/widgets/shared/QualityBadge.svelte";

  let { widget, tag = null }: WidgetRendererProps = $props();

  const cfg = $derived(configOf(widget));
  const title = $derived(readString(cfg, "title", ""));
  const tabs = $derived(readStringList(cfg, "tabs", ["Tab 1", "Tab 2"]));

  let activeIndex = $state(0);
  let tabRefs = $state<HTMLButtonElement[]>([]);

  const safeIndex = $derived(Math.min(activeIndex, Math.max(0, tabs.length - 1)));

  function contentFor(label: string): string {
    const value = cfg[`content:${label}`];
    if (value === undefined || value === null) return `Preview content for "${label}".`;
    return String(value);
  }

  function focusTab(i: number) {
    const ref = tabRefs[i];
    if (ref) ref.focus();
  }

  function onKey(event: KeyboardEvent) {
    if (tabs.length === 0) return;
    let next = safeIndex;
    if (event.key === "ArrowRight" || event.key === "ArrowDown") next = (safeIndex + 1) % tabs.length;
    else if (event.key === "ArrowLeft" || event.key === "ArrowUp")
      next = (safeIndex - 1 + tabs.length) % tabs.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = tabs.length - 1;
    else return;
    event.preventDefault();
    activeIndex = next;
    focusTab(next);
  }
</script>

<div class="tabset">
  <div class="bar">
    {#if title}<span class="title">{title}</span>{/if}
    <div class="tablist" role="tablist" aria-label={title || "Tabs"}>
      {#each tabs as label, i (i)}
        <button
          bind:this={tabRefs[i]}
          type="button"
          role="tab"
          class="tab"
          class:active={i === safeIndex}
          aria-selected={i === safeIndex}
          tabindex={i === safeIndex ? 0 : -1}
          onclick={() => (activeIndex = i)}
          onkeydown={onKey}
        >{label}</button>
      {/each}
    </div>
    {#if tag}<QualityBadge {tag} />{/if}
  </div>
  <div class="panel" role="tabpanel">
    {#if tabs.length === 0}
      <span class="hint">No tabs configured</span>
    {:else}
      <div class="preview">
        <strong>{tabs[safeIndex]}</strong>
        <p>{contentFor(tabs[safeIndex])}</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .tabset {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    border: 1px solid #d8dee8;
    border-radius: 7px;
    background: #fff;
    overflow: hidden;
    font-family: "Segoe UI", system-ui, sans-serif;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 6px;
    background: #f8fafc;
    border-bottom: 1px solid #e5e7eb;
  }
  .title {
    font-size: 10px;
    font-weight: 800;
    text-transform: uppercase;
    color: #334155;
    white-space: nowrap;
  }
  .tablist {
    display: flex;
    gap: 2px;
    flex: 1;
    min-width: 0;
    overflow-x: auto;
  }
  .tab {
    border: 1px solid transparent;
    border-bottom: none;
    background: transparent;
    color: #64748b;
    font-size: 10px;
    font-weight: 700;
    padding: 4px 10px;
    border-radius: 5px 5px 0 0;
    cursor: pointer;
    white-space: nowrap;
  }
  .tab:hover {
    color: #1e293b;
    background: #eef2f7;
  }
  .tab.active {
    color: #1d4ed8;
    background: #fff;
    border-color: #cbd5e1;
    border-bottom: 2px solid #2563eb;
  }
  .tab:focus-visible {
    outline: 2px solid #2563eb;
    outline-offset: -2px;
  }
  .panel {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 8px 10px;
  }
  .preview strong {
    font-size: 11px;
    color: #1e293b;
  }
  .preview p {
    margin: 4px 0 0;
    font-size: 10px;
    color: #475569;
    white-space: pre-wrap;
  }
  .hint {
    font-size: 10px;
    color: #94a3b8;
  }
</style>
