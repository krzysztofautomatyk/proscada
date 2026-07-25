<script lang="ts">
  interface Props {
    title: string;
    detail?: string;
    icon?: string;
  }
  let { title, detail = "", icon = "◇" }: Props = $props();
</script>

<div class="empty">
  <span class="icon" aria-hidden="true">{icon}</span>
  <strong>{title}</strong>
  {#if detail}<span>{detail}</span>{/if}
</div>

<style>
  .empty {
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 10px;
    color: #64748b;
    text-align: center;
  }
  .icon {
    color: #3b82f6;
    font-size: 22px;
  }
  strong {
    color: #334155;
    font-size: 11px;
  }
  span:not(.icon) {
    max-width: 24rem;
    font-size: 9px;
  }
</style>

