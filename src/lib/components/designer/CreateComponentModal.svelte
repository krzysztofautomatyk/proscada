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
    background: var(--gh-canvas-overlay, #161b22); border: 1px solid var(--gh-border-default, #30363d);
    border-radius: 10px; overflow: hidden;
    box-shadow: 0 24px 64px rgba(0,0,0,.8), 0 0 0 1px rgba(163, 113, 247, 0.2);
    color: var(--gh-fg-default, #e6edf3);
  }

  /* HEADER */
  .header {
    display: flex; align-items: flex-start; gap: 12px;
    padding: 16px 20px 14px;
    background: var(--gh-canvas-default, #0d1117); border-bottom: 1px solid var(--gh-border-default, #30363d);
    flex-shrink: 0;
  }
  .hdr-icon { font-size: 26px; padding-top: 2px; }
  .hdr-text { flex: 1; min-width: 0; }
  .hdr-text h2 { margin: 0 0 3px; font-size: 18px; font-weight: 600; color: var(--vs-text-bright, #f0f6fc); }
  .hdr-text p { margin: 0; font-size: 12px; color: var(--gh-fg-muted, #848d97); }
  .btn-x {
    background: none; border: none; color: var(--gh-fg-muted, #848d97);
    font-size: 18px; cursor: pointer; padding: 4px 8px;
    border-radius: 4px; transition: all .15s; flex-shrink: 0;
  }
  .btn-x:hover { background: var(--gh-border-muted, #21262d); color: var(--vs-text-bright, #f0f6fc); }

  /* BODY */
  .body {
    padding: 20px; display: flex; flex-direction: column; gap: 18px;
    overflow-y: auto; max-height: calc(85vh - 120px);
  }

  /* INFO BOX */
  .info-box {
    background: var(--gh-canvas-inset, #010409); border: 1px solid var(--gh-border-muted, #21262d);
    border-radius: 8px; padding: 12px 14px;
  }
  .info-empty { color: #f87171; font-size: 13px; text-align: center; }
  .info-summary { font-size: 13px; color: var(--copilot-cyan, #39c5cf); margin-bottom: 8px; }
  .summary-badge strong { color: var(--copilot-purple-light, #a371f7); }

  .slots-preview { display: flex; flex-direction: column; gap: 6px; margin-top: 4px; }
  .preview-label { font-size: 11px; color: var(--gh-fg-muted, #848d97); font-weight: 600; text-transform: uppercase; letter-spacing: .04em; }
  .slots-chips { display: flex; flex-wrap: wrap; gap: 6px; }

  .slot-chip {
    display: inline-flex; align-items: center; gap: 6px;
    background: var(--gh-canvas-default, #0d1117); border: 1px solid var(--gh-border-default, #30363d);
    border-radius: 5px; padding: 3px 8px; font-size: 11px;
  }
  .chip-name { color: var(--gh-fg-default, #e6edf3); font-weight: 600; }
  .chip-field { color: var(--copilot-purple-light, #a371f7); font-size: 10px; background: rgba(163, 113, 247, 0.15); padding: 1px 4px; border-radius: 3px; }
  .chip-key { font-family: var(--font-mono, monospace); font-size: 10px; color: var(--gh-fg-subtle, #6e7681); }

  /* FORM GRID */
  .form-grid { display: flex; flex-direction: column; gap: 14px; }
  .form-group { display: flex; flex-direction: column; gap: 5px; }
  .form-row { display: flex; gap: 12px; }
  .flex-1 { flex: 1; }

  label { font-size: 13px; font-weight: 500; color: var(--gh-fg-muted, #848d97); }
  .label-req { font-weight: 600; color: var(--gh-fg-default, #e6edf3); }
  .req-star { color: var(--gh-danger-emphasis, #da3633); margin-left: 2px; }

  .input-text, .input-textarea {
    background: var(--gh-canvas-inset, #010409); border: 1px solid var(--gh-border-default, #30363d);
    border-radius: 6px; padding: 8px 12px; color: var(--gh-fg-default, #e6edf3);
    font-size: 13px; outline: none; transition: all .15s;
    width: 100%; box-sizing: border-box;
  }
  .input-text:focus, .input-textarea:focus {
    border-color: var(--copilot-purple-light, #a371f7); box-shadow: 0 0 0 2px rgba(163, 113, 247, 0.3);
  }
  .input-err {
    border-color: var(--gh-danger-emphasis, #da3633) !important; background: rgba(218, 54, 51, 0.15) !important;
  }

  .input-textarea { resize: vertical; font-family: inherit; }

  .err-msg { font-size: 12px; color: #f87171; margin-top: 2px; }
  .err-msg strong { color: #fca5a5; }
  .help-msg { font-size: 11px; color: var(--gh-fg-subtle, #6e7681); }

  /* FOOTER */
  .footer {
    display: flex; align-items: center; justify-content: flex-end; gap: 10px;
    padding: 14px 20px; background: var(--gh-canvas-default, #0d1117); border-top: 1px solid var(--gh-border-default, #30363d);
    flex-shrink: 0;
  }

  .btn {
    padding: 8px 20px; border-radius: 6px; font-size: 13px;
    font-weight: 500; border: none; cursor: pointer; transition: all .15s;
  }
  .btn-sec { background: var(--gh-border-muted, #21262d); color: var(--gh-fg-muted, #848d97); border: 1px solid var(--gh-border-default, #30363d); }
  .btn-sec:hover { background: var(--gh-border-default, #30363d); color: var(--gh-fg-default, #e6edf3); }
  .btn-pri {
    background: var(--copilot-gradient); color: #fff; font-weight: 600;
    box-shadow: 0 0 12px rgba(163, 113, 247, 0.35); border: 1px solid rgba(255, 255, 255, 0.2);
  }
  .btn-pri:hover:not(:disabled) { background: linear-gradient(135deg, #b78af7 0%, #388bfd 100%); box-shadow: 0 0 16px rgba(163, 113, 247, 0.5); }
  .btn-pri:disabled { opacity: 0.4; cursor: not-allowed; box-shadow: none; }
</style>
