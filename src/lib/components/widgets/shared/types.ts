import type { FormDef, TagValue, WidgetDef } from "$lib/types";

export interface ProcessWriteOptions {
  pin?: string;
}

export interface ProcessWriteResult {
  status: "accepted" | "observed";
  tagId: string;
  requestedValue: number;
  observedValue?: number;
  rawReadback?: number;
  protocol?: string;
  verifyReadback?: boolean;
  matches?: boolean;
  selfCleared?: boolean;
}

export type ProcessWrite = (
  tagId: string,
  value: number,
  options?: ProcessWriteOptions,
) => Promise<ProcessWriteResult>;

export interface WidgetRendererProps {
  widget: WidgetDef;
  tag?: TagValue | null;
  design?: boolean;
  onWrite?: ProcessWrite;
  tagMap?: Map<string, TagValue>;
  forms?: FormDef[];
  ancestorFormIds?: Set<string>;
}

export type WidgetConfig = Record<string, unknown>;
