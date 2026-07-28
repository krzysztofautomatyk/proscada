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

  // ── Slots grouped by widgetType/widgetLabel ──────────────────────────────
  type SlotGroup = {
    widgetId: string;
    widgetLabel: string;
    widgetType: string;
    slots: TagSlot[];
  };

  const slotGroups = $derived.by((): SlotGroup[] => {
    if (!template) return [];
    const slots = extractTagSlotsFromWidgets(
      template.widgets,
      template.tag_slots_meta,
      currentProject?.tags,
    );
    // Group by widgetId without deduplicating any field!
    const map = new Map<string, SlotGroup>();
    for (const s of slots) {
      if (!map.has(s.widgetId)) {
        map.set(s.widgetId, {
          widgetId: s.widgetId,
          widgetLabel: s.widgetLabel,
          widgetType: s.widgetType,
          slots: [],
        });
      }
      map.get(s.widgetId)!.slots.push(s);
    }
    return Array.from(map.values());
  });

  const allSlots = $derived(slotGroups.flatMap((g) => g.slots));

  // tagMapping: slot.id (or slotKey) → assigned tag id
  let tagMapping = $state<Record<string, string>>({});
  // pickerSlot: which slot's ID is currently open
  let pickerSlot = $state<string | null>(null);
  let pickerSearch = $state("");
  let showAllTypes = $state(false);

  // Reset on template change
  $effect(() => {
    if (template) {
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
    }
  });

  // ── Type hinting ─────────────────────────────────────────────────────────
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

  // ── Picker tags ──────────────────────────────────────────────────────────
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

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!template) return;
    // Build mapping: templateTagKey → realTagId
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
      tag_id: "Główna zmienna",
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

  const assignedCount = $derived(Object.values(tagMapping).filter(Boolean).length);
  const totalSlots = $derived(allSlots.length);

  /** Extract meaningful variable name from template slot key.
   * e.g. "{tagPrefix}hh" → "hh",  "{tagPrefix}Nazwa" → "Nazwa", "MY_VAR" → "MY_VAR" */
  function slotVarName(slotKey: string): string {
    const stripped = slotKey.replace(/^\{[^}]+\}/, "").replace(/^_/, "");
    return stripped || slotKey;
  }
</script>

{#if open && template}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}
  >
    <form class="panel" onsubmit={handleSubmit}>

      <!-- ── HEADER ── -->
      <div class="header">
        <span class="hdr-icon">🧩</span>
        <div class="hdr-text">
          <h2>Wstaw komponent: <strong>{template.name}</strong></h2>
          <p>Kliknij <em>Wybierz…</em> przy slocie, aby przypisać istniejącą zmienną projektu.</p>
        </div>
        <button type="button" class="btn-x" onclick={onClose} title="Zamknij">✕</button>
      </div>

      <!-- ── BODY ── -->
      <div class="body">
        {#if slotGroups.length === 0}
          <div class="empty-state">Komponent nie zawiera slotów zmiennych.</div>
        {:else}
          {#each slotGroups as group (group.widgetId)}
            <!-- Widget group header -->
            <div class="group-header">
              <span class="group-icon">{widgetIcon(group.widgetType)}</span>
              <span class="group-label">{group.widgetLabel}</span>
              <span class="group-type">{group.widgetType}</span>
            </div>

            {#each group.slots as slot (slot.id)}
              {@const assigned = allTags.find((t) => t.id === tagMapping[slot.id] || (tagMapping[slot.slotKey] && t.id === tagMapping[slot.slotKey]))}
              {@const isOpen = pickerSlot === slot.id}
              {@const hint = guessType(slot)}

              <div class="slot-row" class:slot-done={!!assigned} class:slot-open={isOpen}>
                <!-- Slot info -->
                <div class="slot-info">
                  <div class="slot-name-row">
                    <span class="slot-varname">{slot.name || slotVarName(slot.slotKey)}</span>
                    {#if hint}
                      <span class="hint-badge hint-{hint}">{hint}</span>
                    {/if}
                    <span class="slot-field-label">{fieldLabel(slot.field)}</span>
                  </div>
                  <div class="slot-comment">
                    💬 {assigned?.description || slot.comment || `Zmienna: ${slot.name || slotVarName(slot.slotKey)} (${fieldLabel(slot.field)})`}
                  </div>
                  <div class="slot-key-row">
                    <span class="slot-key">{slot.slotKey}</span>
                  </div>
                </div>

                <!-- Assigned value chip -->
                <div class="slot-value">
                  {#if assigned}
                    <div class="chip">
                      <span class="chip-id">{assigned.id}</span>
                      <span class="chip-name">{assigned.name}</span>
                      <span class="chip-badge {tagTypeBadgeClass(assigned.data_type)}">{assigned.data_type}</span>
                    </div>
                  {:else}
                    <span class="unassigned">— nie przypisano —</span>
                  {/if}
                </div>

                <!-- Actions -->
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

              <!-- INLINE PICKER -->
              {#if isOpen}
                <div class="picker">
                  <div class="picker-toolbar">
                    <input
                      id="picker-search"
                      class="picker-search"
                      type="search"
                      placeholder="Szukaj ID, nazwy, opisu…"
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
                      Filtr: <strong>{hint}</strong> · {pickerTags.length} zmiennych
                    </div>
                  {/if}

                  <div class="picker-list">
                    {#each pickerTags as tag (tag.id)}
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
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
                        Brak zmiennych.
                        {#if !showAllTypes}
                          <button type="button" class="link-btn" onclick={() => showAllTypes = true}>Pokaż wszystkie typy</button>
                        {/if}
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            {/each}
          {/each}
        {/if}
      </div>

      <!-- ── FOOTER ── -->
      <div class="footer">
        <span class="footer-info">
          Przypisano <strong>{assignedCount}</strong> / {totalSlots} slotów
        </span>
        <div class="footer-btns">
          <button type="button" class="btn btn-sec" onclick={onClose}>Anuluj</button>
          <button type="submit" class="btn btn-pri">🧩 Wstaw komponent na ekran</button>
        </div>
      </div>
    </form>
  </div>
{/if}

<style>
  /* ── BACKDROP ── */
  .backdrop {
    position: fixed; inset: 0; z-index: 9000;
    background: rgba(0,0,0,.65);
    display: flex; align-items: flex-start; justify-content: center;
    padding: 12px;
    backdrop-filter: blur(2px);
  }

  /* ── PANEL ── */
  .panel {
    display: flex; flex-direction: column;
    width: calc(100vw - 24px);
    height: calc(100vh - 24px);
    max-width: 1100px;
    background: #1a2236; border: 1px solid #2d3f5a;
    border-radius: 10px; overflow: hidden;
    box-shadow: 0 24px 64px rgba(0,0,0,.7);
    color: #e2e8f0;
  }

  /* ── HEADER ── */
  .header {
    display: flex; align-items: flex-start; gap: 12px;
    padding: 14px 18px 12px;
    background: #111827; border-bottom: 1px solid #2d3f5a;
    flex-shrink: 0;
  }
  .hdr-icon { font-size: 26px; padding-top: 2px; }
  .hdr-text { flex: 1; min-width: 0; }
  .hdr-text h2 { margin: 0 0 3px; font-size: 17px; font-weight: 600; color: #f1f5f9; }
  .hdr-text h2 strong { color: #60a5fa; }
  .hdr-text p { margin: 0; font-size: 12px; color: #94a3b8; }
  .hdr-text em { color: #60a5fa; font-style: normal; }
  .btn-x {
    background: none; border: none; color: #64748b;
    font-size: 17px; cursor: pointer; padding: 4px 8px;
    border-radius: 4px; transition: all .15s; flex-shrink: 0;
  }
  .btn-x:hover { background: #2d3f5a; color: #f1f5f9; }

  /* ── BODY ── */
  .body { flex: 1; overflow-y: auto; padding-bottom: 8px; }
  .empty-state { padding: 60px; text-align: center; color: #475569; font-size: 14px; }

  /* ── GROUP HEADER ── */
  .group-header {
    display: flex; align-items: center; gap: 8px;
    padding: 10px 18px 6px;
    background: #0f1623;
    border-top: 1px solid #1e2a40;
    border-bottom: 1px solid #1e2a40;
    position: sticky; top: 0; z-index: 5;
  }
  .group-icon { font-size: 15px; }
  .group-label { font-size: 13px; font-weight: 600; color: #cbd5e1; }
  .group-type {
    font-size: 10px; color: #475569; font-family: monospace;
    background: #1e293b; padding: 1px 6px; border-radius: 999px;
  }

  /* ── SLOT ROW ── */
  .slot-row {
    display: flex; align-items: center; gap: 12px;
    padding: 10px 18px;
    border-bottom: 1px solid #192030;
    transition: background .1s;
  }
  .slot-row:hover { background: #1e2d46; }
  .slot-row.slot-done { background: #162038; }
  .slot-row.slot-open { background: #112040; }

  .slot-info { width: 280px; flex-shrink: 0; display: flex; flex-direction: column; gap: 2px; }
  .slot-name-row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .slot-varname { font-size: 14px; font-weight: 700; color: #f1f5f9; letter-spacing: .01em; }
  .slot-field-label { font-size: 10px; color: #60a5fa; font-weight: 600; background: #0f1d33; padding: 2px 6px; border-radius: 4px; border: 1px solid #1e3a5f; }
  .slot-comment { font-size: 11px; color: #94a3b8; line-height: 1.3; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .slot-key-row { display: flex; align-items: center; gap: 6px; margin-top: 1px; }
  .slot-key { font-size: 10px; color: #475569; font-family: monospace; word-break: break-all; }
  .hint-bool   { background: #064e3b; color: #6ee7b7; }
  .hint-string { background: #4c1d95; color: #c4b5fd; }

  .slot-value { flex: 1; min-width: 0; }

  .chip {
    display: inline-flex; align-items: center; gap: 8px;
    background: #0f2444; border: 1px solid #1d4ed8;
    border-radius: 6px; padding: 5px 10px; max-width: 100%;
  }
  .chip-id { font-family: monospace; font-size: 12px; color: #60a5fa; font-weight: 600; }
  .chip-name { font-size: 11px; color: #94a3b8; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .chip-badge {
    font-size: 9px; font-weight: 700; padding: 1px 5px;
    border-radius: 999px; text-transform: uppercase; font-family: monospace; flex-shrink: 0;
  }

  .unassigned { font-size: 12px; color: #475569; font-style: italic; }

  .slot-actions { display: flex; align-items: center; gap: 6px; flex-shrink: 0; }

  .btn-pick {
    background: #1e3a5f; border: 1px solid #2563eb;
    color: #93c5fd; border-radius: 6px; padding: 6px 14px;
    font-size: 12px; font-weight: 500; cursor: pointer;
    transition: all .15s; white-space: nowrap;
  }
  .btn-pick:hover, .btn-pick-active {
    background: #2563eb; color: #fff;
    box-shadow: 0 2px 8px rgba(37,99,235,.35);
  }

  .btn-clear-slot {
    background: none; border: 1px solid #334155;
    color: #64748b; border-radius: 6px; padding: 6px 8px;
    font-size: 11px; cursor: pointer; transition: all .15s;
  }
  .btn-clear-slot:hover { background: #7f1d1d; color: #fca5a5; border-color: #7f1d1d; }

  /* ── PICKER ── */
  .picker {
    margin: 0 18px 10px 36px;
    background: #0a101c;
    border: 1px solid #2563eb;
    border-radius: 8px; overflow: hidden;
    box-shadow: 0 8px 24px rgba(0,0,0,.5);
  }

  .picker-toolbar {
    display: flex; align-items: center; gap: 8px;
    padding: 8px 12px;
    background: #0f1d33;
    border-bottom: 1px solid #1e3a5f;
  }

  .picker-search {
    flex: 1; background: #0a1020; border: 1px solid #334155;
    border-radius: 5px; padding: 6px 10px; color: #e2e8f0;
    font-size: 12px; outline: none; transition: border .15s;
  }
  .picker-search:focus { border-color: #3b82f6; box-shadow: 0 0 0 2px rgba(59,130,246,.15); }

  .show-all-label {
    display: flex; align-items: center; gap: 5px;
    font-size: 11px; color: #64748b; cursor: pointer;
    white-space: nowrap; user-select: none;
  }
  .show-all-label input { cursor: pointer; }

  .btn-close-picker {
    background: #1e293b; border: 1px solid #334155; color: #94a3b8;
    border-radius: 5px; padding: 5px 8px; font-size: 11px;
    cursor: pointer; transition: all .15s;
  }
  .btn-close-picker:hover { background: #334155; color: #f1f5f9; }

  .picker-hint {
    padding: 5px 12px; font-size: 10px; color: #64748b;
    background: #0e1829; border-bottom: 1px solid #1e2a40;
  }
  .picker-hint strong { color: #60a5fa; }

  .picker-list { max-height: 240px; overflow-y: auto; }

  .picker-row {
    display: flex; align-items: center; gap: 8px;
    padding: 7px 12px; cursor: pointer;
    border-bottom: 1px solid #0f1825;
    transition: background .1s;
  }
  .picker-row:last-child { border-bottom: none; }
  .picker-row:hover { background: #1a2a42; }
  .picker-selected { background: #1e3a5f !important; outline: 2px solid #3b82f6; outline-offset: -2px; }

  .pr-id { font-family: monospace; font-size: 12px; color: #60a5fa; min-width: 100px; flex-shrink: 0; }
  .pr-name { font-size: 12px; color: #cbd5e1; flex-shrink: 0; }
  .pr-desc { font-size: 10px; color: #475569; flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .pr-badge {
    font-size: 9px; font-weight: 700; padding: 1px 5px;
    border-radius: 999px; text-transform: uppercase; font-family: monospace; flex-shrink: 0;
  }
  .pr-addr { font-size: 10px; color: #475569; font-family: monospace; flex-shrink: 0; }

  .picker-empty { padding: 18px; text-align: center; color: #475569; font-size: 12px; }
  .link-btn { background: none; border: none; color: #60a5fa; font-size: 12px; cursor: pointer; text-decoration: underline; padding: 0 4px; }

  /* ── BADGES ── */
  .badge-bool   { background: #064e3b; color: #6ee7b7; }
  .badge-float  { background: #1e3a8a; color: #93c5fd; }
  .badge-string { background: #4c1d95; color: #c4b5fd; }
  .badge-int    { background: #1e293b; color: #94a3b8; }
  .badge-mem    { background: #713f12; color: #fcd34d; }

  /* ── FOOTER ── */
  .footer {
    display: flex; align-items: center; justify-content: space-between;
    padding: 12px 18px;
    background: #111827; border-top: 1px solid #2d3f5a;
    flex-shrink: 0; gap: 12px;
  }
  .footer-info { font-size: 12px; color: #64748b; }
  .footer-info strong { color: #34d399; }
  .footer-btns { display: flex; gap: 8px; }

  .btn {
    padding: 8px 20px; border-radius: 6px; font-size: 13px;
    font-weight: 500; border: none; cursor: pointer; transition: all .15s;
  }
  .btn-sec { background: #1e293b; color: #94a3b8; border: 1px solid #334155; }
  .btn-sec:hover { background: #334155; color: #e2e8f0; }
  .btn-pri { background: #2563eb; color: #fff; font-weight: 600; box-shadow: 0 2px 8px rgba(37,99,235,.3); }
  .btn-pri:hover { background: #1d4ed8; box-shadow: 0 4px 12px rgba(37,99,235,.4); }
</style>
