<script lang="ts">
  import type { TagValue, WidgetDef } from "$lib/types";
  import { configOf, qualityLabel, readBoolean, readString } from "$lib/components/widgets/shared/config";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }
  let { widget, tag = null, design = false }: Props = $props();
  const config = $derived(configOf(widget));
  const title = $derived(readString(config, "title", "Equipment help"));
  const text = $derived(readString(config, "text", "Operational guidance and diagnostic context."));
  const showQuality = $derived(readBoolean(config, "showQuality", false));
  const showAge = $derived(readBoolean(config, "showAge", false));
  const quality = $derived(tag?.quality ?? (design ? "uncertain" : undefined));
  const age = $derived(tag ? `${Math.max(0, Math.round(tag.age_ms))} ms` : design ? "12.4 s preview" : "");
</script>

<section class="tooltip" role="tooltip" aria-label={title}>
  <span class="arrow" aria-hidden="true"></span>
  <strong>ⓘ {title}</strong>
  <p>{text}</p>
  {#if showQuality || showAge || design}
    <div class="diagnostic">
      {#if showQuality || design}<span class="quality {quality ?? 'none'}">{quality === "good" ? "●" : quality === "uncertain" ? "◆" : "▲"} {qualityLabel(quality)}</span>{/if}
      {#if showAge || design}<span>Age: {age || "No data"}</span>{/if}
    </div>
  {/if}
</section>

<style>
  .tooltip { position:relative; width:100%; height:100%; min-height:54px; box-sizing:border-box; padding:7px 8px; border:1px solid #475569; border-radius:5px; background:#1e293b; color:#f8fafc; font:10px "Segoe UI",system-ui,sans-serif; box-shadow:0 3px 9px rgba(15,23,42,.22); } .arrow { position:absolute; top:-6px; left:14px; width:10px; height:10px; transform:rotate(45deg); border-top:1px solid #475569; border-left:1px solid #475569; background:#1e293b; } strong { position:relative; display:block; font-size:10px; } p { margin:3px 0 0; line-height:1.3; color:#dbeafe; } .diagnostic { display:flex; gap:6px; margin-top:5px; color:#cbd5e1; font-size:8px; } .quality { font-weight:800; } .quality.good { color:#86efac; } .quality.uncertain { color:#fde047; } .quality.bad { color:#fca5a5; background:repeating-linear-gradient(135deg,transparent,transparent 2px,rgba(255,255,255,.16) 2px,rgba(255,255,255,.16) 4px); }
</style>
