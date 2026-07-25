<script lang="ts">
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import { configOf, readString } from "$lib/components/widgets/shared/config";
  import EmptyState from "$lib/components/widgets/shared/EmptyState.svelte";

  let { widget, design = false }: WidgetRendererProps = $props();

  interface MenuItem {
    label: string;
    groups: string[];
    depth: number;
    target: string;
    safe: boolean;
  }

  const cfg = $derived(configOf(widget));
  const title = $derived(readString(cfg, "title", "Menu"));

  const items = $derived.by<MenuItem[]>(() => {
    const raw = cfg["items"];
    const lines =
      typeof raw === "string"
        ? raw.split(/\r?\n/)
        : Array.isArray(raw)
          ? raw.map(String)
          : [];
    const out: MenuItem[] = [];
    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) continue;
      const eq = trimmed.indexOf("=");
      const labelPath = eq >= 0 ? trimmed.slice(0, eq).trim() : trimmed;
      const target = eq >= 0 ? trimmed.slice(eq + 1).trim() : "";
      const segments = labelPath.split("/").map((s) => s.trim()).filter(Boolean);
      const label = segments.length ? segments[segments.length - 1] : labelPath;
      const groups = segments.slice(0, -1);
      out.push({
        label,
        groups,
        depth: groups.length,
        target,
        safe: target.startsWith("/") || target.startsWith("screen:"),
      });
    }
    return out;
  });

  let selected = $state(-1);
  let refs = $state<HTMLButtonElement[]>([]);

  function activate(item: MenuItem, index: number) {
    selected = index;
    if (design || !item.safe) return;
    window.dispatchEvent(
      new CustomEvent("proscada:navigate", {
        detail: { target: item.target, params: {}, sourceWidgetId: widget.id },
      }),
    );
  }

  function onKey(event: KeyboardEvent, index: number) {
    let next = index;
    if (event.key === "ArrowDown") next = Math.min(items.length - 1, index + 1);
    else if (event.key === "ArrowUp") next = Math.max(0, index - 1);
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = items.length - 1;
    else return;
    event.preventDefault();
    refs[next]?.focus();
  }
</script>

<div class="menu">
  <div class="head">{title}</div>
  {#if items.length === 0}
    <EmptyState title="No menu items" detail="Add items as label=screen:target per line" icon="☰" />
  {:else}
    <ul role="menu" aria-label={title}>
      {#each items as item, i (i)}
        <li>
          <button
            bind:this={refs[i]}
            type="button"
            role="menuitem"
            class="item"
            class:selected={selected === i}
            class:blocked={!item.safe}
            disabled={!item.safe}
            style:padding-left="{8 + item.depth * 14}px"
            aria-current={selected === i ? "true" : undefined}
            title={item.safe ? item.target : `Blocked target: ${item.target || "(empty)"}`}
            onclick={() => activate(item, i)}
            onkeydown={(e) => onKey(e, i)}
          >
            {#if item.depth > 0}<span class="tree" aria-hidden="true">└</span>{/if}
            {#if item.groups.length}<span class="group">{item.groups.join(" / ")} /</span>{/if}
            <span class="label">{item.label}</span>
            {#if !item.safe}<span class="blocked-tag">blocked</span>{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .menu {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    border: 1px solid #d8dee8;
    border-radius: 7px;
    background: #fff;
    overflow: hidden;
    font-family: "Segoe UI", system-ui, sans-serif;
  }
  .head {
    padding: 5px 8px;
    background: #f1f5f9;
    border-bottom: 1px solid #e5e7eb;
    font-size: 10px;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #334155;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 4px;
    overflow: auto;
    flex: 1;
    min-height: 0;
  }
  .item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 5px;
    border: 1px solid transparent;
    border-radius: 5px;
    background: transparent;
    color: #1e293b;
    font-size: 11px;
    padding: 5px 8px;
    cursor: pointer;
    text-align: left;
  }
  .item:hover:not(:disabled) {
    background: #eff6ff;
  }
  .item.selected {
    background: #dbeafe;
    border-color: #93c5fd;
    font-weight: 700;
  }
  .item:focus-visible {
    outline: 2px solid #2563eb;
    outline-offset: -2px;
  }
  .item.blocked {
    color: #b91c1c;
    background: #fef2f2;
    cursor: not-allowed;
  }
  .tree {
    color: #94a3b8;
    font-size: 10px;
  }
  .group {
    color: #94a3b8;
    font-size: 9px;
  }
  .label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .blocked-tag {
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border: 1px solid currentColor;
    border-radius: 3px;
    padding: 0 3px;
  }
</style>
