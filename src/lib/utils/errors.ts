/**
 * Extract a human-readable message from an unknown thrown value.
 *
 * Tauri rejects with a plain string, `fetch` and the DOM throw `Error`, and a
 * bug can throw anything at all — typing those call sites as `any` just moves
 * the failure to runtime.
 */
export function errorMessage(error: unknown, fallback: string): string {
  if (error instanceof Error && error.message) return error.message;
  if (typeof error === "string" && error.trim() !== "") return error;
  if (error && typeof error === "object" && "message" in error) {
    const message = (error as { message: unknown }).message;
    if (typeof message === "string" && message.trim() !== "") return message;
  }
  return fallback;
}
