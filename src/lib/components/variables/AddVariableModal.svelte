<script lang="ts">
  import type { TagDefinition } from "$lib/types";
  import { project, addTagToProject, addTagsToProject } from "$lib/stores/app";
  import {
    extractDevicePollQueries,
    buildRegisterMap,
    setTagReadonly,
  } from "$lib/services/registerMapService";
  import DeviceQuerySelector from "./DeviceQuerySelector.svelte";
  import RegisterMapView from "./RegisterMapView.svelte";
  import SingleVariableForm from "./SingleVariableForm.svelte";
  import BatchRangeGenerator from "./BatchRangeGenerator.svelte";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  type TabMode = "map" | "single" | "batch";
  let activeTab = $state<TabMode>("map");

  const devices = $derived($project?.devices ?? []);
  const existingTags = $derived($project?.tags ?? []);

  let selectedDeviceId = $state<string>("");
  let selectedQueryId = $state<string>("");
  let editingTag = $state<TagDefinition | null>(null);
  let initialFormAddress = $state<number>(0);

  // Computed queries for selected device
  const availableQueries = $derived.by(() => {
    const dev = devices.find((d) => d.id === selectedDeviceId);
    return extractDevicePollQueries(dev);
  });

  const activeQuery = $derived.by(() => {
    return availableQueries.find((q) => q.id === selectedQueryId) || availableQueries[0] || null;
  });

  const registerMapEntries = $derived.by(() => {
    if (!activeQuery) return [];
    return buildRegisterMap(activeQuery, existingTags);
  });

  $effect(() => {
    if (open) {
      if (devices.length > 0 && !selectedDeviceId) {
        selectedDeviceId = devices[0].id;
      }
    }
  });

  $effect(() => {
    if (selectedDeviceId && availableQueries.length > 0) {
      if (!availableQueries.some((q) => q.id === selectedQueryId)) {
        selectedQueryId = availableQueries[0].id;
      }
    }
  });

  function handleSelectDevice(id: string) {
    selectedDeviceId = id;
    const queries = extractDevicePollQueries(devices.find((d) => d.id === id));
    if (queries.length > 0) {
      selectedQueryId = queries[0].id;
    }
  }

  function handleSelectQuery(id: string) {
    selectedQueryId = id;
  }

  function handleToggleReadonly(tagId: string, currentReadonly: boolean) {
    const targetTag = existingTags.find((t) => t.id === tagId);
    if (!targetTag) return;
    const updatedTag = setTagReadonly(targetTag, !currentReadonly);
    addTagToProject(updatedTag);
  }

  function handleSelectRegisterForEdit(address: number, tagId?: string) {
    const found = tagId ? existingTags.find((t) => t.id === tagId) : null;
    editingTag = found || null;
    initialFormAddress = address;
    activeTab = "single";
  }

  function handleAddTagAtAddress(address: number) {
    editingTag = null;
    initialFormAddress = address;
    activeTab = "single";
  }

  function handleEditBitTag(address: number, bitIndex: number) {
    editingTag = null;
    initialFormAddress = address;
    activeTab = "single";
  }

  function handleSaveSingleTag(tag: TagDefinition) {
    addTagToProject(tag);
    editingTag = null;
    activeTab = "map";
  }

  function handleBatchGenerate(tags: TagDefinition[]) {
    addTagsToProject(tags);
    activeTab = "map";
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={(event) => open && handleKeyDown(event)} />

{#if open}
  <button type="button" class="backdrop" aria-label="Zamknij okno zmiennej" onclick={onClose}></button>
  <div class="modal" role="dialog" aria-labelledby="add-var-title" aria-modal="true">
    <!-- Modal Header -->
    <div class="modal-header">
      <div class="title-wrap">
        <span class="header-icon">🏷️</span>
        <div>
          <h3 id="add-var-title">Konfigurator Zmiennych i Mapy Rejestrów SCADA</h3>
          <p class="subtitle">
            Wybierz sterownik PLC oraz pole zapytania Modbus, aby zdefiniować mapę rejestrów z opcją Read-Only
          </p>
        </div>
      </div>
      <button type="button" class="close-btn" onclick={onClose} title="Zamknij (Esc)">✕</button>
    </div>

    <!-- Device & Query Selection Header Bar -->
    <div class="selector-container">
      <DeviceQuerySelector
        {devices}
        {selectedDeviceId}
        {selectedQueryId}
        queries={availableQueries}
        onSelectDevice={handleSelectDevice}
        onSelectQuery={handleSelectQuery}
      />
    </div>

    <!-- Main Navigation Tab Bar -->
    <div class="tab-bar">
      <button
        type="button"
        class="tab-btn"
        class:active={activeTab === "map"}
        onclick={() => (activeTab = "map")}
      >
        🗺️ Mapa Rejestrów ({activeQuery ? `R${activeQuery.startAddress}-R${activeQuery.endAddress}` : "—"})
      </button>
      <button
        type="button"
        class="tab-btn"
        class:active={activeTab === "single"}
        onclick={() => {
          editingTag = null;
          activeTab = "single";
        }}
      >
        🔹 Pojedynczy Tag / Zmienna
      </button>
      <button
        type="button"
        class="tab-btn"
        class:active={activeTab === "batch"}
        onclick={() => (activeTab = "batch")}
      >
        🔢 Generator Serii Rejestrów
      </button>
    </div>

    <!-- Modal Body Content -->
    <div class="modal-body">
      {#if activeTab === "map"}
        <RegisterMapView
          entries={registerMapEntries}
          onToggleReadonly={handleToggleReadonly}
          onSelectRegisterForEdit={handleSelectRegisterForEdit}
          onAddTagAtAddress={handleAddTagAtAddress}
          onEditBitTag={handleEditBitTag}
        />
      {:else if activeTab === "single"}
        <SingleVariableForm
          {devices}
          {existingTags}
          initialQuery={activeQuery}
          initialAddress={initialFormAddress}
          {editingTag}
          onSave={handleSaveSingleTag}
          onCancel={() => (activeTab = "map")}
        />
      {:else if activeTab === "batch"}
        <BatchRangeGenerator
          selectedQuery={activeQuery}
          {existingTags}
          onGenerate={handleBatchGenerate}
          onCancel={() => (activeTab = "map")}
        />
      {/if}
    </div>

    <!-- Modal Footer -->
    <div class="modal-footer">
      <div class="hint">
        🔒 Wskazówka: Oznaczenie zmiennej jako <strong>Read-Only</strong> gwarantuje, że silnik Runtime nie wykona zapisu procesowego.
      </div>
      <button type="button" class="btn-close-footer" onclick={onClose}>Zamknij</button>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    /* Rendered as a <button> so the dismissal affordance is keyboard-reachable. */
    appearance: none;
    border: 0;
    padding: 0;
    margin: 0;
    font: inherit;
    color: inherit;
    cursor: default;
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.72);
    backdrop-filter: blur(4px);
    z-index: 9998;
    animation: fade-in 0.15s ease-out;
  }

  .modal-card {
    background: var(--gh-canvas-overlay, #161b22);
    border: 1px solid var(--gh-border-default, #30363d);
    border-radius: 10px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.7), 0 0 0 1px rgba(163, 113, 247, 0.2);
    width: 95vw;
    max-width: 1020px;
    max-height: 95vh;
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 9999;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    font-family: var(--font-ui);
    animation: modal-pop 0.15s ease-out;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 18px;
    background: var(--gh-canvas-default, #0d1117);
    border-bottom: 1px solid var(--gh-border-default, #30363d);
  }

  .title-wrap { display: flex; align-items: center; gap: 12px; }
  .header-icon { font-size: 24px; }
  .title-wrap h3 { margin: 0; font-size: 15px; font-weight: 700; color: var(--vs-text-bright, #f0f6fc); }
  .subtitle { margin: 2px 0 0; font-size: 11px; color: var(--gh-fg-muted, #848d97); }

  .close-btn {
    background: transparent; border: none; color: var(--gh-fg-muted, #848d97); font-size: 18px;
    cursor: pointer; padding: 4px 8px; border-radius: 4px;
  }
  .close-btn:hover { background: var(--gh-danger-emphasis, #da3633); color: #fff; }

  .selector-container {
    padding: 10px 18px 4px 18px;
    background: var(--gh-canvas-default, #0d1117);
  }

  .tab-bar {
    display: flex; background: var(--gh-canvas-default, #0d1117); border-bottom: 1px solid var(--gh-border-default, #30363d);
    padding: 0 18px;
  }

  .tab-btn {
    flex: 1; padding: 10px 14px; background: transparent; border: none;
    border-bottom: 2px solid transparent; color: var(--gh-fg-muted, #848d97); font-size: 12px;
    font-weight: 600; cursor: pointer; transition: all 0.12s;
  }

  .tab-btn.active {
    color: var(--copilot-purple-light, #a371f7); border-bottom-color: var(--copilot-purple, #8957e5); background: var(--gh-canvas-inset, #010409);
  }

  .modal-body {
    padding: 14px 18px; overflow-y: auto; display: flex; flex-direction: column;
    gap: 14px; flex: 1; max-height: calc(95vh - 160px);
  }

  .modal-footer {
    padding: 10px 18px; background: var(--gh-canvas-default, #0d1117); border-top: 1px solid var(--gh-border-default, #30363d);
    display: flex; align-items: center; justify-content: space-between;
  }

  .hint { font-size: 11px; color: var(--gh-fg-muted, #848d97); }

  .btn-close-footer {
    background: var(--gh-border-muted, #21262d); border: 1px solid var(--gh-border-default, #30363d); color: var(--gh-fg-default, #e6edf3);
    padding: 6px 16px; border-radius: 4px; font-size: 12px; cursor: pointer;
  }

  @keyframes fade-in { from { opacity: 0; } to { opacity: 1; } }
  @keyframes modal-pop {
    from { opacity: 0; transform: translate(-50%, -48%) scale(0.96); }
    to { opacity: 1; transform: translate(-50%, -50%) scale(1); }
  }
</style>
