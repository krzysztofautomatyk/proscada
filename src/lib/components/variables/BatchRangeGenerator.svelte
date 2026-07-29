<script lang="ts">
  import type { TagDefinition } from "$lib/types";
  import type { DevicePollQuery } from "$lib/types/registerMap";

  interface Props {
    selectedQuery?: DevicePollQuery | null;
    existingTags: TagDefinition[];
    onGenerate: (tags: TagDefinition[]) => void;
    onCancel?: () => void;
  }

  let {
    selectedQuery = null,
    existingTags,
    onGenerate,
    onCancel,
  }: Props = $props();

  let batchPrefix = $state("Sensor_");
  let batchDataType = $state<TagDefinition["data_type"]>("u16");
  let batchStartAddr = $state(100);
  let batchCount = $state(10);
  let batchStep = $state(1);
  let batchUnit = $state("bar");
  let batchScale = $state(1);
  let batchOffset = $state(0);
  let batchDecimals = $state(2);
  let batchReadonly = $state(false);

  let errorMessage = $state("");

  $effect(() => {
    if (selectedQuery) {
      batchStartAddr = selectedQuery.startAddress;
      batchCount = Math.min(10, selectedQuery.count);
    }
  });

  function handleBatchTypeChange() {
    if (batchDataType === "f32") {
      batchStep = 2;
      batchDecimals = 2;
    } else {
      batchStep = 1;
      batchDecimals = 0;
    }
  }

  const generatedBatchPreview = $derived.by(() => {
    if (!selectedQuery) return [];

    const list: TagDefinition[] = [];
    let currentAddr = Number(batchStartAddr) || 0;
    const count = Math.min(Math.max(1, Number(batchCount) || 1), 100);
    const step = Math.max(1, Number(batchStep) || 1);

    for (let i = 1; i <= count; i++) {
      const tName = `${batchPrefix}${i}`;
      const tId = `${tName.toLowerCase().replace(/[^a-z0-9_.]/g, "_")}`;
      list.push({
        id: tId,
        name: tName,
        device_id: selectedQuery.deviceId,
        data_type: batchDataType,
        binding: {
          address: currentAddr,
          bit: batchDataType === "bool" && selectedQuery.table === "holding" ? 0 : null,
          table: selectedQuery.table,
          writable: selectedQuery.table === "input" || selectedQuery.table === "discrete" ? false : !batchReadonly,
        },
        unit: batchUnit,
        description: `Wygenerowano w serii w polu zapytania ${selectedQuery.name} (R${currentAddr})`,
        scale: Number(batchScale) || 1,
        offset: Number(batchOffset) || 0,
        decimals: Number(batchDecimals) || 0,
      });
      currentAddr += step;
    }
    return list;
  });

  function handleSubmit() {
    errorMessage = "";
    if (generatedBatchPreview.length === 0) {
      errorMessage = "Brak zmiennych do wygenerowania.";
      return;
    }

    const existingIds = new Set(existingTags.map((t) => t.id));
    const duplicates = generatedBatchPreview.filter((t) => existingIds.has(t.id));
    if (duplicates.length > 0) {
      errorMessage = `Wykryto powtórzenie ID w projekcie: ${duplicates[0].id}. Zmień prefiks.`;
      return;
    }

    onGenerate(generatedBatchPreview);
  }
</script>

<div class="batch-container">
  {#if errorMessage}
    <div class="error-banner">⚠️ {errorMessage}</div>
  {/if}

  <div class="batch-config-panel">
    <h4 class="sub-title">1. Parametry Serii w Polu Zapytania</h4>
    <div class="form-grid">
      <div class="form-group">
        <label for="b-prefix">Prefiks Nazwy (Name Prefix):</label>
        <input id="b-prefix" type="text" bind:value={batchPrefix} placeholder="Sensor_" required />
      </div>

      <div class="form-group">
        <label for="b-type">Typ Danych:</label>
        <select id="b-type" bind:value={batchDataType} onchange={handleBatchTypeChange}>
          <option value="u16">u16 (Word 16-bit)</option>
          <option value="i16">i16 (Signed Int 16-bit)</option>
          <option value="f32">f32 (Float 32-bit - Krok 2 adresy)</option>
          <option value="bool">bool (Bit)</option>
        </select>
      </div>

      <div class="form-group">
        <label for="b-start">Początkowy Adres Rejestru:</label>
        <input id="b-start" type="number" min="0" max="65535" bind:value={batchStartAddr} required />
      </div>

      <div class="form-group">
        <label for="b-count">Liczba Zmiennych:</label>
        <input id="b-count" type="number" min="1" max="100" bind:value={batchCount} required />
      </div>

      <div class="form-group">
        <label for="b-step">Krok Adresowania (Addr Step):</label>
        <input id="b-step" type="number" min="1" max="10" bind:value={batchStep} required />
      </div>

      <div class="form-group">
        <label for="b-unit">Domyślna Jednostka:</label>
        <input id="b-unit" type="text" bind:value={batchUnit} placeholder="bar, °C, %" />
      </div>

      <div class="form-group full-width checkbox-group">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={batchReadonly} />
          <span>🔒 Oznacz całą serię jako <strong>Read-Only (Tylko Odczyt)</strong></span>
        </label>
      </div>
    </div>
  </div>

  <div class="preview-panel">
    <h4 class="sub-title">2. Podgląd Wygenerowanych Zmiennych ({generatedBatchPreview.length})</h4>
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Lp</th>
            <th>Nazwa</th>
            <th>Tag ID</th>
            <th>Typ</th>
            <th>Dostęp</th>
            <th>Adres</th>
            <th>Jednostka</th>
          </tr>
        </thead>
        <tbody>
          {#each generatedBatchPreview as g, idx}
            <tr>
              <td>{idx + 1}</td>
              <td class="bold">{g.name}</td>
              <td class="mono">{g.id}</td>
              <td><span class="badge {g.data_type}">{g.data_type}</span></td>
              <td>
                <span class="rw-tag" class:readonly={!g.binding.writable}>
                  {g.binding.writable ? "R/W" : "READONLY"}
                </span>
              </td>
              <td class="mono">R{g.binding.address}</td>
              <td>{g.unit || "—"}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>

  <div class="actions">
    {#if onCancel}
      <button type="button" class="btn-cancel" onclick={onCancel}>Anuluj</button>
    {/if}
    <button type="button" class="btn-primary" onclick={handleSubmit}>
      🚀 Wygeneruj {generatedBatchPreview.length} Zmiennych
    </button>
  </div>
</div>

<style>
  .batch-container { display: flex; flex-direction: column; gap: 14px; }

  .error-banner {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid #ef4444;
    color: #fca5a5;
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 12px;
  }

  .sub-title {
    margin: 0 0 8px;
    font-size: 11.5px;
    font-weight: 700;
    color: #4ade80;
    text-transform: uppercase;
  }

  .batch-config-panel, .preview-panel {
    background: #141418;
    border: 1px solid #282d37;
    border-radius: 6px;
    padding: 12px;
  }

  .form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px 14px; }
  .full-width { grid-column: span 2; }

  .form-group { display: flex; flex-direction: column; gap: 3px; }
  label { font-size: 11.5px; font-weight: 600; color: #cbd5e1; }

  input, select {
    background: var(--gh-canvas-inset, #010409);
    border: 1px solid var(--gh-border-default, #30363d);
    border-radius: 4px;
    color: var(--gh-fg-default, #e6edf3);
    padding: 6px 10px;
    font-size: 12px;
    outline: none;
  }

  .checkbox-group { flex-direction: row; align-items: center; margin-top: 4px; }
  .checkbox-label { display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 12px; }
  .checkbox-label input { width: 16px; height: 16px; accent-color: var(--copilot-purple, #8957e5); }

  .table-wrap {
    max-height: 180px;
    overflow-y: auto;
    border: 1px solid var(--gh-border-default, #30363d);
    border-radius: 4px;
  }

  table { width: 100%; border-collapse: collapse; font-size: 11px; text-align: left; }
  th { background: var(--gh-canvas-default, #0d1117); color: var(--gh-fg-muted, #848d97); padding: 6px 10px; border-bottom: 1px solid var(--gh-border-default, #30363d); font-weight: 600; }
  td { padding: 5px 10px; border-bottom: 1px solid var(--gh-border-muted, #21262d); color: var(--gh-fg-default, #e6edf3); }
  tr:nth-child(even) { background: var(--gh-canvas-subtle, #161b22); }

  .mono { font-family: var(--font-mono, monospace); }
  .bold { font-weight: 600; color: var(--vs-text-bright, #f0f6fc); }

  .badge {
    padding: 2px 6px; border-radius: 3px; font-size: 10px; font-weight: 600; background: var(--gh-border-muted, #21262d); color: var(--gh-fg-default, #e6edf3);
  }
  .badge.bool { background: rgba(57, 197, 207, 0.2); color: var(--copilot-cyan, #39c5cf); }
  .badge.u16, .badge.i16 { background: rgba(35, 134, 54, 0.2); color: #4ade80; }
  .badge.f32 { background: rgba(163, 113, 247, 0.2); color: var(--copilot-purple-light, #a371f7); }

  .rw-tag { font-size: 10px; font-weight: 700; color: #4ade80; }
  .rw-tag.readonly { color: var(--gh-attention-fg, #d29922); }

  .actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 4px; }

  .btn-cancel {
    background: var(--gh-border-muted, #21262d); border: 1px solid var(--gh-border-default, #30363d); color: var(--gh-fg-default, #e6edf3);
    padding: 7px 14px; border-radius: 5px; font-size: 12px; cursor: pointer;
  }
  .btn-submit {
    background: var(--copilot-gradient); border: 1px solid rgba(255, 255, 255, 0.2); color: #fff;
    padding: 7px 16px; border-radius: 5px; font-size: 12px; font-weight: 600; cursor: pointer;
    box-shadow: 0 0 12px rgba(163, 113, 247, 0.35);
  }
  .btn-submit:hover:not(:disabled) {
    background: linear-gradient(135deg, #b78af7 0%, #388bfd 100%);
    box-shadow: 0 0 16px rgba(163, 113, 247, 0.5);
  }
  .btn-submit:disabled { opacity: 0.4; cursor: not-allowed; box-shadow: none; }
</style>
