<script lang="ts">
  /**
   * Generic chrome for ANY widget: visibility + blink (+ design-time hints).
   * Runtime: optional On Click Script from project tree.
   */
  import type { WidgetDef } from "$lib/types";
  import { selectedFormId, tagMap } from "$lib/stores/app";
  import { isWidgetBlinking, isWidgetVisible } from "$lib/utils/dynamics";
  import { runScriptById } from "$lib/services/scriptRuntime";
  import { get } from "svelte/store";
  import type { Snippet } from "svelte";

  interface Props {
    widget: WidgetDef;
    design?: boolean;
    children: Snippet;
  }

  let { widget, design = false, children }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const blinking = $derived(isWidgetBlinking(cfg, $tagMap, widget.tag_id));
  const visible = $derived(isWidgetVisible(cfg, $tagMap, widget.tag_id));
  const blinkSpeedMs = $derived(Number(cfg.blinkSpeedMs ?? 600));
  const clickScriptId = $derived(String(cfg.onClickScriptId ?? ""));

  async function onShellClick(e: MouseEvent) {
    if (design || !clickScriptId) return;
    // Write buttons handle their own script + write path
    if (widget.widget_type === "write_button") return;
    e.stopPropagation();
    try {
      await runScriptById(clickScriptId, {
        type: "click",
        widgetId: widget.id,
        formId: get(selectedFormId),
        tagId: widget.tag_id ?? null,
      });
    } catch {
      /* logged */
    }
  }
</script>

{#if visible || design}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="dyn-shell"
    class:blinking
    class:hidden-design={!visible && design}
    class:locked={widget.locked}
    class:clickable={!design && !!clickScriptId && widget.widget_type !== "write_button"}
    style:--blink-speed="{blinkSpeedMs}ms"
    onclick={onShellClick}
  >
    {@render children()}
    {#if !visible && design}
      <span class="badge hide" title="Hidden in runtime">👁‍🗨</span>
    {/if}
    {#if widget.locked && design}
      <span class="badge lock" title="Locked">🔒</span>
    {/if}
  </div>
{/if}

<style>
  .dyn-shell {
    width: 100%;
    height: 100%;
    position: relative;
    box-sizing: border-box;
  }
  .clickable {
    cursor: pointer;
  }
  .blinking {
    animation: scada-blink var(--blink-speed, 600ms) infinite ease-in-out;
  }
  .hidden-design {
    opacity: 0.35 !important;
    outline: 1px dashed #ef4444 !important;
  }
  .badge {
    position: absolute;
    top: 2px;
    font-size: 10px;
    pointer-events: none;
    z-index: 5;
    line-height: 1;
  }
  .badge.hide {
    right: 2px;
  }
  .badge.lock {
    left: 2px;
  }
  @keyframes scada-blink {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.12;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .blinking {
      animation: none !important;
    }
  }
</style>
