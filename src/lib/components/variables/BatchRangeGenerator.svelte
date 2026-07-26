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
    background: #0f0f13;
    border: 1px solid #334155;
    border-radius: 4px;
    color: #f8fafc;
    padding: 6px 10px;
    font-size: 12px;
    outline: none;
  }

  .checkbox-group { flex-direction: row; align-items: center; margin-top: 4px; }
  .checkbox-label { display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 12px; }
  .checkbox-label input { width: 16px; height: 16px; accent-color: #f59e0b; }

  .table-wrap {
    max-height: 180px;
    overflow-y: auto;
    border: 1px solid #2d3748;
    border-radius: 4px;
  }

  table { width: 100%; border-collapse: collapse; font-size: 11px; text-align: left; }
  th { background: #1e1e26; color: #94a3b8; padding: 6px 10px; border-bottom: 1px solid #334155; font-weight: 600; }
  td { padding: 5px 10px; border-bottom: 1px solid #272730; color: #cbd5e1; }
  tr:nth-child(even) { background: #16161c; }

  .mono { font-family: monospace; }
  .bold { font-weight: 600; color: #fff; }

  .badge {
    padding: 2px 6px; border-radius: 3px; font-size: 10px; font-weight: 600; background: #334155; color: #fff;
  }
  .badge.bool { background: #0284c7; }
  .badge.u16, .badge.i16 { background: #16a34a; }
  .badge.f32 { background: #9333ea; }

  .rw-tag { font-size: 10px; font-weight: 700; color: #22c55e; }
  .rw-tag.readonly { color: #f59e0b; }

  .actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 4px; }

  .btn-cancel {
    background: #27272a; border: 1px solid #3f3f46; color: #cbd5e1;
    padding: 7px 14px; border-radius: 5px; font-size: 12px; cursor: pointer;
  }

  .btn-primary {
    background: #16a34a; border: 1px solid #22c55e; color: #fff;
    padding: 7px 16px; border-radius: 5px; font-size: 12px; font-weight: 600; cursor: pointer;
  }

  .btn-primary:hover { background: #15803d; }
</style>
