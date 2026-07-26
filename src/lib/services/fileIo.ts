import { save, open } from "@tauri-apps/plugin-dialog";
import { writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";
import type { ScadaProject } from "$lib/types";

const isTauri = () =>
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

/** Native save dialog on macOS/Tauri; browser download fallback for `npm run dev`. */
export async function saveTextFile(
  defaultName: string,
  contents: string,
  filters: { name: string; extensions: string[] }[] = [
    { name: "ProScada Project", extensions: ["json"] },
  ],
): Promise<string | null> {
  if (isTauri()) {
    const path = await save({
      title: "Export ProScada Project",
      defaultPath: defaultName,
      filters,
    });
    if (!path) return null;
    await writeTextFile(path, contents);
    return path;
  }

  // Browser / Vite-only fallback
  if (typeof document === "undefined") {
    return defaultName;
  }

  const blob = new Blob([contents], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = defaultName;
  a.style.display = "none";
  document.body.appendChild(a);
  a.click();
  // delay revoke so Safari/Chrome finish the download handoff
  setTimeout(() => {
    URL.revokeObjectURL(url);
    a.remove();
  }, 1500);
  return defaultName;
}

/** Native open dialog on macOS/Tauri; file input fallback for browser. */
export async function openTextFile(
  filters: { name: string; extensions: string[] }[] = [
    { name: "ProScada Project", extensions: ["json"] },
  ],
): Promise<{ path: string; text: string } | null> {
  if (isTauri()) {
    const selected = await open({
      title: "Import ProScada Project",
      multiple: false,
      directory: false,
      filters,
    });
    if (!selected || Array.isArray(selected)) return null;
    const text = await readTextFile(selected);
    return { path: selected, text };
  }

  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = [
      ...filters.flatMap((filter) => filter.extensions.map((extension) => `.${extension}`)),
      "application/json",
      "text/csv",
    ].join(",");
    input.style.display = "none";
    input.onchange = async () => {
      const file = input.files?.[0];
      input.remove();
      if (!file) {
        resolve(null);
        return;
      }
      resolve({ path: file.name, text: await file.text() });
    };
    input.oncancel = () => {
      input.remove();
      resolve(null);
    };
    document.body.appendChild(input);
    input.click();
  });
}

/** Direct write to known file path without prompting dialog (Tauri mode). */
export async function writeDirectTextFile(path: string, contents: string): Promise<boolean> {
  if (isTauri()) {
    try {
      await writeTextFile(path, contents);
      return true;
    } catch {
      return false;
    }
  }
  return false;
}

/** Direct read from known file path without prompting dialog (Tauri mode). */
export async function readDirectTextFile(path: string): Promise<string | null> {
  if (isTauri()) {
    try {
      return await readTextFile(path);
    } catch {
      return null;
    }
  }
  return null;
}

export type { ScadaProject };
