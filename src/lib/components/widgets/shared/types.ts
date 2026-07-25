import type { FormDef, TagValue, WidgetDef } from "$lib/types";

export interface WidgetRendererProps {
  widget: WidgetDef;
  tag?: TagValue | null;
  design?: boolean;
  onWrite?: (tagId: string, value: number) => void;
  tagMap?: Map<string, TagValue>;
  forms?: FormDef[];
  ancestorFormIds?: Set<string>;
}

export type WidgetConfig = Record<string, unknown>;

