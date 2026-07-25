<script lang="ts">
  /**
   * Generic condition editor for blink / marquee / visibility.
   * Shared by Label and every future widget PropertyGrid section.
   */
  import type { TagDefinition } from "$lib/types";
  import { CONDITION_MODE_OPTIONS, type ConditionMode } from "$lib/utils/dynamics";

  interface Props {
    title: string;
    mode: string;
    tagId: string;
    bit: number;
    val: number;
    tags: TagDefinition[];
    /** Extra rows after condition (speed etc.) */
    showBit?: boolean;
    showVal?: boolean;
    onMode: (v: string) => void;
    onTag: (v: string) => void;
    onBit: (v: number) => void;
    onVal: (v: number) => void;
  }

  let {
    title,
    mode,
    tagId,
    bit,
    val,
    tags,
    showBit = true,
    showVal = true,
    onMode,
    onTag,
    onBit,
    onVal,
  }: Props = $props();

  const needsTag = $derived(mode !== "none" && mode !== "always");
  const needsBit = $derived(mode === "tag_bit");
  const needsVal = $derived(
    mode === "tag_val_eq" ||
      mode === "tag_val_gt" ||
      mode === "tag_val_lt" ||
      mode === "tag_val_neq",
  );
</script>

<table class="props-table cond-table">
  <thead>
    <tr><th colspan="2">{title}</th></tr>
  </thead>
  <tbody>
    <tr>
      <td>Condition</td>
      <td>
        <select value={mode} onchange={(e) => onMode(e.currentTarget.value)}>
          {#each CONDITION_MODE_OPTIONS as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </td>
    </tr>
    {#if needsTag}
      <tr>
        <td>Tag / Register</td>
        <td>
          <select value={tagId} onchange={(e) => onTag(e.currentTarget.value)}>
            <option value="">(widget Tag)</option>
            {#each tags as t}
              <option value={t.id}>{t.name}</option>
            {/each}
          </select>
        </td>
      </tr>
    {/if}
    {#if needsBit && showBit}
      <tr>
        <td>Bit index (0–15)</td>
        <td>
          <input
            type="number"
            min="0"
            max="15"
            value={bit}
            onchange={(e) => onBit(Number(e.currentTarget.value))}
          />
        </td>
      </tr>
    {/if}
    {#if needsVal && showVal}
      <tr>
        <td>Compare value</td>
        <td>
          <input
            type="number"
            value={val}
            onchange={(e) => onVal(Number(e.currentTarget.value))}
          />
        </td>
      </tr>
    {/if}
  </tbody>
</table>

<style>
  .cond-table :global(td:first-child) {
    width: 42%;
    color: var(--vs-text-dim, #9d9d9d);
  }
</style>
