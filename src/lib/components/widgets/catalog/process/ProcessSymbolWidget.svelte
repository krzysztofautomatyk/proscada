<script lang="ts">
  import type { TagValue, WidgetDef } from "$lib/types";
  import { configOf, readBoolean, readString } from "$lib/components/widgets/shared/config";

  interface Props {
    widget: WidgetDef;
    tag?: TagValue | null;
    design?: boolean;
    onWrite?: (tagId: string, value: number) => void;
  }
  let { widget, tag = null, design = false }: Props = $props();
  const config = $derived(configOf(widget));
  const variant = $derived(readString(config, "variant", "pump"));
  const label = $derived(readString(config, "label", widget.tag_id ?? "Equipment"));
  const quality = $derived(tag?.quality ?? "bad");
  const known = $derived(design || quality === "good");
  const running = $derived(known && !!tag && (tag.bool_value || tag.value !== 0));
  const fault = $derived(known && readBoolean(config, "fault", false));
  const local = $derived(readBoolean(config, "local", false));
  const stateText = $derived(
    known ? (fault ? "FAULT" : local ? "LOCAL" : running ? "RUNNING" : "STOPPED") : "NO DATA",
  );
</script>

<section class="symbol-card" class:fault class:running aria-label={`${label}: ${stateText}`}>
  <header><strong>{label}</strong><span class="quality {quality}">{quality === "good" ? "● GOOD" : quality === "uncertain" ? "◆ UNCERTAIN" : "▲ BAD"}</span></header>
  <div class="drawing" class:running class:fault>
    <svg viewBox="0 0 160 90" role="img" aria-label={`${variant} process symbol`}>
      {#if variant === "valve"}
        <path d="M20 45h38m44 0h38M58 25l22 20-22 20m44-40L80 45l22 20" class="line" /><circle cx="80" cy="18" r="9" class="detail" />
      {:else if variant === "motor"}
        <path d="M20 45h28m64 0h28M48 24h64v42H48z" class="shape" /><path d="M72 34v22m16-22v22" class="detail" />
      {:else if variant === "tank"}
        <path d="M48 12h64v66H48z" class="shape" /><path d="M49 58h62v19H49z" class="fill" />
      {:else if variant === "sensor"}
        <path d="M80 14a23 23 0 1 0 0 46a23 23 0 1 0 0-46M80 60v16m-14 0h28" class="line" /><path d="M80 37l13-8" class="detail" />
      {:else}
        <path d="M15 45h25m80 0h25M40 20h55v50H40z" class="shape" /><circle cx="95" cy="45" r="25" class="shape" /><path d="M95 28v34m-17-17h34" class="detail" />
      {/if}
    </svg>
  </div>
  <footer><span class="badge {fault ? 'fault' : running ? 'run' : ''}">{known ? (fault ? "▲ FAULT" : running ? "● RUNNING" : "○ STOPPED") : "▲ NO DATA"}</span>{#if local && known}<span class="badge local">◇ LOCAL</span>{/if}</footer>
</section>

<style>
  .symbol-card { width:100%; height:100%; box-sizing:border-box; display:flex; flex-direction:column; border:1px solid #cbd5e1; border-radius:6px; background:#fff; color:#334155; overflow:hidden; font:10px "Segoe UI",system-ui,sans-serif; } header { display:flex; align-items:center; justify-content:space-between; gap:5px; padding:5px 7px; background:#f8fafc; border-bottom:1px solid #e2e8f0; } header strong { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size:10px; } .quality { font-size:7px; font-weight:800; } .quality.good { color:#166534; } .quality.uncertain { color:#854d0e; } .quality.bad { color:#991b1b; background:repeating-linear-gradient(135deg,#fef2f2,#fef2f2 3px,#fee2e2 3px,#fee2e2 6px); } .drawing { min-height:0; flex:1; display:flex; align-items:center; justify-content:center; padding:4px; } svg { width:100%; height:100%; max-height:82px; } .shape,.line,.detail { fill:none; stroke:#475569; stroke-width:4; stroke-linecap:round; stroke-linejoin:round; } .fill { fill:#cbd5e1; } .drawing.running .fill { fill:#bbf7d0; } .drawing.running .shape,.drawing.running .line { stroke:#166534; } .drawing.fault .shape,.drawing.fault .line,.drawing.fault .detail { stroke:#991b1b; } footer { display:flex; gap:4px; padding:4px 6px; border-top:1px solid #e2e8f0; } .badge { padding:2px 4px; border:1px solid #94a3b8; border-radius:3px; color:#475569; font-size:8px; font-weight:800; } .badge.run { border-color:#15803d; color:#166534; } .badge.fault { border-color:#991b1b; color:#991b1b; background:repeating-linear-gradient(135deg,#fef2f2,#fef2f2 3px,#fee2e2 3px,#fee2e2 6px); } .badge.local { border-style:dashed; } @media (prefers-reduced-motion: reduce) { * { animation:none !important; transition:none !important; } }
</style>
