<script lang="ts">
  import type { ComponentTemplate, ScadaProject, TagDefinition } from "$lib/types";
  import { instantiateComponentTemplate, project, extractTagSlotsFromWidgets } from "$lib/stores/app";
  import type { TagSlot } from "$lib/stores/app";

  interface Props {
    open: boolean;
    template: ComponentTemplate | null;
    targetX?: number;
    targetY?: number;
    onClose: () => void;
  }

  let { open = $bindable(false), template, targetX = 60, targetY = 60, onClose }: Props = $props();

  const currentProject = $derived($project as ScadaProject | null);
  const allTags = $derived((currentProject?.tags ?? []) as TagDefinition[]);

  // ── Extract all slots for the template ──────────────────────────────────
  const allSlots = $derived.by((): TagSlot[] => {
    if (!template) return [];
    return extractTagSlotsFromWidgets(
      template.widgets,
      template.tag_slots_meta,
      currentProject?.tags,
    );
  });

  // ── Section per sub-widget (element) inside the component ───────────────
  type ElementSection = {
    widgetId: string;
    widgetLabel: string;
    widgetType: string;
    slots: TagSlot[];
  };

  const elementSections = $derived.by((): ElementSection[] => {
    if (!allSlots.length) return [];
    const map = new Map<string, ElementSection>();
    for (const slot of allSlots) {
      if (!map.has(slot.widgetId)) {
        map.set(slot.widgetId, {
          widgetId: slot.widgetId,
          widgetLabel: slot.widgetLabel || slot.widgetType,
          widgetType: slot.widgetType,
          slots: [],
        });
      }
      map.get(slot.widgetId)!.slots.push(slot);
    }
    return Array.from(map.values());
  });

  // ── Mapping & Picker State ───────────────────────────────────────────────
  let tagMapping = $state<Record<string, string>>({});
  let pickerSlot = $state<string | null>(null);
  let pickerSearch = $state("");
  let showAllTypes = $state(false);

  // Search & Filter state
  let slotSearchQuery = $state("");
  let showOnlyUnassigned = $state(false);

  // Auto assign toolbar state
  let autoPrefix = $state("");
  let autoAssignMsg = $state<string | null>(null);

  // Reset state when template changes or modal opens
  $effect(() => {
    if (template && open) {
      const slots = extractTagSlotsFromWidgets(
        template.widgets,
        template.tag_slots_meta,
        currentProject?.tags,
      );
      const m: Record<string, string> = {};
      for (const s of slots) {
        m[s.id] = "";
        m[s.slotKey] = "";
      }
      tagMapping = m;
      pickerSlot = null;
      pickerSearch = "";
      slotSearchQuery = "";
      showOnlyUnassigned = false;
      autoPrefix = "";
      autoAssignMsg = null;
    }
  });

  // ── Filtered Sections ───────────────────────────────────────────────────
  const filteredSections = $derived.by((): ElementSection[] => {
    const q = slotSearchQuery.toLowerCase().trim();
    return elementSections
      .map((section) => {
        const matchingSlots = section.slots.filter((slot) => {
          const assignedId = tagMapping[slot.id] || (tagMapping[slot.slotKey] ? tagMapping[slot.slotKey] : "");
          const isAssigned = !!assignedId;

          if (showOnlyUnassigned && isAssigned) return false;
          if (!q) return true;

          const vName = (slot.name || slotVarName(slot.slotKey)).toLowerCase();
          const fLbl = fieldLabel(slot.field).toLowerCase();
          const wLbl = section.widgetLabel.toLowerCase();
          const key = slot.slotKey.toLowerCase();
          return vName.includes(q) || fLbl.includes(q) || wLbl.includes(q) || key.includes(q);
        });

        return { ...section, slots: matchingSlots };
      })
      .filter((section) => section.slots.length > 0);
  });

  // ── Type Hinting ─────────────────────────────────────────────────────────
  function guessType(slot: TagSlot): string | null {
    const key = slot.field.toLowerCase();
    const val = slot.slotKey.toLowerCase();
    if (key === "blinktagid" || key === "statetagid" || key === "visibilitytagid") return "bool";
    if (val.includes("run") || val.includes("enable") || val.includes("ok") ||
        val.includes("fault") || val.includes("demand") || val.includes("cmd")) return "bool";
    if (val.includes("nazwa") || val.includes("name") || val.includes("label") ||
        val.includes("text") || val.includes("str")) return "string";
    return null;
  }

  function matchesHint(tag: TagDefinition, hint: string | null): boolean {
    if (!hint) return true;
    if (hint === "bool") return tag.data_type === "bool";
    if (hint === "string") return tag.data_type === "string";
    return tag.data_type !== "bool" && tag.data_type !== "string";
  }

  // ── Picker Tags ──────────────────────────────────────────────────────────
  const pickerTags = $derived.by((): TagDefinition[] => {
    if (!pickerSlot) return [];
    const slot = allSlots.find((s) => s.id === pickerSlot);
    const hint = slot ? guessType(slot) : null;
    const q = pickerSearch.toLowerCase();
    return allTags.filter((t) => {
      const typeOk = showAllTypes || matchesHint(t, hint);
      const searchOk = !q ||
        t.id.toLowerCase().includes(q) ||
        t.name.toLowerCase().includes(q) ||
        (t.description ?? "").toLowerCase().includes(q);
      return typeOk && searchOk;
    });
  });

  function openPicker(slotId: string) {
    if (pickerSlot === slotId) {
      pickerSlot = null;
    } else {
      pickerSlot = slotId;
      pickerSearch = "";
      showAllTypes = false;
    }
  }

  function assignTag(slot: TagSlot, tagId: string) {
    tagMapping[slot.id] = tagId;
    tagMapping[slot.slotKey] = tagId;
    pickerSlot = null;
  }

  function clearSlot(slot: TagSlot) {
    tagMapping[slot.id] = "";
    tagMapping[slot.slotKey] = "";
    if (pickerSlot === slot.id) pickerSlot = null;
  }

  function clearAllSlots() {
    const m: Record<string, string> = {};
    for (const s of allSlots) {
      m[s.id] = "";
      m[s.slotKey] = "";
    }
    tagMapping = m;
    pickerSlot = null;
    autoAssignMsg = "Wszystkie przypisania zostały wyczyszczone.";
  }

  // ── Smart Tag Auto-Assign by Prefix ──────────────────────────────────────
  function applyAutoAssign() {
    const prefix = autoPrefix.trim();
    if (!allTags.length) {
      autoAssignMsg = "⚠️ Brak dostępnych tagów w projekcie do przypisania.";
      return;
    }

    let matchedCount = 0;
    const newMapping = { ...tagMapping };

    for (const slot of allSlots) {
      const hint = guessType(slot);
      const varName = slotVarName(slot.slotKey);
      const rawKey = slot.slotKey;

      const candidates = [
        (prefix + varName).toLowerCase(),
        (prefix + slot.field).toLowerCase(),
        (prefix + rawKey).toLowerCase(),
      ];

      let foundTag: TagDefinition | undefined = allTags.find((t) => {
        if (hint && !matchesHint(t, hint)) return false;
        const tid = t.id.toLowerCase();
        const tname = t.name.toLowerCase();
        return candidates.some((st) => tid === st || tname === st);
      });

      if (!foundTag && prefix) {
        const p = prefix.toLowerCase();
        const vn = varName.toLowerCase();
        foundTag = allTags.find((t) => {
          if (hint && !matchesHint(t, hint)) return false;
          const tid = t.id.toLowerCase();
          const tname = t.name.toLowerCase();
          return (tid.startsWith(p) || tname.startsWith(p)) && (tid.includes(vn) || tname.includes(vn));
        });
      }

      if (!foundTag && !prefix) {
        const vn = varName.toLowerCase();
        foundTag = allTags.find((t) => {
          if (hint && !matchesHint(t, hint)) return false;
          return t.id.toLowerCase() === vn || t.name.toLowerCase() === vn;
        });
      }

      if (foundTag) {
        newMapping[slot.id] = foundTag.id;
        newMapping[slot.slotKey] = foundTag.id;
        matchedCount++;
      }
    }

    tagMapping = newMapping;
    if (matchedCount > 0) {
      autoAssignMsg = `✓ Automatycznie dopasowano ${matchedCount} z ${allSlots.length} slotów po prefiksie "${prefix || '(brak)'}".`;
    } else {
      autoAssignMsg = `⚠️ Nie znaleziono pasujących tagów dla prefiksu "${prefix}". Sprawdź pisownię tagów w projekcie.`;
    }
  }

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!template) return;
    const mapping: Record<string, string> = {};
    for (const [k, v] of Object.entries(tagMapping)) {
      if (v) mapping[k] = v;
    }
    instantiateComponentTemplate(template.id, targetX, targetY, {}, { tagMapping: mapping });
    onClose();
  }

  function tagTypeBadgeClass(type: string): string {
    if (type === "bool") return "badge-bool";
    if (type === "f32" || type === "f64") return "badge-float";
    if (type === "string") return "badge-string";
    return "badge-int";
  }

  function widgetIcon(type: string): string {
    switch (type) {
      case "image": return "🖼️";
      case "label": return "🏷️";
      case "button": return "🔘";
      case "gauge": return "📊";
      case "tank": return "🫙";
      case "pump": return "⚙️";
      case "valve": return "🔧";
      case "chart": return "📈";
      default: return "🧩";
    }
  }

  function fieldLabel(field: string): string {
    const map: Record<string, string> = {
      tag_id: "Główna zmienna (tag_id)",
      blinkTagId: "Miganie (Blink)",
      stateTagId: "Stan (State)",
      animationTagId: "Animacja",
      visibilityTagId: "Widoczność",
      setpointTagId: "Wartość zadana (SP)",
      levelTagId: "Poziom",
      flowTagId: "Przepływ",
      pressureTagId: "Ciśnienie",
      speedTagId: "Prędkość",
      tempTagId: "Temperatura",
      cmdTagId: "Komenda",
      runTagId: "Praca",
      faultTagId: "Awaria",
      enableTagId: "Zezwolenie",
      feedbackTagId: "Sprzężenie zwrotne",
    };
    return map[field] ?? field;
  }

  const assignedCount = $derived(
    allSlots.filter((s) => !!(tagMapping[s.id] || (tagMapping[s.slotKey] && tagMapping[s.slotKey]))).length
  );
  const totalSlots = $derived(allSlots.length);
  const unassignedCount = $derived(totalSlots - assignedCount);

  function slotVarName(slotKey: string): string {
    const stripped = slotKey.replace(/^\{[^}]+\}/, "").replace(/^_/, "");
    return stripped || slotKey;
  }
</script>

{#if open && template}
  <div class="backdrop" role="dialog" aria-modal="true">
    <button type="button" class="backdrop-dismiss" aria-label="Zamknij okno" onclick={onClose}></button>
    <form class="panel" onsubmit={handleSubmit}>

      <!-- ── HEADER ── -->
      <div class="header">
        <span class="hdr-icon">🧩</span>
        <div class="hdr-text">
          <h2>Wstaw komponent: <strong>{template.name}</strong></h2>
          <p>Przypisz zmienne dla każdego elementu składowego komponentu.</p>
        </div>
        <button type="button" class="btn-x" onclick={onClose} title="Zamknij">✕</button>
      </div>

      <!-- ── TOOLBAR (Smart Auto-Assign & Search) ── -->
      <div class="toolbar-section">
        <!-- Auto Assign Box -->
        <div class="auto-assign-bar">
          <div class="auto-assign-input-group">
            <span class="auto-icon">⚡</span>
            <input
              type="text"
              class="prefix-input"
              placeholder="Wpisz prefiks tagów (np. PUMP_101_ lub P1_)..."
              bind:value={autoPrefix}
              onkeydown={(e) => e.key === "Enter" && (e.preventDefault(), applyAutoAssign())}
            />
            <button type="button" class="btn-auto" onclick={applyAutoAssign} title="Dopasuj tagi w projekcie na podstawie wpisanego prefiksu">
              ⚡ Dopasuj po prefiksie
            </button>
          </div>
          {#if autoAssignMsg}
            <div class="auto-msg" class:auto-msg-ok={autoAssignMsg.startsWith('✓')} class:auto-msg-warn={autoAssignMsg.startsWith('⚠️')}>
              {autoAssignMsg}
            </div>
          {/if}
        </div>

        <!-- Search & Filter Controls -->
        <div class="filter-bar">
          <div class="search-wrap">
            <span class="search-icon">🔍</span>
            <input
              type="search"
              class="slot-search-input"
              placeholder="Filtruj elementy i zmienne..."
              bind:value={slotSearchQuery}
            />
            {#if slotSearchQuery}
              <button type="button" class="btn-clear-search" onclick={() => (slotSearchQuery = "")}>✕</button>
            {/if}
          </div>

          <div class="filter-actions">
            <label class="toggle-unassigned">
              <input type="checkbox" bind:checked={showOnlyUnassigned} />
              Tylko nieprzypisane ({unassignedCount})
            </label>
            {#if assignedCount > 0}
              <button type="button" class="btn-clear-all" onclick={clearAllSlots} title="Wyczyść wszystkie przypisane zmienne">
                Wyczyść przypisania
              </button>
            {/if}
          </div>
        </div>
      </div>

      <!-- ── BODY (Each Sub-Widget Element as a Card Section with its Alias) ── -->
      <div class="body">
        {#if allSlots.length === 0}
          <div class="empty-state">Komponent nie posiada zdefiniowanych slotów zmiennych.</div>
        {:else if filteredSections.length === 0}
          <div class="empty-state">
            Brak elementów spełniających kryteria filtrowania.
            {#if showOnlyUnassigned}
              <br/><button type="button" class="link-btn" onclick={() => (showOnlyUnassigned = false)}>Pokaż wszystkie elementy</button>
            {/if}
          </div>
        {:else}
          {#each filteredSections as section (section.widgetId)}
            <!-- Element Section Card titled with its ALIAS -->
            <div class="element-card">

              <!-- Element Card Header (Alias/Name) -->
              <div class="element-card-header">
                <span class="elem-icon">{widgetIcon(section.widgetType)}</span>
                <span class="elem-alias">{section.widgetLabel}</span>
                <span class="elem-type-badge">{section.widgetType}</span>
                <span class="elem-slot-count">
                  {section.slots.length} {section.slots.length === 1 ? 'zmienna' : 'zmienne'}
                </span>
              </div>

              <!-- List of variable properties inside this element -->
              <div class="element-slots-list">
                {#each section.slots as slot (slot.id)}
                  {@const tagId = tagMapping[slot.id] || (tagMapping[slot.slotKey] ? tagMapping[slot.slotKey] : "")}
                  {@const assigned = allTags.find((t) => t.id === tagId)}
                  {@const isOpen = pickerSlot === slot.id}
                  {@const hint = guessType(slot)}

                  <div class="slot-row" class:slot-done={!!assigned} class:slot-open={isOpen}>

                    <!-- Property Name & Details -->
                    <div class="slot-info">
                      <div class="slot-title-row">
                        <span class="field-title">{fieldLabel(slot.field)}</span>
                        {#if hint}
                          <span class="hint-badge hint-{hint}">{hint}</span>
                        {/if}
                        <code class="slot-key-tag">{slot.slotKey}</code>
                      </div>
                      {#if slot.comment && !slot.comment.startsWith("Zmienna:")}
                        <div class="slot-comment">💬 {slot.comment}</div>
                      {/if}
                    </div>

                    <!-- Assigned Tag Value Chip -->
                    <div class="slot-value">
                      {#if assigned}
                        <div class="tag-chip">
                          <span class="chip-id">{assigned.id}</span>
                          <span class="chip-name">{assigned.name}</span>
                          <span class="chip-badge {tagTypeBadgeClass(assigned.data_type)}">{assigned.data_type}</span>
                          {#if assigned.binding?.table === "memory"}<span class="chip-badge badge-mem">MEM</span>{/if}
                        </div>
                      {:else}
                        <span class="unassigned-badge">— nie przypisano —</span>
                      {/if}
                    </div>

                    <!-- Action Buttons -->
                    <div class="slot-actions">
                      <button
                        type="button"
                        class="btn-pick"
                        class:btn-pick-active={isOpen}
                        onclick={() => openPicker(slot.id)}
                      >
                        {#if assigned}✏️ Zmień{:else}📎 Wybierz…{/if}
                      </button>
                      {#if assigned}
                        <button type="button" class="btn-clear-slot" onclick={() => clearSlot(slot)} title="Usuń przypisanie">✕</button>
                      {/if}
                    </div>
                  </div>

                  <!-- INLINE TAG PICKER -->
                  {#if isOpen}
                    <div class="picker">
                      <div class="picker-toolbar">
                        <input
                          id="picker-search"
                          class="picker-search"
                          type="search"
                          placeholder="Szukaj ID tagu, nazwy, opisu..."
                          bind:value={pickerSearch}
                          autocomplete="off"
                        />
                        <label class="show-all-label">
                          <input type="checkbox" bind:checked={showAllTypes} />
                          Wszystkie typy
                        </label>
                        <button type="button" class="btn-close-picker" onclick={() => { pickerSlot = null; }}>✕</button>
                      </div>

                      {#if hint && !showAllTypes}
                        <div class="picker-hint">
                          Sugerowany typ danych: <strong>{hint}</strong> · Znaleziono {pickerTags.length} pasujących tagów
                        </div>
                      {/if}

                      <div class="picker-list">
                        {#each pickerTags as tag (tag.id)}
                          <div
                            class="picker-row"
                            class:picker-selected={tagMapping[slot.id] === tag.id || tagMapping[slot.slotKey] === tag.id}
                            role="option"
                            tabindex="0"
                            aria-selected={tagMapping[slot.id] === tag.id || tagMapping[slot.slotKey] === tag.id}
                            onclick={() => assignTag(slot, tag.id)}
                            onkeydown={(e) => (e.key === "Enter" || e.key === " ") && assignTag(slot, tag.id)}
                          >
                            <span class="pr-id">{tag.id}</span>
                            <span class="pr-name">{tag.name}</span>
                            {#if tag.description}<span class="pr-desc">{tag.description}</span>{/if}
                            <span class="pr-badge {tagTypeBadgeClass(tag.data_type)}">{tag.data_type}</span>
                            {#if tag.binding?.table === "memory"}<span class="pr-badge badge-mem">MEM</span>{/if}
                            <span class="pr-addr">{tag.binding?.table}:{tag.binding?.address}{tag.binding?.bit !== undefined ? ` b${tag.binding.bit}` : ""}</span>
                          </div>
                        {:else}
                          <div class="picker-empty">
                            Brak tagów spełniających kryteria.
                            {#if !showAllTypes}
                              <button type="button" class="link-btn" onclick={() => showAllTypes = true}>Pokaż wszystkie typy tagów</button>
                            {/if}
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/if}
                {/each}
              </div>

            </div>
          {/each}
        {/if}
      </div>

      <!-- ── FOOTER ── -->
      <div class="footer">
        <div class="footer-info">
          <span>Przypisano <strong class="counter-assigned">{assignedCount}</strong> z <strong>{totalSlots}</strong> zmiennych</span>
          {#if unassignedCount > 0}
            <span class="counter-unassigned">({unassignedCount} nieprzypisanych)</span>
          {/if}
        </div>
        <div class="footer-btns">
          <button type="button" class="btn btn-sec" onclick={onClose}>Anuluj</button>
          <button type="submit" class="btn btn-pri">
            🧩 Wstaw komponent na ekran
          </button>
        </div>
      </div>
    </form>
  </div>
{/if}

<style>
  /* ── BACKDROP & CONTAINER ── */
  .backdrop-dismiss {
    position: absolute; inset: 0;
    appearance: none; border: 0; padding: 0; margin: 0;
    background: transparent; cursor: default;
  }
  .backdrop {
    position: fixed; inset: 0; z-index: 9200;
    background: rgba(0, 0, 0, 0.75);
    display: flex; align-items: center; justify-content: center;
    padding: 16px;
    backdrop-filter: blur(4px);
  }

  .panel {
    position: relative; z-index: 1;
    display: flex; flex-direction: column;
    width: 100%; max-width: 960px;
    height: min(85vh, 760px);
    background: var(--gh-canvas-overlay, #161b22);
    border: 1px solid var(--gh-border-default, #30363d);
    border-radius: 12px; overflow: hidden;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.8), 0 0 0 1px rgba(163, 113, 247, 0.2);
    color: var(--gh-fg-default, #e6edf3);
  }

  /* ── HEADER ── */
  .header {
    display: flex; align-items: flex-start; gap: 14px;
    padding: 16px 20px 14px;
    background: var(--gh-canvas-default, #0d1117);
    border-bottom: 1px solid var(--gh-border-default, #30363d);
    flex-shrink: 0;
  }
  .hdr-icon { font-size: 26px; padding-top: 2px; }
  .hdr-text { flex: 1; min-width: 0; }
  .hdr-text h2 { margin: 0 0 3px; font-size: 18px; font-weight: 600; color: var(--vs-text-bright, #f0f6fc); }
  .hdr-text h2 strong { color: var(--copilot-purple-light, #a371f7); }
  .hdr-text p { margin: 0; font-size: 12px; color: var(--gh-fg-muted, #848d97); }

  .btn-x {
    background: none; border: none; color: var(--gh-fg-muted, #848d97);
    font-size: 18px; cursor: pointer; padding: 4px 8px;
    border-radius: 4px; transition: all .15s; flex-shrink: 0;
  }
  .btn-x:hover { background: var(--gh-border-muted, #21262d); color: var(--vs-text-bright, #f0f6fc); }

  /* ── TOOLBAR (AUTO ASSIGN & FILTER) ── */
  .toolbar-section {
    display: flex; flex-direction: column; gap: 10px;
    padding: 12px 20px;
    background: var(--gh-canvas-inset, #010409);
    border-bottom: 1px solid var(--gh-border-default, #30363d);
    flex-shrink: 0;
  }

  .auto-assign-bar { display: flex; flex-direction: column; gap: 6px; }
  .auto-assign-input-group { display: flex; align-items: center; gap: 8px; }
  .auto-icon { font-size: 14px; color: var(--copilot-purple-light, #a371f7); }

  .prefix-input {
    flex: 1; background: var(--gh-canvas-default, #0d1117);
    border: 1px solid var(--gh-border-default, #30363d);
    border-radius: 6px; padding: 7px 12px;
    color: var(--gh-fg-default, #e6edf3); font-size: 13px; outline: none;
    transition: all .15s;
  }
  .prefix-input:focus {
    border-color: var(--copilot-purple-light, #a371f7);
    box-shadow: 0 0 0 2px rgba(163, 113, 247, 0.25);
  }

  .btn-auto {
    background: linear-gradient(135deg, rgba(163, 113, 247, 0.2) 0%, rgba(57, 197, 207, 0.2) 100%);
    border: 1px solid var(--copilot-purple-light, #a371f7);
    color: var(--copilot-purple-light, #a371f7);
    border-radius: 6px; padding: 7px 14px; font-size: 12px; font-weight: 600;
    cursor: pointer; transition: all .15s; white-space: nowrap;
  }
  .btn-auto:hover {
    background: var(--copilot-gradient); color: #fff;
    box-shadow: 0 0 12px rgba(163, 113, 247, 0.4);
  }

  .auto-msg { font-size: 11px; padding: 3px 8px; border-radius: 4px; }
  .auto-msg-ok { color: #34d399; background: rgba(52, 211, 153, 0.1); border: 1px solid rgba(52, 211, 153, 0.2); }
  .auto-msg-warn { color: #f87171; background: rgba(248, 113, 113, 0.1); border: 1px solid rgba(248, 113, 113, 0.2); }

  .filter-bar { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .search-wrap { position: relative; flex: 1; max-width: 360px; display: flex; align-items: center; }
  .search-icon { position: absolute; left: 10px; font-size: 12px; opacity: 0.6; pointer-events: none; }
  .slot-search-input {
    width: 100%; background: var(--gh-canvas-default, #0d1117);
    border: 1px solid var(--gh-border-default, #30363d);
    border-radius: 6px; padding: 5px 28px 5px 28px;
    color: var(--gh-fg-default, #e6edf3); font-size: 12px; outline: none;
    transition: border .15s;
  }
  .slot-search-input:focus { border-color: var(--copilot-cyan, #39c5cf); }
  .btn-clear-search {
    position: absolute; right: 8px; background: none; border: none;
    color: var(--gh-fg-muted, #848d97); cursor: pointer; font-size: 12px;
  }

  .filter-actions { display: flex; align-items: center; gap: 14px; }
  .toggle-unassigned {
    display: flex; align-items: center; gap: 6px;
    font-size: 12px; color: var(--gh-fg-muted, #848d97); cursor: pointer; user-select: none;
  }
  .toggle-unassigned input { cursor: pointer; }

  .btn-clear-all {
    background: none; border: none; color: #f87171; font-size: 11px;
    cursor: pointer; text-decoration: underline; padding: 0;
  }

  /* ── BODY ── */
  .body { flex: 1; overflow-y: auto; padding: 16px 20px; display: flex; flex-direction: column; gap: 14px; }
  .empty-state { padding: 48px; text-align: center; color: var(--gh-fg-muted, #848d97); font-size: 13px; }

  /* ── ELEMENT CARD (SECTION FOR EACH SUB-WIDGET ELEMENT) ── */
  .element-card {
    background: var(--gh-canvas-default, #0d1117);
    border: 1px solid var(--gh-border-default, #30363d);
    border-radius: 10px; overflow: hidden;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  .element-card-header {
    display: flex; align-items: center; gap: 10px;
    padding: 10px 14px;
    background: var(--gh-canvas-inset, #010409);
    border-bottom: 1px solid var(--gh-border-default, #30363d);
  }
  .elem-icon { font-size: 16px; }
  .elem-alias { font-size: 14px; font-weight: 700; color: var(--vs-text-bright, #f0f6fc); flex: 1; }

  .elem-type-badge {
    font-size: 10px; color: var(--copilot-purple-light, #a371f7); font-family: var(--font-mono, monospace);
    background: rgba(163, 113, 247, 0.15); padding: 2px 8px; border-radius: 999px; border: 1px solid rgba(163, 113, 247, 0.3);
  }
  .elem-slot-count {
    font-size: 11px; color: var(--copilot-cyan, #39c5cf); font-weight: 600;
    background: rgba(57, 197, 207, 0.12); padding: 2px 8px; border-radius: 999px;
  }

  /* ── LIST OF TAG PROPERTIES INSIDE ELEMENT CARD ── */
  .element-slots-list { display: flex; flex-direction: column; }

  .slot-row {
    display: flex; align-items: center; gap: 12px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--gh-border-muted, #21262d);
    transition: all .15s;
  }
  .slot-row:last-child { border-bottom: none; }
  .slot-row:hover { background: #131924; }
  .slot-row.slot-done { background: rgba(52, 211, 153, 0.02); }
  .slot-row.slot-open { background: rgba(163, 113, 247, 0.05); }

  .slot-info { width: 320px; flex-shrink: 0; display: flex; flex-direction: column; gap: 3px; }
  .slot-title-row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .field-title { font-size: 13px; font-weight: 600; color: var(--gh-fg-default, #e6edf3); }

  .slot-key-tag { font-family: var(--font-mono, monospace); font-size: 10px; color: var(--gh-fg-subtle, #6e7681); background: var(--gh-canvas-inset, #010409); padding: 1px 5px; border-radius: 3px; }
  .slot-comment { font-size: 11px; color: var(--gh-fg-muted, #848d97); line-height: 1.2; }

  .hint-badge { font-size: 9px; font-weight: 700; padding: 1px 5px; border-radius: 999px; text-transform: uppercase; font-family: monospace; }
  .hint-bool   { background: rgba(6, 78, 59, 0.6); color: #6ee7b7; border: 1px solid rgba(110, 231, 183, 0.3); }
  .hint-string { background: rgba(76, 29, 149, 0.6); color: #c4b5fd; border: 1px solid rgba(196, 181, 253, 0.3); }

  .slot-value { flex: 1; min-width: 0; }

  .tag-chip {
    display: inline-flex; align-items: center; gap: 8px;
    background: var(--gh-canvas-inset, #010409);
    border: 1px solid var(--copilot-cyan, #39c5cf);
    border-radius: 6px; padding: 4px 10px; max-width: 100%;
  }
  .chip-id { font-family: var(--font-mono, monospace); font-size: 12px; color: var(--copilot-cyan, #39c5cf); font-weight: 600; }
  .chip-name { font-size: 11px; color: var(--gh-fg-default, #e6edf3); min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .unassigned-badge { font-size: 11px; color: var(--gh-fg-subtle, #6e7681); font-style: italic; }

  .slot-actions { display: flex; align-items: center; gap: 6px; flex-shrink: 0; }

  .btn-pick {
    background: var(--gh-border-muted, #21262d); border: 1px solid var(--gh-border-default, #30363d);
    color: var(--gh-fg-default, #e6edf3); border-radius: 6px; padding: 5px 12px;
    font-size: 12px; font-weight: 500; cursor: pointer; transition: all .15s; white-space: nowrap;
  }
  .btn-pick:hover, .btn-pick-active {
    background: var(--copilot-purple-light, #a371f7); color: #fff;
    border-color: var(--copilot-purple-light, #a371f7);
    box-shadow: 0 0 10px rgba(163, 113, 247, 0.35);
  }

  .btn-clear-slot {
    background: none; border: 1px solid var(--gh-border-default, #30363d);
    color: var(--gh-fg-muted, #848d97); border-radius: 6px; padding: 5px 8px;
    font-size: 11px; cursor: pointer; transition: all .15s;
  }
  .btn-clear-slot:hover { background: rgba(218, 54, 51, 0.2); color: #f87171; border-color: rgba(218, 54, 51, 0.4); }

  /* ── INLINE TAG PICKER ── */
  .picker {
    margin: 6px 14px 10px;
    background: var(--gh-canvas-inset, #010409);
    border: 1px solid var(--copilot-purple-light, #a371f7);
    border-radius: 8px; overflow: hidden;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
  }

  .picker-toolbar {
    display: flex; align-items: center; gap: 8px;
    padding: 8px 12px;
    background: var(--gh-canvas-default, #0d1117);
    border-bottom: 1px solid var(--gh-border-default, #30363d);
  }

  .picker-search {
    flex: 1; background: var(--gh-canvas-inset, #010409);
    border: 1px solid var(--gh-border-default, #30363d);
    border-radius: 5px; padding: 5px 10px; color: var(--gh-fg-default, #e6edf3);
    font-size: 12px; outline: none; transition: border .15s;
  }
  .picker-search:focus { border-color: var(--copilot-purple-light, #a371f7); }

  .show-all-label {
    display: flex; align-items: center; gap: 5px;
    font-size: 11px; color: var(--gh-fg-muted, #848d97); cursor: pointer;
    white-space: nowrap; user-select: none;
  }
  .show-all-label input { cursor: pointer; }

  .btn-close-picker {
    background: none; border: 1px solid var(--gh-border-default, #30363d);
    color: var(--gh-fg-muted, #848d97); border-radius: 5px; padding: 4px 7px;
    font-size: 11px; cursor: pointer; transition: all .15s;
  }
  .btn-close-picker:hover { background: var(--gh-border-muted, #21262d); color: var(--vs-text-bright, #f0f6fc); }

  .picker-hint {
    padding: 5px 12px; font-size: 10px; color: var(--gh-fg-muted, #848d97);
    background: var(--gh-canvas-default, #0d1117); border-bottom: 1px solid var(--gh-border-muted, #21262d);
  }
  .picker-hint strong { color: var(--copilot-cyan, #39c5cf); }

  .picker-list { max-height: 220px; overflow-y: auto; }

  .picker-row {
    display: flex; align-items: center; gap: 8px;
    padding: 7px 12px; cursor: pointer;
    border-bottom: 1px solid var(--gh-border-muted, #21262d);
    transition: background .1s;
  }
  .picker-row:last-child { border-bottom: none; }
  .picker-row:hover { background: var(--gh-border-muted, #21262d); }
  .picker-selected { background: rgba(163, 113, 247, 0.2) !important; outline: 1px solid var(--copilot-purple-light, #a371f7); }

  .pr-id { font-family: var(--font-mono, monospace); font-size: 12px; color: var(--copilot-cyan, #39c5cf); min-width: 100px; flex-shrink: 0; }
  .pr-name { font-size: 12px; color: var(--gh-fg-default, #e6edf3); flex-shrink: 0; }
  .pr-desc { font-size: 10px; color: var(--gh-fg-muted, #848d97); flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .pr-badge {
    font-size: 9px; font-weight: 700; padding: 1px 5px;
    border-radius: 999px; text-transform: uppercase; font-family: monospace; flex-shrink: 0;
  }
  .pr-addr { font-size: 10px; color: var(--gh-fg-subtle, #6e7681); font-family: monospace; flex-shrink: 0; }

  .picker-empty { padding: 18px; text-align: center; color: var(--gh-fg-muted, #848d97); font-size: 12px; }
  .link-btn { background: none; border: none; color: var(--copilot-cyan, #39c5cf); font-size: 12px; cursor: pointer; text-decoration: underline; padding: 0 4px; }

  /* ── BADGES ── */
  .badge-bool   { background: rgba(6, 78, 59, 0.8); color: #6ee7b7; }
  .badge-float  { background: rgba(30, 58, 138, 0.8); color: #93c5fd; }
  .badge-string { background: rgba(76, 29, 149, 0.8); color: #c4b5fd; }
  .badge-int    { background: var(--gh-border-muted, #21262d); color: var(--gh-fg-muted, #848d97); }
  .badge-mem    { background: rgba(113, 63, 18, 0.8); color: #fcd34d; }
  .chip-badge   { font-size: 9px; font-weight: 700; padding: 1px 5px; border-radius: 999px; text-transform: uppercase; font-family: monospace; flex-shrink: 0; }

  /* ── FOOTER ── */
  .footer {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 20px;
    background: var(--gh-canvas-default, #0d1117);
    border-top: 1px solid var(--gh-border-default, #30363d);
    flex-shrink: 0; gap: 12px;
  }
  .footer-info { font-size: 12px; color: var(--gh-fg-muted, #848d97); display: flex; align-items: center; gap: 6px; }
  .counter-assigned { color: #34d399; }
  .counter-unassigned { color: var(--gh-fg-subtle, #6e7681); }

  .footer-btns { display: flex; gap: 10px; }

  .btn {
    padding: 8px 20px; border-radius: 6px; font-size: 13px;
    font-weight: 500; border: none; cursor: pointer; transition: all .15s;
  }
  .btn-sec {
    background: var(--gh-border-muted, #21262d); color: var(--gh-fg-muted, #848d97);
    border: 1px solid var(--gh-border-default, #30363d);
  }
  .btn-sec:hover { background: var(--gh-border-default, #30363d); color: var(--gh-fg-default, #e6edf3); }

  .btn-pri {
    background: var(--copilot-gradient); color: #fff; font-weight: 600;
    box-shadow: 0 0 12px rgba(163, 113, 247, 0.35); border: 1px solid rgba(255, 255, 255, 0.2);
  }
  .btn-pri:hover {
    background: linear-gradient(135deg, #b78af7 0%, #388bfd 100%);
    box-shadow: 0 0 16px rgba(163, 113, 247, 0.5);
  }
</style>
