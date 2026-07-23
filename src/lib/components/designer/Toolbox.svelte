<script lang="ts">
  import { WIDGET_CATALOG } from "$lib/types";

  const categories = [...new Set(WIDGET_CATALOG.map((w) => w.category))];

  function onDragStart(e: DragEvent, type: string) {
    e.dataTransfer?.setData("application/x-proscada-widget", type);
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "copy";
  }
</script>

<div class="panel" style:height="100%;border:none">
  <div class="panel-header">Toolbox</div>
  <div class="panel-body">
    {#each categories as cat}
      <div class="tree-group">{cat}</div>
      {#each WIDGET_CATALOG.filter((w) => w.category === cat) as item}
        <div
          class="toolbox-item"
          draggable="true"
          ondragstart={(e) => onDragStart(e, item.type)}
          role="listitem"
        >
          <span class="icon">{item.icon}</span>
          <span>{item.label}</span>
        </div>
      {/each}
    {/each}
  </div>
</div>
