import type { FormDef } from "$lib/types";

/** Check if a form is the protected primary Main screen. */
export function isMainScreen(form: FormDef | { name: string; id: string }): boolean {
  if (!form) return false;
  const nameLower = form.name.trim().toLowerCase();
  return (
    nameLower === "main" ||
    nameLower === "main_synoptic" ||
    nameLower === "main.form" ||
    nameLower === "ekran_glowny" ||
    nameLower === "ekran główny"
  );
}

/** Check if a form's name can be renamed (Main screen name is strictly immutable). */
export function canRenameForm(form: FormDef | { name: string; id: string }): boolean {
  if (!form) return false;
  return !isMainScreen(form);
}

/** Check if a form can be deleted (Main screen and the last remaining screen are strictly undeletable). */
export function canDeleteForm(formId: string, forms: FormDef[]): boolean {
  if (!Array.isArray(forms) || forms.length <= 1) return false;
  const target = forms.find((f) => f.id === formId);
  if (!target) return false;
  if (isMainScreen(target)) return false;
  return true;
}

/** Ensure project forms array always contains a protected Main screen. */
export function ensureMainFormExists(forms: FormDef[]): FormDef[] {
  if (!Array.isArray(forms) || forms.length === 0) {
    return [
      {
        id: "Main_Synoptic",
        name: "Main",
        width: 1040,
        height: 700,
        background: "#F4F5F7",
        grid: 8,
        widgets: [],
      },
    ];
  }

  const hasMain = forms.some((f) => isMainScreen(f));
  if (!hasMain) {
    const mainForm: FormDef = {
      id: "Main_Synoptic",
      name: "Main",
      width: 1040,
      height: 700,
      background: "#F4F5F7",
      grid: 8,
      widgets: [],
    };
    return [mainForm, ...forms];
  }

  return forms;
}
