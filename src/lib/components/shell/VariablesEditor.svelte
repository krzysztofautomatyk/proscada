<script lang="ts">
  import type { ScadaProject, TagDefinition } from "$lib/types";
  import { project, snapshot, dirty, log, addVariableModalOpen } from "$lib/stores/app";
  import { SYSTEM_TAG_DEFINITIONS } from "$lib/services/systemTagsService";
  import SingleVariableForm from "../variables/SingleVariableForm.svelte";

  interface Props {
    scada: ScadaProject;
    design?: boolean;
  }

  let { scada, design = true }: Props = $props();

  let filter = $state("");
  let categoryTab = $state<"all" | "plc" | "memory" | "system">("all");
  let editingTag = $state<TagDefinition | null>(null);

  function handleSaveEditedTag(updatedTag: TagDefinition) {
    project.update((p) => {
      if (!p) return p;
      dirty.set(true);
      return {
        ...p,
        tags: p.tags.map((t) => (t.id === updatedTag.id ? updatedTag : t)),
      };
    });
    log(`Edytowano zmienną: ${updatedTag.id}`, "ok");
    editingTag = null;
  }

  const tagValueMap = $derived.by(() => {
    const map = new Map<string, { value: number; boolValue: boolean; stringValue?: string; quality: string }>();
    if ($snapshot?.tags) {
      for (const tv of $snapshot.tags) {
        map.set(tv.tag_id, {
          value: tv.value,
          boolValue: tv.bool_value,
          stringValue: tv.string_value,
          quality: tv.quality,
        });
      }
    }
    return map;
  });

  const combinedTags = $derived.by(() => {
    const plcAndMemTags = scada.tags ?? [];
    const sysTags = SYSTEM_TAG_DEFINITIONS;
    return [...plcAndMemTags, ...sysTags];
  });

  const filteredTags = $derived(
    combinedTags.filter((t) => {
      // Category filter
      if (categoryTab === "plc" && (t.binding.table === "memory" || t.binding.table === "system")) {
        return false;
      }
      if (categoryTab === "memory" && t.binding.table !== "memory") {
        return false;
      }
      if (categoryTab === "system" && t.binding.table !== "system") {
        return false;
      }

      // Text query filter
      if (!filter.trim()) return true;
      const q = filter.toLowerCase();
      return (
        t.name.toLowerCase().includes(q) ||
        t.id.toLowerCase().includes(q) ||
        t.binding.table.toLowerCase().includes(q) ||
        t.data_type.toLowerCase().includes(q) ||
        (t.description ?? "").toLowerCase().includes(q)
      );
    })
  );

  const plcCount = $derived(scada.tags.filter((t) => t.binding.table !== "memory" && t.binding.table !== "system").length);
  const memoryCount = $derived(scada.tags.filter((t) => t.binding.table === "memory").length);
  const systemCount = $derived(SYSTEM_TAG_DEFINITIONS.length);
  const totalCount = $derived(scada.tags.length + systemCount);

  function patchTag(id: string, patch: Partial<TagDefinition>) {
    // System tags are read-only
    if (SYSTEM_TAG_DEFINITIONS.some((st) => st.id === id)) return;
    project.update((p) => {
      if (!p) return p;
      dirty.set(true);
      return {
        ...p,
        tags: p.tags.map((t) => (t.id === id ? { ...t, ...patch } : t)),
      };
    });
  }

  function removeTag(id: string) {
    if (SYSTEM_TAG_DEFINITIONS.some((st) => st.id === id)) {
      alert("Zmienne systemowe są chronione i nie mogą zostać usunięte!");
      return;
    }
    if (!confirm(`Czy na pewno chcesz usunąć zmienną '${id}'?`)) return;
    project.update((p) => {
      if (!p) return p;
      dirty.set(true);
      return { ...p, tags: p.tags.filter((t) => t.id !== id) };
    });
    log(`Tag deleted: ${id}`, "warn");
  }

  function formatLiveValue(tag: TagDefinition): string {
    const live = tagValueMap.get(tag.id);
    if (!live) return "—";

    if (live.stringValue !== undefined) {
      return live.stringValue;
    }
    if (tag.data_type === "bool") {
      return live.boolValue ? "1 (TRUE)" : "0 (FALSE)";
    }
    const numVal = live.value;
    const formatted = numVal.toFixed(tag.decimals ?? 0);
    return tag.unit ? `${formatted} ${tag.unit}` : formatted;
  }
</script>

<div class="vars-manager">
  <!-- Header Toolbar -->
  <div class="toolbar">
    <div class="title-group">
      <span class="title-icon">🏷️</span>
      <span class="title-text">Centralna Baza Zmiennych SCADA</span>
      <span class="badge count-badge">{totalCount} zmiennych</span>
    </div>

    <!-- Category Tabs -->
    <div class="cat-tabs">
      <button
        type="button"
        class="cat-tab"
        class:active={categoryTab === "all"}
        onclick={() => (categoryTab = "all")}
      >
        Wszystkie ({totalCount})
      </button>
      <button
        type="button"
        class="cat-tab"
        class:active={categoryTab === "plc"}
        onclick={() => (categoryTab = "plc")}
      >
        🌐 PLC Modbus ({plcCount})
      </button>
      <button
        type="button"
        class="cat-tab"
        class:active={categoryTab === "memory"}
        onclick={() => (categoryTab = "memory")}
      >
        🧠 Pamięć ({memoryCount})
      </button>
      <button
        type="button"
        class="cat-tab"
        class:active={categoryTab === "system"}
        onclick={() => (categoryTab = "system")}
      >
        ⚙️ Systemowe ({systemCount})
      </button>
    </div>

    <div class="search-box">
      <input
        type="text"
        class="filter-input"
        placeholder="Filtruj (ID, nazwa, typ, tabela, opis)..."
        bind:value={filter}
      />
    </div>

    {#if design}
      <button
        type="button"
        class="btn-add"
        onclick={() => addVariableModalOpen.set(true)}
      >
        ➕ Konfigurator Zmiennych / Menedżer Mapy…
      </button>
    {/if}
  </div>

  <!-- Data Table -->
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th style:width="90px">Status</th>
          <th style:width="140px">Wartość Żywa (Live)</th>
          <th>Nazwa Zmiennej</th>
          <th>Tag ID</th>
          <th style:width="90px">Typ</th>
          <th style:width="100px">Tabela / Źródło</th>
          <th style:width="80px">Adres / Bit</th>
          <th style:width="70px">Dostęp</th>
          <th style:width="80px">Jednostka</th>
          <th>Opis Zmiennej</th>
          {#if design}<th style:width="90px">Akcje</th>{/if}
        </tr>
      </thead>
      <tbody>
        {#each filteredTags as t (t.id)}
          {@const isSys = t.is_system || t.binding.table === "system"}
          {@const live = tagValueMap.get(t.id)}
          {@const qualityClass = isSys ? "good" : (live?.quality ?? "bad")}
          <tr>
            <!-- Status Badge -->
            <td>
              <span class="status-badge {qualityClass}">
                <span class="dot"></span>
                {isSys ? "SYS" : (live?.quality?.toUpperCase() ?? "OFFLINE")}
              </span>
            </td>

            <!-- Live Value -->
            <td class="live-val-cell">
              <span class="live-val-text">{formatLiveValue(t)}</span>
            </td>

            <!-- Name -->
            <td>
              {#if design && !isSys}
                <input
                  class="edit-input"
                  value={t.name}
                  onchange={(e) => patchTag(t.id, { name: e.currentTarget.value })}
                />
              {:else}
                <span class="tag-name">{t.name}</span>
              {/if}
            </td>

            <!-- Tag ID -->
            <td>
              <span class="tag-id-code">{t.id}</span>
            </td>

            <!-- Data Type -->
            <td>
              <span class="type-pill">{t.data_type}</span>
            </td>

            <!-- Table / Source -->
            <td>
              <span class="table-pill {t.binding.table}">
                {#if t.binding.table === "memory"}
                  🧠 Memory
                {:else if t.binding.table === "system"}
                  ⚙️ System
                {:else}
                  🌐 {t.binding.table}
                {/if}
              </span>
            </td>

            <!-- Address / Bit -->
            <td class="mono">
              {#if t.binding.table === "system" || t.binding.table === "memory"}
                —
              {:else}
                R{t.binding.address}{t.binding.bit !== null && t.binding.bit !== undefined ? `.B${t.binding.bit}` : ""}
              {/if}
            </td>

            <!-- Access (R/W vs Read-Only) -->
            <td>
              {#if t.binding.writable && !isSys}
                <span class="access-badge rw">R/W</span>
              {:else}
                <span class="access-badge ro">🔒 Read-Only</span>
              {/if}
            </td>

            <!-- Unit -->
            <td>
              {#if design && !isSys}
                <input
                  class="edit-input-sm"
                  value={t.unit ?? ""}
                  placeholder="—"
                  onchange={(e) => patchTag(t.id, { unit: e.currentTarget.value })}
                />
              {:else}
                {t.unit || "—"}
              {/if}
            </td>

            <!-- Description -->
            <td>
              {#if design && !isSys}
                <input
                  class="edit-input"
                  value={t.description ?? ""}
                  placeholder="Brak opisu..."
                  onchange={(e) => patchTag(t.id, { description: e.currentTarget.value })}
                />
              {:else}
                <span class="desc-text">{t.description || "—"}</span>
              {/if}
            </td>

            <!-- Actions -->
            {#if design}
              <td>
                {#if !isSys}
                  <div class="actions-cell">
                    <button
                      type="button"
                      class="btn-edit"
                      title="Edytuj pełną konfigurację zmiennej..."
                      onclick={() => (editingTag = t)}
                    >
                      ✏️
                    </button>
                    <button
                      type="button"
                      class="btn-del"
                      title="Usuń zmienną"
                      onclick={() => removeTag(t.id)}
                    >
                      🗑️
                    </button>
                  </div>
                {/if}
              </td>
            {/if}
          </tr>
        {:else}
          <tr>
            <td colspan={design ? 11 : 10} class="empty-cell">
              Brak zmiennych spełniających kryteria wyszukiwania.
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

{#if editingTag}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="modal-backdrop" onclick={() => (editingTag = null)} role="presentation">
    <div class="modal-card" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-header">
        <h3>✏️ Edycja Pełnej Konfiguracji Zmiennej: {editingTag.name} ({editingTag.id})</h3>
        <button type="button" class="btn-close" onclick={() => (editingTag = null)}>✕</button>
      </div>
      <div class="modal-body">
        <SingleVariableForm
          devices={scada.devices ?? []}
          existingTags={scada.tags ?? []}
          editingTag={editingTag}
          onSave={handleSaveEditedTag}
          onCancel={() => (editingTag = null)}
        />
      </div>
    </div>
  </div>
{/if}

<style>
  .vars-manager {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--vs-bg, #1e1e1e);
    color: var(--vs-text, #cccccc);
    font-family: var(--font-ui, sans-serif);
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 14px;
    background: var(--vs-bg-2, #252526);
    border-bottom: 1px solid var(--vs-border, #3e3e42);
    flex-wrap: wrap;
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .title-icon { font-size: 16px; }

  .title-text {
    font-size: 13px;
    font-weight: 700;
    color: var(--vs-text-bright, #f3f3f3);
  }

  .count-badge {
    background: var(--vs-accent, #007acc);
    color: #ffffff;
    font-size: 10px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 10px;
  }

  .cat-tabs {
    display: flex;
    gap: 4px;
    background: var(--vs-bg, #1e1e1e);
    padding: 2px;
    border-radius: 4px;
    border: 1px solid var(--vs-border, #3e3e42);
  }

  .cat-tab {
    background: transparent;
    border: none;
    color: var(--vs-text-dim, #9d9d9d);
    font-size: 11px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 3px;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .cat-tab:hover {
    color: var(--vs-text-bright, #ffffff);
    background: rgba(255, 255, 255, 0.06);
  }

  .cat-tab.active {
    background: var(--vs-accent, #007acc);
    color: #ffffff;
    font-weight: 700;
  }

  .search-box {
    flex: 1;
    min-width: 200px;
  }

  .filter-input {
    width: 100%;
    background: var(--vs-bg, #1e1e1e);
    border: 1px solid var(--vs-border, #3e3e42);
    border-radius: 4px;
    color: var(--vs-text-bright, #f3f3f3);
    padding: 5px 10px;
    font-size: 12px;
    outline: none;
  }

  .filter-input:focus {
    border-color: var(--vs-accent, #007acc);
  }

  .btn-add {
    background: var(--vs-accent, #007acc);
    border: 1px solid var(--vs-accent-2, #0e639c);
    color: #ffffff;
    font-size: 11px;
    font-weight: 600;
    padding: 5px 12px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .btn-add:hover {
    background: var(--vs-accent-2, #0e639c);
  }

  .table-wrap {
    flex: 1;
    overflow: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
    text-align: left;
  }

  th {
    position: sticky;
    top: 0;
    background: var(--vs-bg-3, #2d2d30);
    color: var(--vs-text-dim, #9d9d9d);
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    padding: 7px 10px;
    border-bottom: 1px solid var(--vs-border, #3e3e42);
    z-index: 10;
  }

  td {
    padding: 5px 10px;
    border-bottom: 1px solid var(--vs-border-soft, #2b2b2b);
    vertical-align: middle;
  }

  tr:hover td {
    background: var(--vs-selection, #264f78);
    color: #ffffff;
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 10px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 3px;
  }

  .status-badge .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }

  .status-badge.good { background: rgba(34, 197, 94, 0.2); color: #4ade80; }
  .status-badge.bad { background: rgba(239, 68, 68, 0.2); color: #fca5a5; }

  .live-val-cell {
    font-family: var(--font-mono, monospace);
    font-weight: 700;
    color: #4ec9b0;
  }

  .tag-name { font-weight: 600; color: var(--vs-text-bright, #f3f3f3); }
  .tag-id-code { font-family: var(--font-mono, monospace); color: #ce9178; font-size: 11px; }

  .type-pill {
    background: var(--vs-bg-3, #2d2d30);
    border: 1px solid var(--vs-border, #3e3e42);
    color: #4ec9b0;
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 10px;
    font-family: var(--font-mono, monospace);
  }

  .table-pill {
    display: inline-block;
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 600;
  }

  .table-pill.holding { background: rgba(0, 122, 204, 0.2); color: #9cdcfe; }
  .table-pill.input { background: rgba(197, 134, 192, 0.2); color: #c586c0; }
  .table-pill.coil { background: rgba(220, 220, 170, 0.2); color: #dcdcaa; }
  .table-pill.discrete { background: rgba(128, 128, 128, 0.2); color: #cccccc; }
  .table-pill.memory { background: rgba(78, 201, 176, 0.2); color: #4ec9b0; }
  .table-pill.system { background: rgba(86, 156, 214, 0.2); color: #569cd6; }

  .access-badge {
    display: inline-block;
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 700;
  }

  .access-badge.rw { background: rgba(78, 201, 176, 0.2); color: #4ec9b0; }
  .access-badge.ro { background: rgba(206, 145, 120, 0.2); color: #ce9178; }

  .mono { font-family: var(--font-mono, monospace); font-size: 11px; }

  .edit-input {
    width: 100%;
    background: var(--vs-bg, #1e1e1e);
    border: 1px solid var(--vs-border, #3e3e42);
    color: var(--vs-text-bright, #f3f3f3);
    padding: 3px 6px;
    font-size: 12px;
    border-radius: 3px;
  }

  .edit-input-sm {
    width: 60px;
    background: var(--vs-bg, #1e1e1e);
    border: 1px solid var(--vs-border, #3e3e42);
    color: var(--vs-text-bright, #f3f3f3);
    padding: 3px 6px;
    font-size: 12px;
    border-radius: 3px;
  }

  .actions-cell {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }

  .btn-edit,
  .btn-del {
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 13px;
    opacity: 0.75;
    padding: 2px 4px;
    border-radius: 3px;
    transition: opacity 0.1s, background-color 0.1s;
  }

  .btn-edit:hover,
  .btn-del:hover {
    opacity: 1;
    background: rgba(255, 255, 255, 0.1);
  }

  .empty-cell {
    text-align: center;
    padding: 24px;
    color: var(--vs-text-dim, #9d9d9d);
  }

  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    padding: 20px;
  }

  .modal-card {
    background: var(--vs-bg-2, #252526);
    border: 1px solid var(--vs-border, #3e3e42);
    border-radius: 6px;
    width: 96vw;
    height: 92vh;
    max-width: 1400px;
    max-height: 94vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.6);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: var(--vs-bg-3, #2d2d30);
    border-bottom: 1px solid var(--vs-border, #3e3e42);
  }

  .modal-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--vs-text-bright, #f3f3f3);
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--vs-text-dim, #9d9d9d);
    font-size: 16px;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 3px;
  }

  .btn-close:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
  }

  .modal-body {
    padding: 16px;
    overflow-y: auto;
  }
</style>
