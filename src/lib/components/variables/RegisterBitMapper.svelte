<script lang="ts">
  import type { RegisterBitEntry } from "$lib/types/registerMap";

  interface Props {
    address: number;
    bits: RegisterBitEntry[];
    onEditBitTag?: (bitIndex: number) => void;
  }

  let { address, bits, onEditBitTag }: Props = $props();
</script>

<div class="bit-mapper-box">
  <div class="bit-mapper-header">
    <span class="header-title">⚡ Bit Structure for Holding Register R{address} (bits 0..15, LSB = Bit 0)</span>
  </div>

  <div class="bits-grid">
    {#each bits as b}
      <div class="bit-cell" class:has-tag={!!b.tagId}>
        <div class="bit-top">
          <span class="bit-num">Bit {b.bitIndex}</span>
          <span class="bit-state">{b.state ? "1" : "0"}</span>
        </div>
        <div class="bit-bottom">
          <span class="bit-tag-name" title={b.tagName || b.description}>
            {b.tagName || "— rezerwa"}
          </span>
        </div>
        {#if onEditBitTag}
          <button
            type="button"
            class="bit-edit-btn"
            title="Skonfiguruj tag bitowy"
            onclick={() => onEditBitTag(b.bitIndex)}
          >
            ✏️
          </button>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .bit-mapper-box {
    background: #0d0d12;
    border: 1px solid #1e293b;
    border-radius: 6px;
    padding: 10px;
    margin: 6px 0;
  }

  .bit-mapper-header {
    margin-bottom: 8px;
    font-size: 11px;
    font-weight: 700;
    color: #38bdf8;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .bits-grid {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: 6px;
  }

  .bit-cell {
    background: #181820;
    border: 1px solid #272738;
    border-radius: 4px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    position: relative;
    transition: all 0.12s;
  }

  .bit-cell.has-tag {
    border-color: #22c55e;
    background: #0f2419;
  }

  .bit-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .bit-num {
    font-size: 10px;
    font-weight: 700;
    color: #94a3b8;
  }

  .bit-state {
    font-family: monospace;
    font-size: 10px;
    font-weight: 700;
    background: #272738;
    color: #4ade80;
    padding: 1px 4px;
    border-radius: 2px;
  }

  .bit-bottom {
    overflow: hidden;
  }

  .bit-tag-name {
    font-size: 9.5px;
    color: #cbd5e1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
  }

  .bit-edit-btn {
    position: absolute;
    top: 2px;
    right: 2px;
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 10px;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .bit-cell:hover .bit-edit-btn {
    opacity: 1;
  }

  @media (max-width: 768px) {
    .bits-grid {
      grid-template-columns: repeat(4, 1fr);
    }
  }
</style>
