<script lang="ts">
  /**
   * Generic chrome for ANY widget: visibility + blink (+ design-time hints).
   * Runtime: optional On Click Script from project tree.
   */
  import type { WidgetDef } from "$lib/types";
  import { project, selectedFormId, snapshot, tagMap } from "$lib/stores/app";
  import {
    isWidgetAnimating,
    isWidgetBlinking,
    isWidgetVisible,
  } from "$lib/utils/dynamics";
  import { normalizeProjectDesignSystem } from "$lib/utils/designSystem";
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
  const customAnimating = $derived(isWidgetAnimating(cfg, $tagMap, widget.tag_id));
  const visible = $derived(isWidgetVisible(cfg, $tagMap, widget.tag_id));
  const currentSecurityLevel = $derived($snapshot?.security_level ?? 1000);
  const minLevel = $derived(widget.min_level ?? 0);
  const isUnauthorized = $derived(!design && minLevel > 0 && currentSecurityLevel < minLevel);
  const behavior = $derived(widget.unauthorized_behavior || "disabled");
  const isHidden = $derived(!visible || (isUnauthorized && behavior === "hidden"));

  const blinkSpeedMs = $derived(Math.max(500, Number(cfg.blinkSpeedMs ?? 600)));
  const clickScriptId = $derived(String(cfg.onClickScriptId ?? ""));
  const designSystem = $derived(normalizeProjectDesignSystem($project?.design_system));
  const styleClass = $derived(
    designSystem.styles.find((style) => style.id === String(cfg.styleClassId ?? "style-default")) ??
      designSystem.styles[0],
  );
  const fontToken = $derived(
    cfg.fontTokenId && cfg.fontTokenId !== "none"
      ? designSystem.fonts.find((font) => font.id === String(cfg.fontTokenId))
      : undefined,
  );
  const animationPreset = $derived(
    designSystem.animations.find(
      (animation) => animation.id === String(cfg.animationPresetId ?? "anim-none"),
    ) ?? designSystem.animations[0],
  );
  const animationDurationMs = $derived(
    Math.max(500, Number(animationPreset?.durationMs ?? 1000)),
  );
  const ownsInteraction = $derived(
    [
      "write_button",
      "command_button",
      "numeric_input",
      "boolean_input",
      "select_input",
      "text_input",
      "datetime_range",
      "faceplate",
    ].includes(widget.widget_type),
  );

  async function onShellClick(e: MouseEvent) {
    if (design) return;
    if (isUnauthorized) {
      if (behavior === "prompt_login") {
        e.stopPropagation();
        window.dispatchEvent(new CustomEvent("proscada:open-login"));
      }
      return;
    }
    if (!clickScriptId || ownsInteraction) return;
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

{#if !isHidden || design}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="dyn-shell"
    class:blinking
    class:hidden-design={(!visible || isUnauthorized) && design}
    class:unauthorized-disabled={isUnauthorized && behavior === "disabled"}
    class:unauthorized-prompt={isUnauthorized && behavior === "prompt_login"}
    class:locked={widget.locked}
    class:clickable={!design && ((!!clickScriptId && !ownsInteraction) || (isUnauthorized && behavior === "prompt_login"))}
    class:anim-pulse={customAnimating && animationPreset?.kind === "pulse"}
    class:anim-rotate={customAnimating && animationPreset?.kind === "rotate"}
    class:anim-fade={customAnimating && animationPreset?.kind === "fade"}
    class:anim-slide={customAnimating && animationPreset?.kind === "slide"}
    style:--blink-speed="{blinkSpeedMs}ms"
    style:--custom-animation-duration="{animationDurationMs}ms"
    style:--custom-animation-easing={animationPreset?.easing ?? "linear"}
    style:--psc-font-family={`${fontToken?.family ?? "Segoe UI"}, ${fontToken?.fallback ?? "system-ui, sans-serif"}`}
    style:--psc-font-size={`${fontToken?.size ?? 11}px`}
    style:--psc-font-weight={fontToken?.weight ?? "600"}
    style:--psc-surface={styleClass?.surface ?? "#ffffff"}
    style:--psc-text={styleClass?.text ?? "#1f2937"}
    style:--psc-accent={styleClass?.accent ?? "#2563eb"}
    style:--psc-border={styleClass?.border ?? "#cbd5e1"}
    style:font-family={fontToken ? "var(--psc-font-family)" : undefined}
    style:color="var(--psc-text)"
    onclick={onShellClick}
  >
    {@render children()}

    {#if isUnauthorized && behavior === "disabled"}
      <div class="unauthorized-overlay" title={`Wymagany poziom uprawnień: L${minLevel}`}>
        <span class="badge security-lock">🔒 L{minLevel}</span>
      </div>
    {/if}

    {#if minLevel > 0 && design}
      <span class="badge security-spec" title={`Minimum Security Level: ${minLevel}`}>🛡 L{minLevel}</span>
    {/if}

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
  .anim-pulse {
    animation: scada-custom-pulse var(--custom-animation-duration, 1200ms)
      var(--custom-animation-easing, ease-in-out) infinite;
  }
  .anim-rotate {
    animation: scada-custom-rotate var(--custom-animation-duration, 1600ms)
      var(--custom-animation-easing, linear) infinite;
  }
  .anim-fade {
    animation: scada-custom-fade var(--custom-animation-duration, 1000ms)
      var(--custom-animation-easing, ease-in-out) infinite alternate;
  }
  .anim-slide {
    animation: scada-custom-slide var(--custom-animation-duration, 1400ms)
      var(--custom-animation-easing, linear) infinite alternate;
  }
  .hidden-design {
    opacity: 0.35 !important;
    outline: 1px dashed #ef4444 !important;
  }
  .unauthorized-disabled {
    pointer-events: none !important;
    filter: grayscale(0.9) opacity(0.5);
  }
  .unauthorized-overlay {
    position: absolute;
    inset: 0;
    background: rgba(15, 23, 42, 0.4);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
    pointer-events: none;
    border-radius: inherit;
  }
  .badge {
    position: absolute;
    top: 2px;
    font-size: 10px;
    pointer-events: none;
    z-index: 12;
    line-height: 1;
  }
  .badge.hide {
    right: 2px;
  }
  .badge.lock {
    left: 2px;
  }
  .badge.security-lock {
    top: auto;
    bottom: 2px;
    right: 2px;
    background: #0f172a;
    color: #f43f5e;
    border: 1px solid rgba(244, 63, 94, 0.4);
    padding: 2px 5px;
    border-radius: 4px;
    font-weight: 700;
  }
  .badge.security-spec {
    top: auto;
    bottom: 2px;
    left: 2px;
    background: #0f172a;
    color: #38bdf8;
    border: 1px solid rgba(56, 189, 248, 0.4);
    padding: 2px 5px;
    border-radius: 4px;
    font-weight: 700;
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
  @keyframes scada-custom-pulse {
    0%,
    100% {
      filter: brightness(1);
    }
    50% {
      filter: brightness(1.22);
    }
  }
  @keyframes scada-custom-rotate {
    to {
      transform: rotate(360deg);
    }
  }
  @keyframes scada-custom-fade {
    from {
      opacity: 0.55;
    }
    to {
      opacity: 1;
    }
  }
  @keyframes scada-custom-slide {
    from {
      transform: translateX(-3px);
    }
    to {
      transform: translateX(3px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .blinking,
    .anim-pulse,
    .anim-rotate,
    .anim-fade,
    .anim-slide {
      animation: none !important;
    }
  }
</style>
