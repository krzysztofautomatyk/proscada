<script lang="ts">
  import type { ScadaProject } from "$lib/types";
  import { selectedFormId, mode } from "$lib/stores/app";

  interface Props {
    project: ScadaProject;
    design: boolean;
  }

  let { project, design }: Props = $props();
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

    <div class="tree-group">Forms</div>
    {#each project.forms as f}
      <div
        class="tree-item"
        class:active={$selectedFormId === f.id}
        role="button"
        tabindex="0"
        onclick={() => selectedFormId.set(f.id)}
        onkeydown={(e) => e.key === "Enter" && selectedFormId.set(f.id)}
      >
        <span>🗂</span>
        <span>{f.name}.form</span>
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
