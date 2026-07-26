<script lang="ts">
  import type { TagDefinition, TagBinding, DeviceConfig } from "$lib/types";
  import type { DevicePollQuery } from "$lib/types/registerMap";
  import { validateRegisterTag } from "$lib/services/registerMapService";
  import { uid } from "$lib/utils/projectTree";

  interface Props {
    devices: DeviceConfig[];
    existingTags: TagDefinition[];
    initialQuery?: DevicePollQuery | null;
    initialAddress?: number;
    editingTag?: TagDefinition | null;
    onSave: (tag: TagDefinition) => void;
    onCancel?: () => void;
  }

  let {
    devices,
    existingTags,
    initialQuery,
    initialAddress = 0,
    editingTag = null,
    onSave,
    onCancel,
  }: Props = $props();

  // Form State
  let id = $state(uid("tag"));
  let name = $state("");
  let device_id = $state("");
  let data_type = $state<TagDefinition["data_type"]>("u16");
  let table = $state<TagBinding["table"]>("holding");
  let address = $state(0);
  let bit = $state<number | null>(null);

  // Read-only switch logic (User requirement: "przy definificji zmiennej moge zazznaczyc readonly aby aplikacja nie mogla zapisywac")
  let readonly = $state(false);

  let bit_write_mode = $state<"mask_write" | "read_modify_write">("mask_write");
  let single_writer = $state(false);
  let verify_readback = $state(true);

  let unit = $state("");
  let scale = $state(1);
  let offset = $state(0);
  let decimals = $state(0);
  let description = $state("");

  let errorMessage = $state("");
  let warningMessage = $state("");

  const unitPresets = ["", "bar", "°C", "m³/h", "Hz", "%", "kW", "RPM", "V", "A", "m", "m/s", "l/min", "kg/h", "Pa"];

  $effect(() => {
    if (editingTag) {
      id = editingTag.id;
      name = editingTag.name;
      device_id = editingTag.device_id;
      data_type = editingTag.data_type;
      table = editingTag.binding.table;
      address = editingTag.binding.address;
      bit = editingTag.binding.bit ?? null;
      readonly = !editingTag.binding.writable;
      bit_write_mode = editingTag.binding.bit_write_mode ?? "mask_write";
      single_writer = editingTag.binding.single_writer ?? false;
      verify_readback = editingTag.binding.verify_readback ?? true;
      unit = editingTag.unit ?? "";
      scale = editingTag.scale ?? 1;
      offset = editingTag.offset ?? 0;
      decimals = editingTag.decimals ?? 0;
      description = editingTag.description ?? "";
    } else {
      id = uid("tag");
      name = "";
      device_id = initialQuery?.deviceId ?? devices[0]?.id ?? "";
      data_type = "u16";
      table = initialQuery?.table ?? "holding";
      address = initialAddress || initialQuery?.startAddress || 0;
      bit = null;
      readonly = false;
      bit_write_mode = "mask_write";
      single_writer = false;
      verify_readback = true;
      unit = "";
      scale = 1;
      offset = 0;
      decimals = 0;
      description = "";
    }
  });

  function handleNameChange() {
    if (!editingTag && (!id || id.startsWith("tag_"))) {
      const slug = name
        .trim()
        .toLowerCase()
        .replace(/[^a-z0-9_.]/g, "_");
      if (slug) id = slug;
    }
  }

  function handleDataTypeChange() {
    if (data_type === "f32") {
      decimals = 2;
      bit = null;
    } else if (data_type === "bool") {
      decimals = 0;
      if (table === "holding" && bit === null) bit = 0;
    } else {
      decimals = 0;
      bit = null;
    }
  }

  function handleSubmit(e?: Event) {
    if (e) e.preventDefault();
    errorMessage = "";
    warningMessage = "";

    const tagCandidate: TagDefinition = {
      id: id.trim(),
      name: name.trim(),
      device_id,
      data_type,
      binding: {
        address: Number(address),
        bit: data_type === "bool" && table === "holding" ? (bit !== null ? Number(bit) : null) : null,
        table,
        writable: table === "input" || table === "discrete" ? false : !readonly,
        bit_write_mode: table === "holding" && bit !== null ? bit_write_mode : undefined,
        single_writer: bit_write_mode === "read_modify_write" ? single_writer : undefined,
        verify_readback: !readonly ? verify_readback : undefined,
      },
      unit: unit.trim(),
      description: description.trim(),
      scale: Number(scale) || 1,
      offset: Number(offset) || 0,
      decimals: Number(decimals) || 0,
    };

    const valRes = validateRegisterTag(tagCandidate, editingTag ? [] : existingTags, initialQuery);
    if (!valRes.valid) {
      errorMessage = valRes.errors.join(" ");
      return;
    }

    if (valRes.warnings.length > 0) {
      warningMessage = valRes.warnings.join(" ");
    }

    onSave(tagCandidate);
  }
</script>

<form onsubmit={handleSubmit} class="form-container">
  {#if errorMessage}
    <div class="banner error-banner">⚠️ {errorMessage}</div>
  {/if}
  {#if warningMessage}
    <div class="banner warning-banner">ℹ️ {warningMessage}</div>
  {/if}

  <div class="form-grid">
    <!-- Friendly Name -->
    <div class="form-group">
      <label for="f-name">Nazwa Zmiennej (Friendly Name):</label>
      <input
        id="f-name"
        type="text"
        bind:value={name}
        oninput={handleNameChange}
        placeholder="np. Ciśnienie Wody P1"
        required
      />
    </div>

    <!-- Tag ID -->
    <div class="form-group">
      <label for="f-id">Identyfikator Tagu (Tag ID):</label>
      <input
        id="f-id"
        type="text"
        bind:value={id}
        placeholder="np. PUMP01.Pressure"
        disabled={!!editingTag}
        required
      />
    </div>

    <!-- PLC Device -->
    <div class="form-group">
      <label for="f-device">Urządzenie PLC (Device):</label>
      <select id="f-device" bind:value={device_id} required>
        {#each devices as d}
          <option value={d.id}>{d.name} ({d.host}:{d.port})</option>
        {/each}
      </select>
    </div>

    <!-- Data Type -->
    <div class="form-group">
      <label for="f-type">Typ Danych (Data Type):</label>
      <select id="f-type" bind:value={data_type} onchange={handleDataTypeChange}>
        <option value="u16">u16 (Unsigned Int 16-bit)</option>
        <option value="i16">i16 (Signed Int 16-bit)</option>
        <option value="f32">f32 (Float 32-bit IEEE 754 - 2 rejestry)</option>
        <option value="bool">bool (Logiczny - 1 bit)</option>
      </select>
    </div>

    <!-- Register Table -->
    <div class="form-group">
      <label for="f-table">Tabela Modbus (Register Table):</label>
      <select id="f-table" bind:value={table} onchange={handleDataTypeChange}>
        <option value="holding">Holding Register (4x)</option>
        <option value="input">Input Register (3x - Tylko Odczyt)</option>
        <option value="coil">Coil (0x)</option>
        <option value="discrete">Discrete Input (1x - Tylko Odczyt)</option>
      </select>
    </div>

    <!-- Modbus Address -->
    <div class="form-group">
      <label for="f-addr">Adres Rejestru Modbus (0..65535):</label>
      <input id="f-addr" type="number" min="0" max="65535" bind:value={address} required />
    </div>

    <!-- Bit Index (if bool on holding) -->
    {#if data_type === "bool" && table === "holding"}
      <div class="form-group">
        <label for="f-bit">Indeks Bitu (Bit Index 0..15):</label>
        <input id="f-bit" type="number" min="0" max="15" bind:value={bit} required />
        <small class="hint">LSB = Bit 0, MSB = Bit 15</small>
      </div>
    {/if}

    <!-- READONLY TOGGLE (PROMINENT REQUIREMENT) -->
    {#if table === "holding" || table === "coil"}
      <div class="form-group full-width readonly-box">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={readonly} />
          <span class="readonly-text">
            🔒 <strong>Zmienna Tylko do Odczytu (Read-Only)</strong> — Zaznacz, aby zabronić aplikacji zapisu do sterownika
          </span>
        </label>
      </div>

      {#if !readonly && table === "holding" && bit !== null}
        <div class="form-group">
          <label for="f-writemode">Tryb Zapisu Bitowego Modbus:</label>
          <select id="f-writemode" bind:value={bit_write_mode}>
            <option value="mask_write">FC22 Maska Bitowa (Preferowane FC22)</option>
            <option value="read_modify_write">FC03 + FC06 RMW (Read-Modify-Write)</option>
          </select>
        </div>

        {#if bit_write_mode === "read_modify_write"}
          <div class="form-group checkbox-group">
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={single_writer} />
              <span>Confirm Single Writer (Wymagane dla bezpiecznego RMW)</span>
            </label>
          </div>
        {/if}
      {/if}
    {/if}

    <!-- Unit & Scaling -->
    <div class="form-group">
      <label for="f-unit">Jednostka Inżynieryjna (Unit):</label>
      <div class="group-input-wrap">
        <input id="f-unit" type="text" bind:value={unit} placeholder="e.g. bar, °C, %" />
        <select onchange={(e) => (unit = e.currentTarget.value)}>
          <option value="">Szablon…</option>
          {#each unitPresets as u}
            {#if u}<option value={u}>{u}</option>{/if}
          {/each}
        </select>
      </div>
    </div>

    <div class="form-group">
      <label for="f-scale">Mnożnik Skali (Scale Factor):</label>
      <input id="f-scale" type="number" step="any" bind:value={scale} required />
    </div>

    <div class="form-group">
      <label for="f-offset">Przesunięcie (Offset):</label>
      <input id="f-offset" type="number" step="any" bind:value={offset} />
    </div>

    <div class="form-group">
      <label for="f-decimals">Miejsca Po Przecinku (Decimals):</label>
      <input id="f-decimals" type="number" min="0" max="6" bind:value={decimals} />
    </div>

    <!-- Description -->
    <div class="form-group full-width">
      <label for="f-desc">Opis Zmiennej (Description):</label>
      <input
        id="f-desc"
        type="text"
        bind:value={description}
        placeholder="np. Czujnik ciśnienia na wyjściu pompy głównej"
      />
    </div>
  </div>

  <div class="form-actions">
    {#if onCancel}
      <button type="button" class="btn-cancel" onclick={onCancel}>Anuluj</button>
    {/if}
    <button type="submit" class="btn-submit">
      💾 {editingTag ? "Zapisz Zmiany" : "Dodaj Zmienną"}
    </button>
  </div>
</form>

<style>
  .form-container {
    display: flex;
    flex-direction: column;
    gap: 14px;
    background: #141418;
    border: 1px solid #282d37;
    border-radius: 6px;
    padding: 16px;
  }

  .banner {
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 12px;
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid #ef4444;
    color: #fca5a5;
  }

  .warning-banner {
    background: rgba(245, 158, 11, 0.15);
    border: 1px solid #f59e0b;
    color: #fef08a;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px 16px;
  }

  .full-width { grid-column: span 2; }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  label {
    font-size: 12px;
    font-weight: 600;
    color: #cbd5e1;
  }

  input, select {
    background: #101014;
    border: 1px solid #334155;
    border-radius: 4px;
    color: #f8fafc;
    padding: 7px 10px;
    font-size: 12px;
    outline: none;
  }

  input:focus, select:focus {
    border-color: #22c55e;
    box-shadow: 0 0 0 2px rgba(34, 197, 94, 0.2);
  }

  .hint { font-size: 10px; color: #64748b; }

  .group-input-wrap { display: flex; gap: 6px; }
  .group-input-wrap input { flex: 1; }
  .group-input-wrap select { width: 120px; }

  .readonly-box {
    background: #1b1b24;
    border: 1px solid #3b82f6;
    border-radius: 6px;
    padding: 10px 12px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    font-size: 12px;
  }

  .checkbox-label input {
    width: 18px;
    height: 18px;
    accent-color: #f59e0b;
  }

  .readonly-text { color: #f8fafc; }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 8px;
  }

  .btn-cancel {
    background: #27272a;
    border: 1px solid #3f3f46;
    color: #cbd5e1;
    padding: 8px 16px;
    border-radius: 5px;
    font-size: 12px;
    cursor: pointer;
  }

  .btn-submit {
    background: #16a34a;
    border: 1px solid #22c55e;
    color: #fff;
    padding: 8px 20px;
    border-radius: 5px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-submit:hover {
    background: #15803d;
    box-shadow: 0 0 12px rgba(34, 197, 94, 0.4);
  }
</style>
