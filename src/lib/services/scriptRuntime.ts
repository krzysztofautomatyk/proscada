import { get } from "svelte/store";
import { api } from "$lib/services/api";
import {
  log,
  project,
  selectedFormId,
  selectedWidgetId,
  snapshot,
  tagMap,
} from "$lib/stores/app";
import type { ProjectNode } from "$lib/types";

export type ScriptEventType = "click" | "load" | "timer" | "custom";

export interface ScriptEvent {
  type: ScriptEventType;
  widgetId?: string | null;
  formId?: string | null;
  tagId?: string | null;
  payload?: unknown;
}

export interface ScriptApi {
  writeTag: (tagId: string, value: number) => Promise<void>;
  getTag: (tagId: string) => ReturnType<typeof getTagSnapshot>;
  getTagValue: (tagId: string) => number | boolean | null;
  log: (msg: string) => void;
  navigate: (formId: string) => void;
  ackAlarm: (defId: string) => Promise<void>;
  getProjectName: () => string;
}

function getTagSnapshot(tagId: string) {
  return get(tagMap).get(tagId) ?? null;
}

function buildApi(): ScriptApi {
  return {
    async writeTag(tagId: string, value: number) {
      await api.writeTag(tagId, value);
      log(`script.write ${tagId}=${value}`, "ok");
    },
    getTag: getTagSnapshot,
    getTagValue(tagId: string) {
      const t = getTagSnapshot(tagId);
      if (!t) return null;
      return t.bool_value !== undefined && get(project)?.tags.find((x) => x.id === tagId)?.data_type === "bool"
        ? t.bool_value
        : t.value;
    },
    log(msg: string) {
      log(`[script] ${msg}`, "info");
    },
    navigate(formId: string) {
      const p = get(project);
      if (!p?.forms.some((f) => f.id === formId)) {
        log(`script.navigate: unknown form ${formId}`, "warn");
        return;
      }
      selectedFormId.set(formId);
      selectedWidgetId.set(null);
      log(`script.navigate → ${formId}`, "ok");
    },
    async ackAlarm(defId: string) {
      await api.ackAlarm(defId);
      log(`script.ack ${defId}`, "ok");
    },
    getProjectName() {
      return get(project)?.name ?? "";
    },
  };
}

/**
 * Run project script in a restricted Function sandbox.
 * No access to window/document/fetch — only the injected API.
 */
export async function runProjectScript(
  node: ProjectNode,
  event: ScriptEvent,
): Promise<void> {
  if (node.kind !== "script") {
    throw new Error("Not a script node");
  }
  const source = node.content ?? "";
  const apiObj = buildApi();
  const snap = get(snapshot);

  // Async Function with frozen API surface
  const runner = new Function(
    "api",
    "event",
    "snapshot",
    `"use strict";
    const { writeTag, getTag, getTagValue, log, navigate, ackAlarm, getProjectName } = api;
    ${source}
    if (typeof onEvent === "function") {
      return onEvent(event);
    }
    if (typeof main === "function") {
      return main(event);
    }
    return undefined;`,
  ) as (
    api: ScriptApi,
    event: ScriptEvent,
    snapshot: unknown,
  ) => unknown;

  try {
    const result = runner(Object.freeze({ ...apiObj }), event, snap);
    await Promise.resolve(result);
  } catch (e) {
    log(`Script "${node.name}" error: ${e}`, "err");
    throw e;
  }
}

export async function runScriptById(scriptId: string, event: ScriptEvent): Promise<void> {
  const p = get(project);
  const node = p?.tree?.find((n) => n.id === scriptId && n.kind === "script");
  if (!node) {
    log(`Script not found: ${scriptId}`, "warn");
    return;
  }
  await runProjectScript(node, event);
}
