<script lang="ts">
  import type { ScadaProject, TagDefinition } from "$lib/types";
  import { project, dirty, log } from "$lib/stores/app";
  import { uid } from "$lib/utils/projectTree";

  interface Props {
    scada: ScadaProject;
    design?: boolean;
  }

  let { scada, design = true }: Props = $props();

  let filter = $state("");

  const tags = $derived(
    scada.tags.filter((t) => {
      if (!filter.trim()) return true;
      const q = filter.toLowerCase();
      return (
        t.name.toLowerCase().includes(q) ||
        t.id.toLowerCase().includes(q) ||
        (t.description ?? "").toLowerCase().includes(q)
      );
    }),
  );

  function patchTag(id: string, patch: Partial<TagDefinition>) {
    project.update((p) => {
      if (!p) return p;
      dirty.set(true);
      return {
        ...p,
        tags: p.tags.map((t) => (t.id === id ? { ...t, ...patch } : t)),
      };
    });
  }

  function addTag() {
    const deviceId = scada.devices[0]?.id ?? "";
    const id = uid("tag");
    const tag: TagDefinition = {
      id,
      name: `TAG_${scada.tags.length + 1}`,
      device_id: deviceId,
      data_type: "u16",
      binding: { address: 0, table: "holding", writable: false },
      unit: "",
      description: "",
      scale: 1,
      offset: 0,
      decimals: 0,
    };
    project.update((p) => {
      if (!p) return p;
      dirty.set(true);
      return { ...p, tags: [...p.tags, tag] };
    });
    log(`Tag added: ${tag.name}`, "ok");
  }

  function removeTag(id: string) {
    if (!confirm(`Delete tag ${id}?`)) return;
    project.update((p) => {
      if (!p) return p;
      dirty.set(true);
      return { ...p, tags: p.tags.filter((t) => t.id !== id) };
    });
    log(`Tag deleted: ${id}`, "warn");
  }
</script>

<div class="vars">
  <div class="toolbar">
    <span class="title">🏷 Variables / Tags ({scada.tags.length})</span>
    <input class="filter" placeholder="Filter…" bind:value={filter} />
    <span class="spacer"></span>
    {#if design}
      <button type="button" class="primary" onclick={addTag}>+ Add Tag</button>
    {/if}
  </div>
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Name</th>
          <th>Id</th>
          <th>Type</th>
          <th>Addr</th>
          <th>Table</th>
          <th>W</th>
          <th>Unit</th>
          <th>Description</th>
          {#if design}<th></th>{/if}
        </tr>
      </thead>
      <tbody>
        {#each tags as t (t.id)}
          <tr>
            <td>
              {#if design}
                <input value={t.name} onchange={(e) => patchTag(t.id, { name: e.currentTarget.value })} />
              {:else}
                {t.name}
              {/if}
            </td>
            <td class="mono">{t.id}</td>
            <td>
              {#if design}
                <select
                  value={t.data_type}
                  onchange={(e) =>
                    patchTag(t.id, {
                      data_type: e.currentTarget.value as TagDefinition["data_type"],
                    })}
                >
                  <option value="bool">bool</option>
                  <option value="u16">u16</option>
                  <option value="i16">i16</option>
                  <option value="f32">f32</option>
                </select>
              {:else}
                {t.data_type}
              {/if}
            </td>
            <td>
              {#if design}
                <input
                  type="number"
                  class="addr"
                  value={t.binding.address}
                  onchange={(e) =>
                    patchTag(t.id, {
                      binding: { ...t.binding, address: Number(e.currentTarget.value) },
                    })}
                />
              {:else}
                {t.binding.address}
              {/if}
            </td>
            <td>{t.binding.table}</td>
            <td>{t.binding.writable ? "✓" : ""}</td>
            <td>
              {#if design}
                <input
                  class="unit"
                  value={t.unit}
                  onchange={(e) => patchTag(t.id, { unit: e.currentTarget.value })}
                />
              {:else}
                {t.unit}
              {/if}
            </td>
            <td>
              {#if design}
                <input
                  value={t.description}
                  onchange={(e) => patchTag(t.id, { description: e.currentTarget.value })}
                />
              {:else}
                {t.description}
              {/if}
            </td>
            {#if design}
              <td>
                <button type="button" class="del" onclick={() => removeTag(t.id)}>✕</button>
              </td>
            {/if}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .vars {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background: #1e1e1e;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--vs-border);
    background: var(--vs-bg-3);
  }
  .title {
    font-weight: 700;
    font-size: 12px;
    color: var(--vs-text-bright);
  }
  .filter {
    background: #3c3c3c;
    border: 1px solid var(--vs-border);
    color: #fff;
    font-size: 11px;
    padding: 3px 8px;
    border-radius: 3px;
    width: 160px;
  }
  .spacer {
    flex: 1;
  }
  .toolbar button.primary {
    background: var(--vs-accent-2);
    border: 1px solid var(--vs-accent);
    color: #fff;
    font-size: 11px;
    font-weight: 700;
    padding: 3px 10px;
    border-radius: 3px;
    cursor: pointer;
  }
  .table-wrap {
    flex: 1;
    overflow: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
  }
  th {
    position: sticky;
    top: 0;
    background: #2d2d30;
    text-align: left;
    padding: 6px 8px;
    color: var(--vs-text-dim);
    font-weight: 700;
    border-bottom: 1px solid var(--vs-border);
  }
  td {
    padding: 3px 6px;
    border-bottom: 1px solid #2a2a2a;
    vertical-align: middle;
  }
  td input,
  td select {
    width: 100%;
    background: #252526;
    border: 1px solid transparent;
    color: #ccc;
    font-size: 11px;
    padding: 2px 4px;
    border-radius: 2px;
  }
  td input:focus,
  td select:focus {
    border-color: var(--vs-accent);
    outline: none;
  }
  .addr {
    width: 64px !important;
  }
  .unit {
    width: 48px !important;
  }
  .mono {
    font-family: ui-monospace, monospace;
    color: #9cdcfe;
  }
  .del {
    background: transparent;
    border: none;
    color: #ef4444;
    cursor: pointer;
    font-weight: 800;
  }
</style>
