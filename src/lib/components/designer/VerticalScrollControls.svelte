<script lang="ts">
  interface Props {
    target: HTMLElement | null;
    step?: number;
  }

  let { target, step = 240 }: Props = $props();
  let canScrollUp = $state(false);
  let canScrollDown = $state(false);

  function updateState() {
    if (!target) {
      canScrollUp = false;
      canScrollDown = false;
      return;
    }
    canScrollUp = target.scrollTop > 1;
    canScrollDown = target.scrollTop + target.clientHeight < target.scrollHeight - 1;
  }

  function scroll(direction: -1 | 1) {
    target?.scrollBy({ top: direction * step, behavior: "smooth" });
  }

  $effect(() => {
    const element = target;
    updateState();
    if (!element) return;
    const onScroll = () => updateState();
    const observer = new ResizeObserver(updateState);
    element.addEventListener("scroll", onScroll, { passive: true });
    observer.observe(element);
    if (element.firstElementChild) observer.observe(element.firstElementChild);
    return () => {
      element.removeEventListener("scroll", onScroll);
      observer.disconnect();
    };
  });
</script>

<div class="scroll-controls" aria-label="Vertical scroll controls">
  <button
    type="button"
    title="Przewiń w górę"
    aria-label="Przewiń panel w górę"
    disabled={!canScrollUp}
    onclick={() => scroll(-1)}
  >▲</button>
  <button
    type="button"
    title="Przewiń w dół"
    aria-label="Przewiń panel w dół"
    disabled={!canScrollDown}
    onclick={() => scroll(1)}
  >▼</button>
</div>

<style>
  .scroll-controls {
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }
  button {
    width: 22px;
    height: 20px;
    display: inline-grid;
    place-items: center;
    border: 1px solid var(--vs-border, #444);
    border-radius: 3px;
    background: var(--vs-bg-2, #252526);
    color: var(--vs-text, #ccc);
    padding: 0;
    font-size: 9px;
    cursor: pointer;
  }
  button:hover:not(:disabled) {
    border-color: #3b82f6;
    background: #2563eb;
    color: #fff;
  }
  button:focus-visible {
    outline: 2px solid #60a5fa;
    outline-offset: 1px;
  }
  button:disabled {
    opacity: 0.28;
    cursor: default;
  }
</style>

