<script lang="ts">
  import type { TagValue } from "$lib/types";
  import { qualityLabel } from "./config";

  interface Props {
    tag?: TagValue | null;
    showAge?: boolean;
  }

  let { tag = null, showAge = false }: Props = $props();
  const quality = $derived(tag?.quality);
  const label = $derived(qualityLabel(quality));
  const age = $derived(tag ? `${Math.max(0, Math.round(tag.age_ms))} ms` : "");
</script>

<span class="quality" class:good={quality === "good"} class:uncertain={quality === "uncertain"} class:bad={quality === "bad"}>
  <span class="symbol" aria-hidden="true">{quality === "good" ? "●" : quality === "uncertain" ? "◆" : "▲"}</span>
  {label}{showAge && age ? ` · ${age}` : ""}
</span>

<style>
  .quality {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    border: 1px solid #94a3b8;
    border-radius: 999px;
    padding: 2px 6px;
    color: #475569;
    background: #f8fafc;
    font-size: 9px;
    font-weight: 800;
    letter-spacing: 0.04em;
    white-space: nowrap;
  }
  .good {
    border-color: #86efac;
    color: #166534;
    background: #f0fdf4;
  }
  .uncertain {
    border-color: #fde047;
    color: #854d0e;
    background: #fefce8;
  }
  .bad {
    border-color: #fca5a5;
    color: #991b1b;
    background: repeating-linear-gradient(135deg, #fef2f2, #fef2f2 4px, #fee2e2 4px, #fee2e2 8px);
  }
  .symbol {
    font-size: 8px;
  }
</style>

