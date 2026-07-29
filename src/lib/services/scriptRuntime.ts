import { get } from "svelte/store";
import { api } from "$lib/services/api";
import {
  log,
  project,
  selectedFormId,
  selectedWidgetId,
  tagMap,
} from "$lib/stores/app";
import type { ProjectNode } from "$lib/types";

/**
 * Project scripts are a small, explicitly parsed action language — not JavaScript.
 *
 * Two reasons:
 *  - the packaged application runs under `script-src 'self'`, which blocks
 *    `eval`/`new Function`, so a JavaScript runtime would simply not execute;
 *  - `AGENTS.md` forbids arbitrary script execution inside a project file.
 *
 * Every statement maps to one whitelisted operation, and anything the parser
 * does not recognise is a hard error rather than a silent no-op.
 */

export type ScriptEventType = "click" | "load" | "timer" | "custom";

export interface ScriptEvent {
  type: ScriptEventType;
  widgetId?: string | null;
  formId?: string | null;
  tagId?: string | null;
  payload?: unknown;
}

export type Comparator = "==" | "!=" | ">" | ">=" | "<" | "<=";

export interface ScriptCondition {
  tagId: string;
  comparator: Comparator;
  value: number;
}

export type ScriptAction =
  | { kind: "writeTag"; tagId: string; value: number }
  | { kind: "ackAlarm"; alarmId: string }
  | { kind: "navigate"; formId: string }
  | { kind: "log"; message: string };

export interface ScriptStatement {
  line: number;
  condition: ScriptCondition | null;
  action: ScriptAction;
}

export class ScriptParseError extends Error {
  constructor(
    message: string,
    readonly line: number,
  ) {
    super(`Line ${line}: ${message}`);
    this.name = "ScriptParseError";
  }
}

const COMPARATORS: Comparator[] = ["==", "!=", ">=", "<=", ">", "<"];

/** Split a statement into tokens, honouring double-quoted string literals. */
function tokenize(source: string, line: number): string[] {
  const tokens: string[] = [];
  let index = 0;
  while (index < source.length) {
    const char = source[index];
    if (char === " " || char === "\t") {
      index += 1;
      continue;
    }
    if (char === '"') {
      const end = source.indexOf('"', index + 1);
      if (end === -1) throw new ScriptParseError("Unterminated string literal", line);
      tokens.push(source.slice(index + 1, end));
      index = end + 1;
      continue;
    }
    let end = index;
    while (end < source.length && source[end] !== " " && source[end] !== "\t") end += 1;
    tokens.push(source.slice(index, end));
    index = end;
  }
  return tokens;
}

function parseNumber(token: string | undefined, line: number, what: string): number {
  const value = Number(token);
  if (token === undefined || token === "" || !Number.isFinite(value)) {
    throw new ScriptParseError(`${what} must be a finite number`, line);
  }
  return value;
}

function parseAction(tokens: string[], line: number): ScriptAction {
  const [head, ...rest] = tokens;
  switch (head) {
    case "writeTag": {
      if (rest.length !== 2) {
        throw new ScriptParseError('writeTag expects: writeTag "tag.id" <number>', line);
      }
      return {
        kind: "writeTag",
        tagId: rest[0],
        value: parseNumber(rest[1], line, "Write value"),
      };
    }
    case "ackAlarm": {
      if (rest.length !== 1) {
        throw new ScriptParseError('ackAlarm expects: ackAlarm "alarm.id"', line);
      }
      return { kind: "ackAlarm", alarmId: rest[0] };
    }
    case "navigate": {
      if (rest.length !== 1) {
        throw new ScriptParseError('navigate expects: navigate "form.id"', line);
      }
      return { kind: "navigate", formId: rest[0] };
    }
    case "log": {
      if (rest.length !== 1) {
        throw new ScriptParseError('log expects: log "message"', line);
      }
      return { kind: "log", message: rest[0] };
    }
    default:
      throw new ScriptParseError(
        `Unknown action "${head ?? ""}". Allowed: writeTag, ackAlarm, navigate, log`,
        line,
      );
  }
}

function parseCondition(tokens: string[], line: number): ScriptCondition {
  if (tokens.length !== 3) {
    throw new ScriptParseError(
      'if expects: if "tag.id" <comparator> <number> then <action>',
      line,
    );
  }
  const [tagId, comparator, rawValue] = tokens;
  if (!COMPARATORS.includes(comparator as Comparator)) {
    throw new ScriptParseError(
      `Unknown comparator "${comparator}". Allowed: ${COMPARATORS.join(", ")}`,
      line,
    );
  }
  return {
    tagId,
    comparator: comparator as Comparator,
    value: parseNumber(rawValue, line, "Comparison value"),
  };
}

/** Parse a project script into statements. Throws {@link ScriptParseError}. */
export function parseScript(source: string): ScriptStatement[] {
  const statements: ScriptStatement[] = [];
  const lines = source.split("\n");

  for (let index = 0; index < lines.length; index += 1) {
    const lineNumber = index + 1;
    const raw = lines[index].trim();
    if (raw === "" || raw.startsWith("#") || raw.startsWith("//")) continue;

    const tokens = tokenize(raw, lineNumber);
    if (tokens.length === 0) continue;

    if (tokens[0] === "if") {
      const thenIndex = tokens.indexOf("then");
      if (thenIndex === -1) {
        throw new ScriptParseError('if statement is missing "then"', lineNumber);
      }
      statements.push({
        line: lineNumber,
        condition: parseCondition(tokens.slice(1, thenIndex), lineNumber),
        action: parseAction(tokens.slice(thenIndex + 1), lineNumber),
      });
      continue;
    }

    statements.push({
      line: lineNumber,
      condition: null,
      action: parseAction(tokens, lineNumber),
    });
  }

  return statements;
}

/**
 * Evaluate a condition against a tag value.
 *
 * A missing tag, or one whose quality is not `good`, never satisfies a
 * condition: acting on an untrusted value is worse than not acting.
 */
export function evaluateCondition(
  condition: ScriptCondition,
  lookup: (tagId: string) => { value: number; quality: string } | undefined,
): boolean {
  const tag = lookup(condition.tagId);
  if (!tag || tag.quality !== "good") return false;
  switch (condition.comparator) {
    case "==":
      return tag.value === condition.value;
    case "!=":
      return tag.value !== condition.value;
    case ">":
      return tag.value > condition.value;
    case ">=":
      return tag.value >= condition.value;
    case "<":
      return tag.value < condition.value;
    case "<=":
      return tag.value <= condition.value;
  }
}

async function runAction(action: ScriptAction): Promise<void> {
  switch (action.kind) {
    case "writeTag":
      await api.writeTag(action.tagId, action.value);
      log(`script.write ${action.tagId}=${action.value}`, "ok");
      return;
    case "ackAlarm":
      await api.ackAlarm(action.alarmId);
      log(`script.ack ${action.alarmId}`, "ok");
      return;
    case "navigate": {
      const current = get(project);
      if (!current?.forms.some((form) => form.id === action.formId)) {
        throw new Error(`Unknown form: ${action.formId}`);
      }
      selectedFormId.set(action.formId);
      selectedWidgetId.set(null);
      log(`script.navigate → ${action.formId}`, "ok");
      return;
    }
    case "log":
      log(`[script] ${action.message}`, "info");
  }
}

/** Run a parsed project script node. */
export async function runProjectScript(node: ProjectNode, _event: ScriptEvent): Promise<void> {
  if (node.kind !== "script") {
    throw new Error("Not a script node");
  }

  let statements: ScriptStatement[];
  try {
    statements = parseScript(node.content ?? "");
  } catch (error) {
    log(`Script "${node.name}" is invalid: ${error}`, "err");
    throw error;
  }

  const tags = get(tagMap);
  for (const statement of statements) {
    if (statement.condition && !evaluateCondition(statement.condition, (id) => tags.get(id))) {
      continue;
    }
    try {
      await runAction(statement.action);
    } catch (error) {
      log(`Script "${node.name}" line ${statement.line} failed: ${error}`, "err");
      throw error;
    }
  }
}

export async function runScriptById(scriptId: string, event: ScriptEvent): Promise<void> {
  const current = get(project);
  const node = current?.tree?.find((n) => n.id === scriptId && n.kind === "script");
  if (!node) {
    log(`Script not found: ${scriptId}`, "warn");
    return;
  }
  await runProjectScript(node, event);
}
