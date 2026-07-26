import { writable, derived, get } from "svelte/store";
import type { ScadaProject } from "$lib/types";

export interface HistorySnapshot {
  project: ScadaProject;
  actionLabel: string;
  selectedFormId: string | null;
  selectedWidgetIds: string[];
  timestamp: number;
}

const MAX_STACK_DEPTH = 50;
const DEBOUNCE_WINDOW_MS = 600;

export const undoStack = writable<HistorySnapshot[]>([]);
export const redoStack = writable<HistorySnapshot[]>([]);

export const canUndo = derived(undoStack, ($stack) => $stack.length > 0);
export const canRedo = derived(redoStack, ($stack) => $stack.length > 0);

export const undoLabel = derived(undoStack, ($stack) => {
  if ($stack.length === 0) return "";
  return $stack[$stack.length - 1].actionLabel;
});

export const redoLabel = derived(redoStack, ($stack) => {
  if ($stack.length === 0) return "";
  return $stack[$stack.length - 1].actionLabel;
});

let lastRecordTime = 0;
let lastActionLabel = "";

/**
 * Record a snapshot into the undo stack before mutating project state.
 */
export function recordHistoryState(
  currentProject: ScadaProject | null,
  actionLabel: string,
  selectedFormId: string | null = null,
  selectedWidgetIds: string[] = [],
  forceNewStep = false,
) {
  if (!currentProject) return;

  const now = Date.now();
  const stack = get(undoStack);

  // Group continuous edits (e.g., continuous widget drag or slider tweaks) if within debounce window with same action
  const isContinuous =
    !forceNewStep &&
    stack.length > 0 &&
    actionLabel === lastActionLabel &&
    now - lastRecordTime < DEBOUNCE_WINDOW_MS;

  if (isContinuous) {
    // Update the timestamp of current entry, keep original snapshot before continuous interaction began
    lastRecordTime = now;
    return;
  }

  // Clear redo stack on new user action
  redoStack.set([]);

  const snapshot: HistorySnapshot = {
    project: structuredClone(currentProject),
    actionLabel,
    selectedFormId,
    selectedWidgetIds: [...selectedWidgetIds],
    timestamp: now,
  };

  undoStack.update((items) => {
    const next = [...items, snapshot];
    if (next.length > MAX_STACK_DEPTH) {
      return next.slice(next.length - MAX_STACK_DEPTH);
    }
    return next;
  });

  lastRecordTime = now;
  lastActionLabel = actionLabel;
}

export function performUndo(
  currentProject: ScadaProject | null,
  currentFormId: string | null,
  currentWidgetIds: string[],
): HistorySnapshot | null {
  const uStack = get(undoStack);
  if (uStack.length === 0 || !currentProject) return null;

  const targetSnapshot = uStack[uStack.length - 1];
  const nextUndoStack = uStack.slice(0, uStack.length - 1);

  // Save current state into redo stack
  const currentSnapshot: HistorySnapshot = {
    project: structuredClone(currentProject),
    actionLabel: targetSnapshot.actionLabel,
    selectedFormId: currentFormId,
    selectedWidgetIds: [...currentWidgetIds],
    timestamp: Date.now(),
  };

  redoStack.update((items) => [currentSnapshot, ...items].slice(0, MAX_STACK_DEPTH));
  undoStack.set(nextUndoStack);

  lastRecordTime = 0;
  lastActionLabel = "";

  return targetSnapshot;
}

export function performRedo(
  currentProject: ScadaProject | null,
  currentFormId: string | null,
  currentWidgetIds: string[],
): HistorySnapshot | null {
  const rStack = get(redoStack);
  if (rStack.length === 0 || !currentProject) return null;

  const targetSnapshot = rStack[0];
  const nextRedoStack = rStack.slice(1);

  // Save current state into undo stack
  const currentSnapshot: HistorySnapshot = {
    project: structuredClone(currentProject),
    actionLabel: targetSnapshot.actionLabel,
    selectedFormId: currentFormId,
    selectedWidgetIds: [...currentWidgetIds],
    timestamp: Date.now(),
  };

  undoStack.update((items) => [...items, currentSnapshot].slice(-MAX_STACK_DEPTH));
  redoStack.set(nextRedoStack);

  lastRecordTime = 0;
  lastActionLabel = "";

  return targetSnapshot;
}

export function clearHistory() {
  undoStack.set([]);
  redoStack.set([]);
  lastRecordTime = 0;
  lastActionLabel = "";
}
