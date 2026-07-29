/**
 * Keyboard activation for elements that behave like buttons.
 *
 * Used together with `role="button"` and `tabindex="0"` so a mouse-only
 * affordance becomes operable from the keyboard instead of being silenced with
 * an `svelte-ignore` comment.
 */
export function activate(action: () => void) {
  return (event: KeyboardEvent) => {
    if (event.key !== "Enter" && event.key !== " " && event.key !== "Spacebar") return;
    event.preventDefault();
    action();
  };
}

/**
 * Keyboard resizing for a splitter handle.
 *
 * `step` is applied on arrow keys along the splitter's axis; Home/End are left
 * to the caller because sensible bounds differ per pane.
 */
export function resizeOnKey(
  orientation: "vertical" | "horizontal",
  apply: (delta: number) => void,
  step = 16,
) {
  return (event: KeyboardEvent) => {
    const decrease = orientation === "vertical" ? "ArrowLeft" : "ArrowUp";
    const increase = orientation === "vertical" ? "ArrowRight" : "ArrowDown";
    if (event.key === decrease) {
      event.preventDefault();
      apply(-step);
      return;
    }
    if (event.key === increase) {
      event.preventDefault();
      apply(step);
    }
  };
}
