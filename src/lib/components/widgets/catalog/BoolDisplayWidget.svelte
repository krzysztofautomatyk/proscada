<script lang="ts">
  import type { WidgetDef, TagValue } from "$lib/types";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }

  let { widget, tag = null }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const str = (k: string, d = "") => String(cfg[k] ?? d);

  const isTrue = $derived(tag?.bool_value ?? false);
  const label = $derived(str("label", "BOOL STATUS"));
  const trueLabel = $derived(str("trueLabel", "TRUE"));
  const falseLabel = $derived(str("falseLabel", "FALSE"));
  const trueColor = $derived(str("trueColor", "#16A34A"));
  const falseColor = $derived(str("falseColor", "#9CA3AF"));
</script>

<div class="bool-card">
  <span class="bool-dot" style:background={isTrue ? trueColor : falseColor}></span>
  <span class="bool-title">{label}</span>
  <span
    class="bool-badge"
    style:background={isTrue ? trueColor : "#f3f4f6"}
    style:color={isTrue ? "#ffffff" : "#4b5563"}
  >
    {isTrue ? trueLabel : falseLabel}
  </span>
</div>

<style>
  .bool-card {
    width: 100%;
    height: 100%;
    background: #ffffff;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 6px 10px;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
  }
  .bool-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    box-shadow: 0 0 4px currentColor;
  }
  .bool-title {
    font-size: 11px;
    font-weight: 700;
    color: #1f2937;
    flex: 1;
  }
  .bool-badge {
    font-size: 9px;
    font-weight: 800;
    padding: 2px 6px;
    border-radius: 99px;
  }
</style>
