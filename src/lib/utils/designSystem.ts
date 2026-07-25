import type {
  ProjectAnimationPreset,
  ProjectDesignSystem,
  ProjectFontToken,
  ProjectStyleClass,
} from "$lib/types";

export function defaultProjectDesignSystem(): ProjectDesignSystem {
  return {
    version: 1,
    fonts: [
      {
        id: "font-label",
        name: "Label",
        family: "Segoe UI",
        fallback: "system-ui, sans-serif",
        size: 11,
        weight: "600",
        lineHeight: 1.2,
      },
      {
        id: "font-value",
        name: "Process Value",
        family: "Segoe UI",
        fallback: "system-ui, sans-serif",
        size: 16,
        weight: "800",
        lineHeight: 1.1,
      },
      {
        id: "font-alarm",
        name: "Alarm",
        family: "Segoe UI",
        fallback: "system-ui, sans-serif",
        size: 11,
        weight: "800",
        lineHeight: 1.2,
      },
    ],
    styles: [
      {
        id: "style-default",
        name: "Default HMI",
        target: "*",
        surface: "#ffffff",
        text: "#1f2937",
        accent: "#2563eb",
        border: "#cbd5e1",
      },
      {
        id: "style-equipment",
        name: "Equipment",
        target: "process_symbol,faceplate",
        surface: "#f8fafc",
        text: "#334155",
        accent: "#2563eb",
        border: "#94a3b8",
      },
      {
        id: "style-high-contrast",
        name: "High Contrast",
        target: "*",
        surface: "#000000",
        text: "#ffffff",
        accent: "#00e5ff",
        border: "#ffffff",
      },
    ],
    animations: [
      { id: "anim-none", name: "None", kind: "none", durationMs: 0, easing: "linear" },
      { id: "anim-pulse", name: "Attention Pulse", kind: "pulse", durationMs: 1200, easing: "ease-in-out" },
      { id: "anim-rotate", name: "Running Rotation", kind: "rotate", durationMs: 1600, easing: "linear" },
      { id: "anim-fade", name: "Soft Fade", kind: "fade", durationMs: 1000, easing: "ease-in-out" },
      { id: "anim-slide", name: "Flow Slide", kind: "slide", durationMs: 1400, easing: "linear" },
    ],
  };
}

function uniqueById<T extends { id: string }>(items: T[], fallback: T[]): T[] {
  const seen = new Set<string>();
  const valid = items.filter((item) => {
    if (!item.id || seen.has(item.id)) return false;
    seen.add(item.id);
    return true;
  });
  return valid.length > 0 ? valid : fallback;
}

export function normalizeProjectDesignSystem(
  value: ProjectDesignSystem | null | undefined,
): ProjectDesignSystem {
  const defaults = defaultProjectDesignSystem();
  if (!value) return defaults;
  return {
    version: Number.isFinite(value.version) ? Math.max(1, value.version) : defaults.version,
    fonts: uniqueById<ProjectFontToken>(
      Array.isArray(value.fonts) ? value.fonts : [],
      defaults.fonts,
    ),
    styles: uniqueById<ProjectStyleClass>(
      Array.isArray(value.styles) ? value.styles : [],
      defaults.styles,
    ),
    animations: uniqueById<ProjectAnimationPreset>(
      Array.isArray(value.animations) ? value.animations : [],
      defaults.animations,
    ),
  };
}

