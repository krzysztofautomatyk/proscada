<script lang="ts">
  import type { ScadaProject } from "$lib/types";
  import { selectedFormId, addNewForm, deleteForm, selectedWidgetId } from "$lib/stores/app";

  interface Props {
    project: ScadaProject;
    design: boolean;
  }

  let { project, design }: Props = $props();

  function selectForm(id: string) {
    selectedFormId.set(id);
    selectedWidgetId.set(null);
  }
</script>

<div class="panel" style:height="100%;border:none;border-right:1px solid var(--vs-border)">
  <div class="panel-header">Solution Explorer</div>
  <div class="panel-body">
    <div class="tree-group">Project</div>
    <div class="tree-item active">
      <span>📦</span>
      <span>{project.name}</span>
    </div>

    <div class="tree-group">Devices</div>
    {#each project.devices as d}
      <div class="tree-item">
        <span>🔌</span>
        <span>{d.name} · {d.host}:{d.port}</span>
      </div>
    {/each}

    <div class="tree-group">Tags ({project.tags.length})</div>
    {#each project.tags.slice(0, design ? 12 : 8) as t}
      <div class="tree-item" title={t.description}>
        <span>🏷</span>
        <span>{t.name}</span>
      </div>
    {/each}
    {#if project.tags.length > 12}
      <div class="tree-item" style:color="var(--vs-text-dim)">
        … +{project.tags.length - 12} more
      </div>
    {/if}

    <div class="tree-group form-header">
      <span>Forms / Screens ({project.forms.length})</span>
      {#if design}
        <button
          class="btn-add-form"
          title="Add New Screen"
          onclick={() => addNewForm()}
        >
          + Add
        </button>
      {/if}
    </div>
    {#each project.forms as f}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="tree-item form-item"
        class:active={$selectedFormId === f.id}
        role="button"
        tabindex="0"
        onclick={() => selectForm(f.id)}
        onkeydown={(e) => e.key === "Enter" && selectForm(f.id)}
      >
        <span>🗂</span>
        <span class="form-title">{f.name}.form</span>
        {#if design && project.forms.length > 1}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <button
            class="btn-del-form"
            title="Delete screen {f.name}"
            onclick={(e) => {
              e.stopPropagation();
              deleteForm(f.id);
            }}
          >
            ✕
          </button>
        {/if}
      </div>
    {/each}

    <div class="tree-group">Alarms</div>
    {#each project.alarms as a}
      <div class="tree-item">
        <span>🔔</span>
        <span>{a.name}</span>
      </div>
    {/each}

    {#if design}
      <div class="tree-group">Mode</div>
      <div class="tree-item">
        <span>🛠</span>
        <span>Designer (Engineering)</span>
      </div>
    {:else}
      <div class="tree-group">Mode</div>
      <div class="tree-item">
        <span>▶</span>
        <span>Runtime (Operator)</span>
      </div>
    {/if}

    <div style:padding="10px" style:font-size="10px" style:color="var(--vs-text-dim)">
      Schema v{project.schema_version}<br />
      Hash: {project.content_hash.slice(0, 12)}…
    </div>
  </div>
</div>

<style>
  .form-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .btn-add-form {
    background: #1e3a8a;
    color: #ffffff;
    border: none;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 800;
    padding: 1px 6px;
    cursor: pointer;
  }
  .btn-add-form:hover {
    background: #2563eb;
  }
  .form-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .form-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-left: 4px;
  }
  .btn-del-form {
    background: transparent;
    border: none;
    color: #ef4444;
    font-size: 11px;
    font-weight: 800;
    cursor: pointer;
    opacity: 0.6;
    padding: 0 4px;
  }
  .btn-del-form:hover {
    opacity: 1;
    color: #dc2626;
  }
</style>
