<script lang="ts">
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import { configOf, readString } from "$lib/components/widgets/shared/config";

  let { widget, design = false }: WidgetRendererProps = $props();

  const cfg = $derived(configOf(widget));

  const segments = $derived.by<string[]>(() => {
    const raw = cfg["path"];
    if (Array.isArray(raw)) return raw.map(String).map((s) => s.trim()).filter(Boolean);
    if (typeof raw === "string") {
      return raw.split(/[/,]/).map((s) => s.trim()).filter(Boolean);
    }
    return [];
  });

  const crumbs = $derived.by(() =>
    segments.map((label, i) => {
      const target = "/" + segments.slice(0, i + 1).join("/");
      return { label, target, current: i === segments.length - 1 };
    }),
  );

  function navigate(target: string, current: boolean) {
    if (design || current) return;
    if (!target.startsWith("/") && !target.startsWith("screen:")) return;
    window.dispatchEvent(
      new CustomEvent("proscada:navigate", {
        detail: { target, params: {}, sourceWidgetId: widget.id },
      }),
    );
  }
</script>

<nav class="breadcrumb" aria-label="Breadcrumb">
  {#if crumbs.length === 0}
    <span class="empty">No path configured</span>
  {:else}
    <ol>
      {#each crumbs as crumb, i (i)}
        <li>
          {#if crumb.current}
            <span class="crumb current" aria-current="page">{crumb.label}</span>
          {:else}
            <button
              type="button"
              class="crumb"
              disabled={design}
              onclick={() => navigate(crumb.target, crumb.current)}
              title={design ? "Navigation disabled in design" : `Navigate to ${crumb.target}`}
            >{crumb.label}</button>
          {/if}
          {#if i < crumbs.length - 1}<span class="sep" aria-hidden="true">/</span>{/if}
        </li>
      {/each}
    </ol>
  {/if}
</nav>

<style>
  .breadcrumb {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    padding: 4px 8px;
    background: #f8fafc;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    overflow-x: auto;
    font-family: "Segoe UI", system-ui, sans-serif;
  }
  ol {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    align-items: center;
    gap: 2px;
    flex-wrap: nowrap;
  }
  li {
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }
  .crumb {
    border: none;
    background: transparent;
    font-size: 11px;
    color: #2563eb;
    font-weight: 600;
    padding: 2px 4px;
    border-radius: 4px;
    cursor: pointer;
    white-space: nowrap;
  }
  .crumb:hover:not(:disabled):not(.current) {
    background: #dbeafe;
    text-decoration: underline;
  }
  .crumb:focus-visible {
    outline: 2px solid #2563eb;
    outline-offset: -1px;
  }
  .crumb:disabled {
    cursor: default;
    color: #64748b;
  }
  .crumb.current {
    color: #1e293b;
    font-weight: 800;
    cursor: default;
  }
  .sep {
    color: #94a3b8;
    font-size: 10px;
  }
  .empty {
    font-size: 10px;
    color: #94a3b8;
  }
</style>
