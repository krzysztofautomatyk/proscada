<script lang="ts">
  import type { WidgetRendererProps } from "../../shared/types";
  import { configOf, readBoolean, readString } from "../../shared/config";

  let { widget, design = false }: WidgetRendererProps = $props();
  const config = $derived(configOf(widget));
  const title = $derived(readString(config, "title", "DISCLOSURE"));
  const content = $derived(readString(config, "content", "Grouped screen content"));
  let expanded = $state(true);

  $effect(() => {
    expanded = readBoolean(config, "expanded", true);
  });
</script>

<section class="disclosure">
  <button
    type="button"
    aria-expanded={expanded}
    onclick={(event) => {
      event.stopPropagation();
      expanded = !expanded;
    }}
  >
    <span aria-hidden="true">{expanded ? "▾" : "▸"}</span>
    <strong>{title}</strong>
    {#if design}<small>DESIGN PREVIEW</small>{/if}
  </button>
  {#if expanded}
    <div class="content">{content}</div>
  {/if}
</section>

<style>
  .disclosure {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    overflow: hidden;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    background: #fff;
  }
  button {
    width: 100%;
    min-height: 30px;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    border: 0;
    border-bottom: 1px solid #e2e8f0;
    background: #f8fafc;
    color: #334155;
    text-align: left;
    cursor: pointer;
  }
  strong {
    flex: 1;
    font-size: 10px;
    letter-spacing: 0.03em;
  }
  small {
    color: #64748b;
    font-size: 8px;
  }
  .content {
    height: calc(100% - 31px);
    box-sizing: border-box;
    overflow: auto;
    padding: 9px;
    color: #475569;
    font-size: 10px;
    white-space: pre-wrap;
  }
</style>

