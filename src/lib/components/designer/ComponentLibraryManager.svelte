<script lang="ts">
  import { project } from "$lib/stores/app";
  import {
    bulkInstantiateComponentTemplate,
    createComponentTemplateFromSelection,
    deleteComponentTemplate,
    exportComponentTemplate,
    importComponentTemplateFile,
    installPumpStationTemplate,
    instantiateComponentTemplate,
  } from "$lib/stores/app";

  let selectedId = $state("");
  const templates = $derived($project?.component_templates ?? []);
  const selected = $derived(templates.find((template) => template.id === selectedId) ?? templates[0]);

  $effect(() => {
    if (!selectedId && templates[0]) selectedId = templates[0].id;
    if (selectedId && !templates.some((template) => template.id === selectedId)) {
      selectedId = templates[0]?.id ?? "";
    }
  });

  function createFromSelection() {
    const name = prompt("Component name", "Custom Component");
    if (!name) return;
    const category = prompt("Toolbox category", "Custom") ?? "Custom";
    const id = createComponentTemplateFromSelection(name, category);
    if (id) selectedId = id;
  }

  async function bulkFromFile() {
    if (!selected) return;
    const { openTextFile } = await import("$lib/services/fileIo");
    const picked = await openTextFile([{ name: "Object list", extensions: ["csv"] }]);
    if (!picked) return;
    try {
      const count = bulkInstantiateComponentTemplate(selected.id, picked.text);
      alert(`Created ${count} component instances.`);
    } catch (error) {
      alert(error instanceof Error ? error.message : "Bulk import failed");
    }
  }

  function createTwentyPumpStations() {
    const templateId = installPumpStationTemplate();
    selectedId = templateId;
    const deviceId = $project?.devices[0]?.id ?? "";
    if (!deviceId) {
      alert("Add a Modbus device before generating pump stations.");
      return;
    }
    const rows = ["objectId,name,tagPrefix,alarmGroup,location,deviceId,baseAddress"];
    for (let index = 1; index <= 20; index++) {
      const n = String(index).padStart(3, "0");
      rows.push(`PS_${n},Pompownia ${n},PLC.PS${n},ZakladA/Pompownie/PS_${n},Strefa ${Math.ceil(index / 4)},${deviceId},${1000 + (index - 1) * 16}`);
    }
    try {
      const count = bulkInstantiateComponentTemplate(templateId, rows.join("\n"));
      alert(`Created ${count} pump-station instances.`);
    } catch (error) {
      alert(error instanceof Error ? error.message : "Pump-station generation failed");
    }
  }
</script>

<div class="manager">
  <header><strong>Component Library</strong><span>Reusable, exportable composite controls</span></header>
  <div class="actions">
    <button type="button" onclick={createFromSelection}>+ From selection</button>
    <button type="button" onclick={() => { selectedId = installPumpStationTemplate(); }}>Install 2P+2F+1S</button>
    <button type="button" onclick={() => void importComponentTemplateFile()}>Import .pscctrl</button>
  </div>
  <div class="template-list">
    {#each templates as template (template.id)}
      <button type="button" class:selected={selected?.id === template.id} onclick={() => (selectedId = template.id)}>
        <strong>{template.name}</strong>
        <span>{template.category} · v{template.version} · {template.widgets.length} widgets</span>
      </button>
    {:else}
      <p>No custom components. Select widgets and create a template.</p>
    {/each}
  </div>
  {#if selected}
    <section>
      <h3>{selected.name}</h3>
      <p>{selected.description}</p>
      <dl>
        <dt>Definition</dt><dd>{selected.id}</dd>
        <dt>Geometry</dt><dd>{selected.width} × {selected.height}</dd>
        <dt>Parameters</dt><dd>{selected.parameter_names.join(", ")}</dd>
      </dl>
      <div class="selected-actions">
        <button type="button" onclick={() => instantiateComponentTemplate(selected.id)}>Add instance</button>
        <button type="button" onclick={() => void bulkFromFile()}>Bulk CSV…</button>
        <button type="button" onclick={() => void exportComponentTemplate(selected.id)}>Export</button>
        <button type="button" class="delete" onclick={() => { if (confirm(`Delete ${selected.name}?`)) deleteComponentTemplate(selected.id); }}>Delete</button>
      </div>
    </section>
  {/if}
  <section class="pump">
    <h3>Reference rollout</h3>
    <p>Creates 20 grouped instances of the wastewater pump-station template with unique objectId, tagPrefix and alarmGroup.</p>
    <button type="button" onclick={createTwentyPumpStations}>Generate 20 pump stations</button>
  </section>
  <footer>Imports reject scripts, javascript: URLs, unknown widget types, duplicate IDs and invalid geometry.</footer>
</div>

<style>
  .manager { height: 100%; overflow: auto; background: var(--vs-panel, #252526); color: #ccc; font-size: 10px; }
  header { padding: 9px; border-bottom: 1px solid #444; background: #202020; } header strong, header span { display: block; } header strong { color: #fff; font-size: 11px; } header span { margin-top: 2px; color: #9ca3af; font-size: 9px; }
  .actions, .selected-actions { display: grid; grid-template-columns: 1fr 1fr; gap: 5px; padding: 7px; }
  button { min-height: 28px; border: 1px solid #555; border-radius: 4px; background: #333337; color: #ddd; font: inherit; cursor: pointer; } button:hover { border-color: #3b82f6; background: #37373d; }
  .template-list { display: grid; gap: 4px; padding: 7px; } .template-list button { display: grid; gap: 2px; padding: 6px; text-align: left; } .template-list button.selected { border-color: #3b82f6; background: #1e3a5f; } .template-list strong { color: #fff; } .template-list span { color: #9ca3af; font-size: 8px; }
  section { margin: 7px; padding: 8px; border: 1px solid #444; border-radius: 5px; background: #2d2d30; } h3 { margin: 0 0 5px; color: #93c5fd; font-size: 10px; } p { margin: 4px 0; color: #aaa; line-height: 1.4; } dl { display: grid; grid-template-columns: 62px 1fr; gap: 3px; } dt { color: #8b949e; } dd { min-width: 0; margin: 0; overflow-wrap: anywhere; color: #ddd; }
  .selected-actions { padding: 6px 0 0; } .delete { color: #fca5a5; } .pump button { width: 100%; margin-top: 6px; border-color: #7c3aed; color: #ddd6fe; }
  footer { padding: 8px; color: #8b949e; font-size: 8px; line-height: 1.4; }
</style>
