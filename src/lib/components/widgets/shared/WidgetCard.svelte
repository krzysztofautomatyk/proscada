<script lang="ts">
  import type { Snippet } from "svelte";
  import type { TagValue } from "$lib/types";
  import QualityBadge from "./QualityBadge.svelte";

  interface Props {
    title: string;
    subtitle?: string;
    tag?: TagValue | null;
    accent?: string;
    compact?: boolean;
    children: Snippet;
    actions?: Snippet;
  }

  let {
    title,
    subtitle = "",
    tag = null,
    accent = "#2563eb",
    compact = false,
    children,
    actions,
  }: Props = $props();
</script>

<section class:compact class="card" style:--accent={accent}>
  <header>
    <div class="heading">
      <strong>{title}</strong>
      {#if subtitle}<span>{subtitle}</span>{/if}
    </div>
    <div class="header-actions">
      {#if tag}<QualityBadge {tag} />{/if}
      {#if actions}{@render actions()}{/if}
    </div>
  </header>
  <div class="body">{@render children()}</div>
</section>

<style>
  .card {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid var(--psc-border, #d8dee8);
    border-top: 3px solid var(--accent, #2563eb);
    border-radius: 7px;
    background: var(--psc-surface, #ffffff);
    color: var(--psc-text, #172033);
    font-family: var(--psc-font-family, "Segoe UI", system-ui, sans-serif);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-height: 30px;
    padding: 5px 8px;
    border-bottom: 1px solid #e5e7eb;
    background: #f8fafc;
  }
  .heading {
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: 6px;
  }
  strong {
    overflow: hidden;
    color: #1e293b;
    font-size: 10px;
    letter-spacing: 0.04em;
    text-overflow: ellipsis;
    text-transform: uppercase;
    white-space: nowrap;
  }
  .heading span {
    overflow: hidden;
    color: #64748b;
    font-size: 9px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .body {
    min-height: 0;
    flex: 1;
    overflow: hidden;
  }
  .compact header {
    min-height: 22px;
    padding: 3px 6px;
  }
</style>
