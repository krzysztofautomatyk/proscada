<script lang="ts">
  import type { WidgetRendererProps } from "$lib/components/widgets/shared/types";
  import {
    configOf,
    readString,
    readStringList,
    readNumber,
    readBoolean,
    readRecordList,
  } from "$lib/components/widgets/shared/config";
  import WidgetCard from "$lib/components/widgets/shared/WidgetCard.svelte";
  import EmptyState from "$lib/components/widgets/shared/EmptyState.svelte";

  let { widget, tag = null, design = false }: WidgetRendererProps = $props();

  const cfg = $derived(configOf(widget));
  const title = $derived(readString(cfg, "title", "Collection"));
  const variant = $derived(
    (["list", "table", "grid"].includes(readString(cfg, "variant", "table"))
      ? readString(cfg, "variant", "table")
      : "table") as "list" | "table" | "grid",
  );
  const pageSize = $derived(Math.max(1, readNumber(cfg, "pageSize", 5, 1, 500)));
  const selectable = $derived(readBoolean(cfg, "selectable", false));
  const loading = $derived(readBoolean(cfg, "loading", false));

  const parsed = $derived(readRecordList(cfg, "rows"));
  const rows = $derived(parsed.rows);
  const configError = $derived(parsed.error);

  const columns = $derived.by<string[]>(() => {
    const explicit = readStringList(cfg, "columns");
    if (explicit.length) return explicit;
    if (rows.length) return Object.keys(rows[0]);
    return [];
  });

  let page = $state(0);
  let selectedIndex = $state(-1);

  const pageCount = $derived(Math.max(1, Math.ceil(rows.length / pageSize)));
  const safePage = $derived(Math.min(page, pageCount - 1));
  const pageRows = $derived.by(() => {
    const start = safePage * pageSize;
    return rows.slice(start, start + pageSize).map((row, i) => ({ row, index: start + i }));
  });

  function cellText(row: Record<string, unknown>, col: string): string {
    const value = row[col];
    if (value === undefined || value === null) return "";
    if (typeof value === "object") return JSON.stringify(value);
    return String(value);
  }

  function toggle(index: number) {
    if (!selectable || design) return;
    selectedIndex = selectedIndex === index ? -1 : index;
  }

  function onRowKey(event: KeyboardEvent, index: number) {
    if (!selectable) return;
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      toggle(index);
    }
  }

  function prev() {
    if (safePage > 0) page = safePage - 1;
  }
  function next() {
    if (safePage < pageCount - 1) page = safePage + 1;
  }
</script>

<WidgetCard {title} subtitle="{rows.length} rows · {variant}" {tag} accent="#4f46e5">
  <div class="collection">
    {#if configError}
      <div class="cfg-error" role="alert"><strong>Config error</strong><span>{configError}</span></div>
    {:else if loading}
      <div class="loading" role="status"><span class="spinner" aria-hidden="true"></span>Loading…</div>
    {:else if rows.length === 0}
      <EmptyState title="No records" detail="Provide rows JSON array in configuration" icon="▤" />
    {:else}
      <div class="scroll">
        {#if variant === "table"}
          <table>
            <thead>
              <tr>
                {#each columns as col (col)}<th>{col}</th>{/each}
              </tr>
            </thead>
            <tbody>
              {#each pageRows as { row, index } (index)}
                <tr
                  class:selected={selectedIndex === index}
                  class:selectable
                  tabindex="0"
                  role={selectable ? "button" : undefined}
                  aria-pressed={selectedIndex === index}
                aria-disabled={!selectable}
                  onclick={() => toggle(index)}
                  onkeydown={(e) => onRowKey(e, index)}
                >
                  {#each columns as col (col)}<td>{cellText(row, col)}</td>{/each}
                </tr>
              {/each}
            </tbody>
          </table>
        {:else if variant === "list"}
          <ul class="list">
            {#each pageRows as { row, index } (index)}
              <li class:selected={selectedIndex === index} class:selectable>
                <button
                  type="button"
                  class="row-button"
                  aria-pressed={selectedIndex === index}
                  disabled={!selectable}
                  onclick={() => toggle(index)}
                  onkeydown={(e) => onRowKey(e, index)}
                >
                  {#each columns as col (col)}
                    <span class="kv"><em>{col}</em>{cellText(row, col)}</span>
                  {/each}
                </button>
              </li>
            {/each}
          </ul>
        {:else}
          <div class="grid">
            {#each pageRows as { row, index } (index)}
              <div
                class="cell"
                class:selected={selectedIndex === index}
                class:selectable
                tabindex="0"
                role="button"
                aria-pressed={selectedIndex === index}
                aria-disabled={!selectable}
                onclick={() => toggle(index)}
                onkeydown={(e) => onRowKey(e, index)}
              >
                {#each columns as col (col)}
                  <div class="kv"><em>{col}</em><span>{cellText(row, col)}</span></div>
                {/each}
              </div>
            {/each}
          </div>
        {/if}
      </div>
      <div class="pager">
        <button type="button" onclick={prev} disabled={safePage === 0}>‹ Prev</button>
        <span>Page {safePage + 1} / {pageCount}</span>
        <button type="button" onclick={next} disabled={safePage >= pageCount - 1}>Next ›</button>
      </div>
    {/if}
  </div>
</WidgetCard>

<style>
  .collection {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
  }
  .scroll {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 10px;
  }
  th,
  td {
    padding: 3px 6px;
    border-bottom: 1px solid #eef2f7;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 180px;
  }
  th {
    position: sticky;
    top: 0;
    background: #f1f5f9;
    color: #334155;
    font-weight: 700;
  }
  tr.selectable {
    cursor: pointer;
  }
  tr.selected,
  li.selected,
  .cell.selected {
    background: #eef2ff;
    outline: 1px solid #6366f1;
  }
  .list {
    list-style: none;
    margin: 0;
    padding: 4px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .row-button {
    display: contents;
    appearance: none;
    background: none;
    border: 0;
    padding: 0;
    font: inherit;
    color: inherit;
    text-align: inherit;
    cursor: pointer;
  }
  .row-button:disabled {
    cursor: default;
  }
  .list li {
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    padding: 5px 7px;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    font-size: 10px;
  }
  .list li.selectable {
    cursor: pointer;
  }
  .kv {
    display: inline-flex;
    gap: 4px;
    align-items: baseline;
  }
  .kv em {
    color: #64748b;
    font-style: normal;
    font-weight: 700;
    font-size: 9px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 6px;
    padding: 6px;
  }
  .grid .cell {
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 10px;
  }
  .grid .cell.selectable {
    cursor: pointer;
  }
  .grid .kv {
    display: flex;
    justify-content: space-between;
  }
  .pager {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    padding: 4px 6px;
    border-top: 1px solid #eef2f7;
    background: #f8fafc;
    font-size: 9px;
    color: #475569;
  }
  .pager button {
    border: 1px solid #cbd5e1;
    background: #fff;
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 9px;
    font-weight: 700;
    color: #334155;
    cursor: pointer;
  }
  .pager button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .loading {
    margin: auto;
    display: flex;
    align-items: center;
    gap: 8px;
    color: #475569;
    font-size: 11px;
  }
  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid #cbd5e1;
    border-top-color: #4f46e5;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .cfg-error {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px;
    margin: auto;
    border: 1px solid #fca5a5;
    border-radius: 6px;
    background: #fef2f2;
    color: #991b1b;
    font-size: 10px;
  }
  .cfg-error strong {
    font-size: 11px;
  }
</style>
