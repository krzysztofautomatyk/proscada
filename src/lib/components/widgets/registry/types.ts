export const TOOLBOX_CATEGORIES = [
  "Prymitywy",
  "Zasoby",
  "Wskaźniki",
  "Wizualizacja procesu",
  "Sterowanie",
  "Wejścia",
  "Dane i historia",
  "Układ",
  "Nawigacja",
  "Informacja i interakcja",
  "Alarmy",
  "Narzędzia",
  "Szablony procesu",
] as const;

export type WidgetCategory = (typeof TOOLBOX_CATEGORIES)[number];
export type WidgetCatalogStatus = "canonical" | "template";

export interface WidgetCatalogItem {
  canonicalId: string;
  type: string;
  label: string;
  description: string;
  category: WidgetCategory;
  icon: string;
  defaultW: number;
  defaultH: number;
  defaultConfig: Record<string, unknown>;
  capabilities: readonly string[];
  aliases?: readonly string[];
  status?: WidgetCatalogStatus;
}

export function defineWidget(item: WidgetCatalogItem): WidgetCatalogItem {
  return item;
}

