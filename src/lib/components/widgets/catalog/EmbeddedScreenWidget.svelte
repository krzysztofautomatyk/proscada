<script lang="ts">
  import type { WidgetDef, TagValue, FormDef } from "$lib/types";
  import { project } from "$lib/stores/app";
  import WidgetView from "../WidgetView.svelte";

  interface Props {
    widget: WidgetDef;
    tagMap: Map<string, TagValue>;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
    ancestorFormIds?: Set<string>;
  }

  let {
    widget,
    tagMap,
    design = false,
    onWrite,
    ancestorFormIds = new Set<string>(),
  }: Props = $props();

  const cfg = $derived((widget.config ?? {}) as Record<string, unknown>);
  const targetFormId = $derived(String(cfg.target_form_id ?? ""));
  const tagPrefix = $derived(String(cfg.tag_prefix ?? ""));
  const scaleMode = $derived(String(cfg.scale_mode ?? "fit"));
  const tagOverrides = $derived((cfg.tag_overrides ?? {}) as Record<string, string>);

  const bgColor = $derived(String(cfg.bgColor ?? "transparent"));
  const borderColor = $derived(String(cfg.borderColor ?? "#9ca3af"));
  const borderWidth = $derived(Number(cfg.borderWidth ?? 1));
  const borderStyle = $derived(String(cfg.borderStyle ?? "dashed"));

  const targetForm = $derived<FormDef | null>(
    $project?.forms.find((f) => f.id === targetFormId) ?? null,
  );

  const isCircular = $derived(
    Boolean(targetFormId && ancestorFormIds.has(targetFormId)),
  );

  const nextAncestors = $derived.by(() => {
    const set = new Set(ancestorFormIds);
    if (targetFormId) set.add(targetFormId);
    return set;
  });

  function resolveInnerTagId(rawTagId: string | null | undefined): string | null {
    if (!rawTagId) return null;
    if (tagOverrides && tagOverrides[rawTagId]) {
      return tagOverrides[rawTagId];
    }
    if (rawTagId.includes("{PREFIX}") || rawTagId.includes("{prefix}")) {
      return rawTagId.replaceAll("{PREFIX}", tagPrefix).replaceAll("{prefix}", tagPrefix);
    }
    if (tagPrefix) {
      return tagPrefix + rawTagId;
    }
    return rawTagId;
  }

  function getInnerTag(rawTagId: string | null | undefined): TagValue | null {
    const resolved = resolveInnerTagId(rawTagId);
    return resolved ? tagMap.get(resolved) ?? null : null;
  }

  function innerOnWrite(rawTagId: string, value: number) {
    const resolved = resolveInnerTagId(rawTagId);
    if (resolved && onWrite) {
      onWrite(resolved, value);
    }
  }

  const nativeW = $derived(targetForm?.width ?? 800);
  const nativeH = $derived(targetForm?.height ?? 600);

  const scaleX = $derived(widget.w / (nativeW || 1));
  const scaleY = $derived(widget.h / (nativeH || 1));
  const scaleFit = $derived(Math.min(scaleX, scaleY));

  function sortedWidgets(widgets: WidgetDef[]): WidgetDef[] {
    return [...widgets].sort((a, b) => (a.z ?? 0) - (b.z ?? 0));
  }
</script>

<div
  class="embedded-container"
  class:design-mode={design}
  style:background={bgColor}
  style:border="{borderWidth}px {borderStyle} {borderColor}"
>
  {#if !targetFormId}
    <div class="placeholder empty">
      <span class="icon">🔲</span>
      <span class="title">Embedded Screen / Faceplate</span>
      <span class="hint">Select target screen in Properties</span>
    </div>
  {:else if isCircular}
    <div class="placeholder error">
      <span class="icon">⚠️</span>
      <span class="title">Circular Reference</span>
      <span class="hint">Screen "{targetForm?.name ?? targetFormId}" embeds itself</span>
    </div>
  {:else if !targetForm}
    <div class="placeholder error">
      <span class="icon">⚠️</span>
      <span class="title">Target Screen Missing</span>
      <span class="hint">ID: {targetFormId}</span>
    </div>
  {:else}
    <!-- Master Screen Content Viewport -->
    <div class="viewport mode-{scaleMode}">
      <div
        class="canvas-frame"
        style:width="{nativeW}px"
        style:height="{nativeH}px"
        style:background={targetForm.background || "transparent"}
        style:transform={scaleMode === "fit"
          ? `scale(${scaleFit})`
          : scaleMode === "stretch"
            ? `scale(${scaleX}, ${scaleY})`
            : "none"}
      >
        {#each sortedWidgets(targetForm.widgets) as innerW (innerW.id)}
          {@const innerTag = getInnerTag(innerW.tag_id)}
          <div
            class="inner-widget-pos"
            style:left="{innerW.x}px"
            style:top="{innerW.y}px"
            style:width="{innerW.w}px"
            style:height="{innerW.h}px"
            style:z-index={innerW.z ?? 1}
          >
            <WidgetView
              widget={innerW}
              tag={innerTag}
              {design}
              onWrite={innerOnWrite}
              ancestorFormIds={nextAncestors}
            />
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .embedded-container {
    width: 100%;
    height: 100%;
    position: relative;
    box-sizing: border-box;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  .placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 12px;
    box-sizing: border-box;
    text-align: center;
    gap: 4px;
    user-select: none;
  }
  .placeholder.empty {
    background: rgba(243, 244, 246, 0.6);
    color: #4b5563;
  }
  .placeholder.error {
    background: #fef2f2;
    color: #991b1b;
  }
  .placeholder .icon {
    font-size: 24px;
    line-height: 1;
  }
  .placeholder .title {
    font-size: 12px;
    font-weight: 700;
  }
  .placeholder .hint {
    font-size: 10px;
    opacity: 0.8;
  }

  .viewport {
    width: 100%;
    height: 100%;
    position: relative;
  }
  .viewport.mode-clip {
    overflow: hidden;
  }
  .viewport.mode-scroll {
    overflow: auto;
  }
  .viewport.mode-fit,
  .viewport.mode-stretch {
    overflow: hidden;
  }

  .canvas-frame {
    position: absolute;
    top: 0;
    left: 0;
    transform-origin: top left;
    box-sizing: border-box;
  }

  .inner-widget-pos {
    position: absolute;
    box-sizing: border-box;
  }
</style>
