<script lang="ts">
  import type { RegisterMapEntry } from "$lib/types/registerMap";
  import RegisterBitMapper from "./RegisterBitMapper.svelte";

  interface Props {
    entries: RegisterMapEntry[];
    onToggleReadonly: (tagId: string, currentReadonly: boolean) => void;
    onSelectRegisterForEdit: (address: number, tagId?: string) => void;
    onAddTagAtAddress: (address: number) => void;
    onEditBitTag?: (address: number, bitIndex: number) => void;
  }

  let {
    entries,
    onToggleReadonly,
    onSelectRegisterForEdit,
    onAddTagAtAddress,
    onEditBitTag,
  }: Props = $props();

  let searchQuery = $state("");
  let filterType = $state<"all" | "mapped" | "unmapped" | "readonly" | "writable">("all");
  let expandedAddress = $state<number | null>(null);

  const filteredEntries = $derived.by(() => {
    let list = entries;
    if (searchQuery.trim()) {
      const q = searchQuery.trim().toLowerCase();
      list = list.filter(
        (e) =>
          e.address.toString().includes(q) ||
          e.symbol.toLowerCase().includes(q) ||
          e.tagId.toLowerCase().includes(q) ||
          e.description.toLowerCase().includes(q)
      );
    }

    if (filterType === "mapped") list = list.filter((e) => e.dataType !== "unmapped");
    if (filterType === "unmapped") list = list.filter((e) => e.dataType === "unmapped");
    if (filterType === "readonly") list = list.filter((e) => e.readonly && e.dataType !== "unmapped");
    if (filterType === "writable") list = list.filter((e) => !e.readonly && e.dataType !== "unmapped");

    return list;
  });

  const stats = $derived.by(() => {
    const mapped = entries.filter((e) => e.dataType !== "unmapped" && !e.isSpanContinuation);
    const readonlyCount = mapped.filter((e) => e.readonly).length;
    const writableCount = mapped.filter((e) => !e.readonly).length;
    const unmappedCount = entries.filter((e) => e.dataType === "unmapped").length;
    return { mapped: mapped.length, readonly: readonlyCount, writable: writableCount, unmapped: unmappedCount };
  });

  function toggleBitDrawer(address: number) {
    expandedAddress = expandedAddress === address ? null : address;
  }
</script>

<div class="map-container">
  <!-- Filter & Search Toolbar -->
  <div class="toolbar">
    <div class="search-box">
      <span class="search-icon">🔍</span>
      <input
        type="text"
        placeholder="Filtruj rejestry (np. 100, LEVEL, SP_P1)..."
        bind:value={searchQuery}
      />
    </div>

    <div class="filter-tabs">
      <button
        type="button"
        class="filter-btn"
        class:active={filterType === "all"}
        onclick={() => (filterType = "all")}
      >
        Wszystkie ({entries.length})
      </button>
      <button
        type="button"
        class="filter-btn"
        class:active={filterType === "mapped"}
        onclick={() => (filterType = "mapped")}
      >
        Zmapowane ({stats.mapped})
      </button>
      <button
        type="button"
        class="filter-btn"
        class:active={filterType === "writable"}
        onclick={() => (filterType = "writable")}
      >
        R/W Zapisywalne ({stats.writable})
      </button>
      <button
        type="button"
        class="filter-btn"
        class:active={filterType === "readonly"}
        onclick={() => (filterType = "readonly")}
      >
        Read-Only ({stats.readonly})
      </button>
      <button
        type="button"
        class="filter-btn"
        class:active={filterType === "unmapped"}
        onclick={() => (filterType = "unmapped")}
      >
        Wolne ({stats.unmapped})
      </button>
    </div>
  </div>

  <!-- Register Map Table -->
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th style="width: 80px;">Adres HR</th>
          <th style="width: 180px;">Symbol / Nazwa</th>
          <th style="width: 160px;">Tag ID</th>
          <th style="width: 80px; text-align: center;">R/W</th>
          <th style="width: 80px; text-align: center;">Typ</th>
          <th style="width: 90px;">Live</th>
          <th style="width: 80px;">Hex</th>
          <th style="width: 110px;">Jednostka</th>
          <th>Opis</th>
          <th style="width: 120px; text-align: center;">Akcje</th>
        </tr>
      </thead>
      <tbody>
        {#if filteredEntries.length === 0}
          <tr>
            <td colspan="10" class="empty-cell">Brak rejestrów spełniających kryteria wyszukiwania.</td>
          </tr>
        {/if}

        {#each filteredEntries as row (row.address)}
          <tr class:span-continuation={row.isSpanContinuation} class:unmapped={row.dataType === "unmapped"}>
            <!-- Register Address -->
            <td class="mono bold addr-cell">
              R{row.address}
            </td>

            <!-- Symbol / Variable Name -->
            <td>
              {#if row.isSpanContinuation}
                <span class="span-label">↳ kontynuacja f32 z R{row.parentAddress}</span>
              {:else if row.tags && row.tags.length > 1}
                <div class="multi-tags-cell">
                  {#each row.tags as t}
                    <button
                      type="button"
                      class="tag-chip"
                      class:is-bool={t.data_type === "bool"}
                      title="Kliknij, aby edytować {t.name} ({t.id})"
                      onclick={() => onSelectRegisterForEdit(row.address, t.id)}
                    >
                      {t.name}
                      {#if t.data_type === "bool" && t.binding.bit !== null && t.binding.bit !== undefined}
                        <small class="bit-idx">.b{t.binding.bit}</small>
                      {/if}
                    </button>
                  {/each}
                </div>
              {:else if row.dataType !== "unmapped"}
                <span class="symbol-name">{row.symbol}</span>
              {:else}
                <span class="unmapped-label">— (Wolny)</span>
              {/if}
            </td>

            <!-- Tag ID -->
            <td class="mono small-text">
              {#if row.tags && row.tags.length > 1}
                <span class="multi-count-badge">{row.tags.length} zmienne</span>
              {:else}
                {row.tagId || "—"}
              {/if}
            </td>

            <!-- Read-Only vs R/W Toggle Switch -->
            <td style="text-align: center;">
              {#if row.dataType !== "unmapped" && !row.isSpanContinuation}
                <button
                  type="button"
                  class="rw-badge"
                  class:is-readonly={row.readonly}
                  class:is-rw={!row.readonly}
                  title={row.readonly ? "Tylko Odczyt (Aplikacja nie może zapisywać) — Kliknij, aby zmienić na R/W" : "Odczyt i Zapis (R/W) — Kliknij, aby zmienić na Read-Only"}
                  onclick={() => onToggleReadonly(row.tagId, row.readonly)}
                >
                  {row.readonly ? "R" : "R/W"}
                </button>
              {:else}
                <span class="rw-badge disabled">—</span>
              {/if}
            </td>

            <!-- Data Type Badge -->
            <td style="text-align: center;">
              {#if row.dataType !== "unmapped"}
                <span class="type-badge {row.dataType}">{row.dataType}</span>
              {:else}
                <span class="type-badge unmapped">empty</span>
              {/if}
            </td>

            <!-- Live Value Preview -->
            <td class="mono live-val">
              {row.liveValue !== undefined ? row.liveValue : "—"}
            </td>

            <!-- Hex Value -->
            <td class="mono small-text muted">
              {row.hexValue || "0x0000"}
            </td>

            <!-- Unit & Scale -->
            <td class="unit-cell">
              {row.unit ? `${row.unit}` : "—"}
              {#if row.scale !== 1}<small> (×{row.scale})</small>{/if}
            </td>

            <!-- Description -->
            <td class="desc-cell" title={row.description}>
              {row.description}
            </td>

            <!-- Action buttons -->
            <td class="actions-cell">
              {#if row.dataType !== "unmapped" && !row.isSpanContinuation}
                <button
                  type="button"
                  class="btn-icon"
                  title="Edytuj Zmienną"
                  onclick={() => onSelectRegisterForEdit(row.address, row.tagId)}
                >
                  ✏️
                </button>

                {#if row.table === "holding" || row.table === "input"}
                  <button
                    type="button"
                    class="btn-icon"
                    class:active={expandedAddress === row.address}
                    class:has-active-bits={row.bits && row.bits.some((b) => !!b.tagId)}
                    title="Pokaż bity 0..15"
                    onclick={() => toggleBitDrawer(row.address)}
                  >
                    ⚡ Bity {row.bits && row.bits.some((b) => !!b.tagId) ? `(${row.bits.filter((b) => !!b.tagId).length})` : ""}
                  </button>
                {/if}
              {:else if row.dataType === "unmapped"}
                <button
                  type="button"
                  class="btn-add-tag"
                  onclick={() => onAddTagAtAddress(row.address)}
                >
                  ➕ Utwórz
                </button>
                {#if row.table === "holding" || row.table === "input"}
                  <button
                    type="button"
                    class="btn-icon"
                    class:active={expandedAddress === row.address}
                    title="Definiuj bity 0..15"
                    onclick={() => toggleBitDrawer(row.address)}
                  >
                    ⚡
                  </button>
                {/if}
              {/if}
            </td>
          </tr>

          <!-- Expandable Bit Drawer -->
          {#if expandedAddress === row.address && row.bits}
            <tr class="drawer-row">
              <td colspan="10">
                <RegisterBitMapper
                  address={row.address}
                  bits={row.bits}
                  onEditBitTag={(bitIdx) => onEditBitTag && onEditBitTag(row.address, bitIdx)}
                />
              </td>
            </tr>
          {/if}
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .map-container {
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: #141418;
    border: 1px solid #282d37;
    border-radius: 6px;
    padding: 12px;
    flex: 1;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .search-box {
    display: flex;
    align-items: center;
    background: #0f0f13;
    border: 1px solid #334155;
    border-radius: 4px;
    padding: 4px 8px;
    flex: 1;
    max-width: 320px;
  }

  .search-icon { font-size: 12px; margin-right: 6px; }

  .search-box input {
    background: transparent;
    border: none;
    color: #f8fafc;
    font-size: 12px;
    outline: none;
    width: 100%;
  }

  .filter-tabs { display: flex; gap: 4px; }

  .filter-btn {
    background: #1e1e26;
    border: 1px solid #2d3748;
    color: #94a3b8;
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.12s;
  }

  .filter-btn.active {
    background: #15803d;
    border-color: #22c55e;
    color: #fff;
    font-weight: 600;
  }

  .table-wrap {
    flex: 1;
    max-height: calc(95vh - 280px);
    overflow-y: auto;
    border: 1px solid #272738;
    border-radius: 4px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11.5px;
    text-align: left;
  }

  th {
    background: #1f1f28;
    color: #94a3b8;
    padding: 7px 10px;
    border-bottom: 1px solid #334155;
    font-weight: 600;
    position: sticky;
    top: 0;
    z-index: 2;
  }

  td {
    padding: 6px 10px;
    border-bottom: 1px solid #1f1f28;
    color: #cbd5e1;
  }

  tr:nth-child(even) { background: #16161c; }
  tr:hover { background: #1f2430; }

  tr.unmapped { opacity: 0.75; }
  tr.span-continuation { background: #181c26; opacity: 0.6; }

  .mono { font-family: monospace; }
  .bold { font-weight: 700; }
  .addr-cell { color: #38bdf8; }

  .symbol-name { font-weight: 600; color: #f8fafc; }
  .unmapped-label { color: #64748b; font-style: italic; }
  .span-label { font-size: 10px; color: #a7f3d0; font-style: italic; }
  .small-text { font-size: 10.5px; color: #94a3b8; }
  .muted { color: #64748b; }

  .rw-badge {
    padding: 2px 7px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 700;
    cursor: pointer;
    border: none;
    transition: transform 0.1s;
  }

  .rw-badge:hover { transform: scale(1.08); }

  .rw-badge.is-rw {
    background: #16a34a;
    color: #ffffff;
    box-shadow: 0 0 6px rgba(34, 197, 94, 0.4);
  }

  .rw-badge.is-readonly {
    background: #d97706;
    color: #ffffff;
    box-shadow: 0 0 6px rgba(217, 119, 6, 0.4);
  }

  .rw-badge.disabled {
    background: #272730;
    color: #64748b;
    cursor: default;
  }

  .multi-tags-cell {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .tag-chip {
    background: var(--gh-canvas-subtle, #161b22);
    border: 1px solid var(--gh-border-default, #30363d);
    color: var(--gh-fg-default, #e6edf3);
    font-size: 10.5px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.12s;
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }

  .tag-chip:hover {
    background: var(--copilot-purple, #8957e5);
    border-color: var(--copilot-purple-light, #a371f7);
    color: #fff;
  }

  .tag-chip.is-bool {
    background: rgba(57, 197, 207, 0.15);
    border-color: var(--copilot-cyan, #39c5cf);
    color: var(--copilot-cyan, #39c5cf);
  }

  .bit-idx {
    color: var(--copilot-cyan, #39c5cf);
    font-family: var(--font-mono, monospace);
    font-size: 9.5px;
  }

  .multi-count-badge {
    background: rgba(137, 87, 229, 0.2);
    border: 1px solid var(--copilot-purple-light, #a371f7);
    color: var(--copilot-purple-light, #a371f7);
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 700;
  }

  .type-badge.multi { background: var(--copilot-purple, #8957e5); }

  .live-val { color: #4ade80; font-weight: 600; }
  .unit-cell small { color: var(--gh-fg-subtle, #6e7681); }
  .desc-cell {
    max-width: 200px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--gh-fg-muted, #848d97);
  }

  .actions-cell { display: flex; gap: 4px; justify-content: center; }

  .btn-icon {
    background: var(--gh-canvas-inset, #010409);
    border: 1px solid var(--gh-border-default, #30363d);
    color: var(--gh-fg-default, #e6edf3);
    padding: 3px 6px;
    border-radius: 3px;
    font-size: 11px;
    cursor: pointer;
  }

  .btn-icon:hover { background: var(--copilot-purple, #8957e5); }
  .btn-icon.active { background: var(--copilot-purple, #8957e5); border-color: var(--copilot-purple-light, #a371f7); }

  .btn-add-tag {
    background: #16a34a;
    border: none;
    color: #fff;
    padding: 3px 8px;
    border-radius: 3px;
    font-size: 10.5px;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-add-tag:hover { background: #15803d; }

  .drawer-row td {
    padding: 0;
    background: #0d0d12;
  }

  .empty-cell {
    text-align: center;
    padding: 24px;
    color: #64748b;
    font-style: italic;
  }
</style>
