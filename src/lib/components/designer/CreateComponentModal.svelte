<script lang="ts">
  import type { FormDef, ScadaProject, WidgetDef } from "$lib/types";
  import {
    project,
    createComponentTemplateFromSelection,
    selectedWidgetsForTemplate,
    extractTagSlotsFromWidgets,
    fieldLabel,
  } from "$lib/stores/app";

  interface Props {
    open: boolean;
    onClose: () => void;
    onCreated?: (templateId: string) => void;
  }

  let { open = $bindable(false), onClose, onCreated }: Props = $props();

  const currentProject = $derived($project as ScadaProject | null);
  const existingTemplates = $derived(currentProject?.component_templates ?? []);

  let snapshotSelection = $state<{ form: FormDef; widgets: WidgetDef[] } | null>(null);

  const selectionInfo = $derived(snapshotSelection || (open ? selectedWidgetsForTemplate() : null));

  const extractedSlots = $derived.by(() => {
    if (!selectionInfo) return [];
    return extractTagSlotsFromWidgets(
      selectionInfo.widgets,
      undefined,
      currentProject?.tags,
    );
  });

  let componentName = $state("Nowy Komponent");
  let componentCategory = $state("Custom");
  let componentVersion = $state("1.0.0");
  let componentDescription = $state("");

  // Snapshot selection & reset fields when modal opens
  $effect(() => {
    if (open) {
      const sel = selectedWidgetsForTemplate();
      snapshotSelection = sel;
      const count = sel?.widgets.length ?? 0;
      componentName = count > 0 ? `Komponent (${count} kontrolek)` : "Nowy Komponent";
      componentCategory = "Custom";
      componentVersion = "1.0.0";
      componentDescription = "";
    } else {
      snapshotSelection = null;
    }
  });

  // Validation
  const trimmedName = $derived(componentName.trim());

  const nameExists = $derived.by(() => {
    if (!trimmedName) return false;
    return existingTemplates.some(
      (t) => t.name.toLowerCase() === trimmedName.toLowerCase(),
    );
  });

  const widgetCount = $derived(selectionInfo?.widgets.length ?? 0);
  const isValid = $derived(widgetCount > 0 && !!trimmedName && !nameExists);

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!isValid) return;

    const id = createComponentTemplateFromSelection(
      trimmedName,
      componentCategory,
      componentVersion,
      componentDescription,
    );

    if (id) {
      onCreated?.(id);
      onClose();
    }
  }

  function slotVarName(slotKey: string): string {
    return slotKey.replace(/^\{[^}]+\}/, "").replace(/^_/, "") || slotKey;
  }
</script>

{#if open}
  <div class="backdrop" role="dialog" aria-modal="true">
    <button type="button" class="backdrop-dismiss" aria-label="Zamknij okno" onclick={onClose}></button>
    <form class="panel" onsubmit={handleSubmit}>

      <!-- HEADER -->
      <div class="header">
        <span class="hdr-icon">🧩</span>
        <div class="hdr-text">
          <h2>Zapisz jako komponent</h2>
          <p>Zapisujesz zaznaczone kontrolki jako wielokrotnego użytku szablon komponentu SCADA.</p>
        </div>
        <button type="button" class="btn-x" onclick={onClose} title="Zamknij">✕</button>
      </div>

      <!-- BODY -->
      <div class="body">
        <!-- SELECTION INFO SUMMARY BOX -->
        <div class="info-box">
          {#if widgetCount === 0}
            <div class="info-empty">
              ⚠️ Brak zaznaczonych kontrolek na ekranie. Zaznacz przynajmniej jedną kontrolkę, aby utworzyć komponent.
            </div>
          {:else}
            <div class="info-summary">
              <span class="summary-badge">✓ Zaznaczono {widgetCount} kontrolek na ekranie <strong>{selectionInfo?.form.name}</strong></span>
            </div>

            {#if extractedSlots.length > 0}
              <div class="slots-preview">
                <span class="preview-label">Wykryte sloty zmiennych ({extractedSlots.length}):</span>
                <div class="slots-chips">
                  {#each extractedSlots as slot (slot.id)}
                    <span class="slot-chip" title="{slot.widgetLabel} · {fieldLabel(slot.field)}">
                      <strong class="chip-name">{slot.name || slotVarName(slot.slotKey)}</strong>
                      <span class="chip-field">{fieldLabel(slot.field)}</span>
                      <code class="chip-key">{slot.slotKey}</code>
                    </span>
                  {/each}
                </div>
              </div>
            {/if}
          {/if}
        </div>

        <!-- FORM FIELDS -->
        <div class="form-grid">

          <!-- NAZWA KOMPONENTU (WYMAGANA & UNIKALNA) -->
          <div class="form-group">
            <label for="cmp-name" class="label-req">
              Nazwa komponentu <span class="req-star">*</span>
            </label>
            <input
              id="cmp-name"
              type="text"
              class="input-text"
              class:input-err={nameExists || (!trimmedName && widgetCount > 0)}
              placeholder="Wpisz unikalną nazwę komponentu (np. Pompa Zespołowa)"
              bind:value={componentName}
              required
              autocomplete="off"
            />
            {#if nameExists}
              <div class="err-msg">⚠️ Komponent o nazwie "<strong>{trimmedName}</strong>" już istnieje w projekcie! Wpisz unikalną nazwę.</div>
            {:else if !trimmedName}
              <div class="err-msg">⚠️ Nazwa komponentu jest wymagana.</div>
            {:else}
              <div class="help-msg">Nazwa będzie wyświetlana w drzewie Solucji oraz Toolboxie.</div>
            {/if}
          </div>

          <!-- KATEGORIA & WERSJA -->
          <div class="form-row">
            <div class="form-group flex-1">
              <label for="cmp-cat">Kategoria (Opcjonalnie)</label>
              <input
                id="cmp-cat"
                type="text"
                class="input-text"
                placeholder="np. Custom, Pumps, Valves, Tanks"
                bind:value={componentCategory}
              />
            </div>
            <div class="form-group flex-1">
              <label for="cmp-ver">Wersja (Opcjonalnie)</label>
              <input
                id="cmp-ver"
                type="text"
                class="input-text"
                placeholder="1.0.0"
                bind:value={componentVersion}
              />
            </div>
          </div>

          <!-- OPIS / KOMENTARZ -->
          <div class="form-group">
            <label for="cmp-desc">Opis komponentu (Opcjonalnie)</label>
            <textarea
              id="cmp-desc"
              class="input-textarea"
              rows="3"
              placeholder="Wpisz krótki opis, przeznaczenie i sposób podłączenia komponentu..."
              bind:value={componentDescription}
            ></textarea>
          </div>

        </div>
      </div>

      <!-- FOOTER -->
      <div class="footer">
        <button type="button" class="btn btn-sec" onclick={onClose}>Anuluj</button>
        <button
          type="submit"
          class="btn btn-pri"
          disabled={!isValid}
        >
          🧩 Zapisz jako nowy komponent
        </button>
      </div>

    </form>
  </div>
{/if}

<style>
  .backdrop-dismiss {
    position: absolute;
    inset: 0;
    appearance: none;
    border: 0;
    padding: 0;
    margin: 0;
    background: transparent;
    cursor: default;
  }
  .panel {
    position: relative;
    z-index: 1;
  }
  .backdrop {
    position: fixed; inset: 0; z-index: 9200;
    background: rgba(0,0,0,.7);
    display: flex; align-items: center; justify-content: center;
    padding: 16px;
    backdrop-filter: blur(3px);
  }

  .panel {
    display: flex; flex-direction: column;
    width: 100%; max-width: 680px;
    background: #1a2236; border: 1px solid #2d3f5a;
    border-radius: 10px; overflow: hidden;
    box-shadow: 0 24px 64px rgba(0,0,0,.8);
    color: #e2e8f0;
  }

  /* HEADER */
  .header {
    display: flex; align-items: flex-start; gap: 12px;
    padding: 16px 20px 14px;
    background: #111827; border-bottom: 1px solid #2d3f5a;
    flex-shrink: 0;
  }
  .hdr-icon { font-size: 26px; padding-top: 2px; }
  .hdr-text { flex: 1; min-width: 0; }
  .hdr-text h2 { margin: 0 0 3px; font-size: 18px; font-weight: 600; color: #f1f5f9; }
  .hdr-text p { margin: 0; font-size: 12px; color: #94a3b8; }
  .btn-x {
    background: none; border: none; color: #64748b;
    font-size: 18px; cursor: pointer; padding: 4px 8px;
    border-radius: 4px; transition: all .15s; flex-shrink: 0;
  }
  .btn-x:hover { background: #2d3f5a; color: #f1f5f9; }

  /* BODY */
  .body {
    padding: 20px; display: flex; flex-direction: column; gap: 18px;
    overflow-y: auto; max-height: calc(85vh - 120px);
  }

  /* INFO BOX */
  .info-box {
    background: #0f1623; border: 1px solid #1e2e48;
    border-radius: 8px; padding: 12px 14px;
  }
  .info-empty { color: #f87171; font-size: 13px; text-align: center; }
  .info-summary { font-size: 13px; color: #60a5fa; margin-bottom: 8px; }
  .summary-badge strong { color: #93c5fd; }

  .slots-preview { display: flex; flex-direction: column; gap: 6px; margin-top: 4px; }
  .preview-label { font-size: 11px; color: #64748b; font-weight: 600; text-transform: uppercase; letter-spacing: .04em; }
  .slots-chips { display: flex; flex-wrap: wrap; gap: 6px; }

  .slot-chip {
    display: inline-flex; align-items: center; gap: 6px;
    background: #111d33; border: 1px solid #1e3a5f;
    border-radius: 5px; padding: 3px 8px; font-size: 11px;
  }
  .chip-name { color: #f1f5f9; font-weight: 600; }
  .chip-field { color: #60a5fa; font-size: 10px; background: #0a1220; padding: 1px 4px; border-radius: 3px; }
  .chip-key { font-family: monospace; font-size: 10px; color: #475569; }

  /* FORM GRID */
  .form-grid { display: flex; flex-direction: column; gap: 14px; }
  .form-group { display: flex; flex-direction: column; gap: 5px; }
  .form-row { display: flex; gap: 12px; }
  .flex-1 { flex: 1; }

  label { font-size: 13px; font-weight: 500; color: #cbd5e1; }
  .label-req { font-weight: 600; color: #f1f5f9; }
  .req-star { color: #ef4444; margin-left: 2px; }

  .input-text, .input-textarea {
    background: #0f1623; border: 1px solid #334155;
    border-radius: 6px; padding: 8px 12px; color: #f1f5f9;
    font-size: 13px; outline: none; transition: all .15s;
    width: 100%; box-sizing: border-border;
  }
  .input-text:focus, .input-textarea:focus {
    border-color: #3b82f6; box-shadow: 0 0 0 2px rgba(59,130,246,.18);
  }
  .input-err {
    border-color: #ef4444 !important; background: #261215 !important;
  }

  .input-textarea { resize: vertical; font-family: inherit; }

  .err-msg { font-size: 12px; color: #f87171; margin-top: 2px; }
  .err-msg strong { color: #fca5a5; }
  .help-msg { font-size: 11px; color: #64748b; }

  /* FOOTER */
  .footer {
    display: flex; align-items: center; justify-content: flex-end; gap: 10px;
    padding: 14px 20px; background: #111827; border-top: 1px solid #2d3f5a;
    flex-shrink: 0;
  }

  .btn {
    padding: 8px 20px; border-radius: 6px; font-size: 13px;
    font-weight: 500; border: none; cursor: pointer; transition: all .15s;
  }
  .btn-sec { background: #1e293b; color: #94a3b8; border: 1px solid #334155; }
  .btn-sec:hover { background: #334155; color: #e2e8f0; }
  .btn-pri {
    background: #2563eb; color: #fff; font-weight: 600;
    box-shadow: 0 2px 8px rgba(37,99,235,.3);
  }
  .btn-pri:hover:not(:disabled) { background: #1d4ed8; box-shadow: 0 4px 12px rgba(37,99,235,.4); }
  .btn-pri:disabled { opacity: 0.4; cursor: not-allowed; box-shadow: none; }
</style>
