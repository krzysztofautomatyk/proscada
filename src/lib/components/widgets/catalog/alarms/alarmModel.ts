import type { AlarmPriority, AlarmState } from "$lib/types";
import { readRecordList } from "$lib/components/widgets/shared/config";
import type { WidgetConfig } from "$lib/components/widgets/shared/types";

export interface ConfigAlarm {
  id: string;
  time: string;
  priority: AlarmPriority;
  state: AlarmState;
  message: string;
  group: string;
  shelved: boolean;
  evaluationSuspended: boolean;
  suspendedReason: string;
  suspendedSince: string;
}

const priorities: AlarmPriority[] = ["low", "medium", "high", "critical"];
const states: AlarmState[] = [
  "inactive",
  "active_unacked",
  "active_acked",
  "cleared_unacked",
];

export const designAlarms: ConfigAlarm[] = [
  {
    id: "LT-201-HH",
    time: "2026-07-25 12:42:16",
    priority: "high",
    state: "active_unacked",
    message: "LT-201 Wet well level high-high",
    group: "Lift station",
    shelved: false,
    evaluationSuspended: false,
    suspendedReason: "",
    suspendedSince: "",
  },
  {
    id: "P-101-THERM",
    time: "2026-07-25 12:38:04",
    priority: "medium",
    state: "active_acked",
    message: "P-101 thermal overload active",
    group: "Pumping",
    shelved: false,
    evaluationSuspended: false,
    suspendedReason: "",
    suspendedSince: "",
  },
];

function cell(row: Record<string, unknown>, key: string, fallback = ""): string {
  const value = row[key];
  return value === undefined || value === null ? fallback : String(value).trim();
}

function boolCell(row: Record<string, unknown>, key: string): boolean {
  const value = row[key];
  return value === true || value === 1 || value === "true";
}

export function parseAlarms(config: WidgetConfig): {
  alarms: ConfigAlarm[];
  error: string | null;
  configured: boolean;
} {
  const configured = Object.prototype.hasOwnProperty.call(config, "alarms");
  const parsed = readRecordList(config, "alarms");
  if (parsed.error) return { alarms: [], error: parsed.error, configured };

  const errors: string[] = [];
  const alarms: ConfigAlarm[] = [];
  for (const [index, row] of parsed.rows.entries()) {
    const id = cell(row, "id");
    const time = cell(row, "time");
    const message = cell(row, "message");
    const group = cell(row, "group");
    const priority = cell(row, "priority").toLowerCase() as AlarmPriority;
    const state = cell(row, "state").toLowerCase() as AlarmState;
    if (!id || !time || !message || !group || !priorities.includes(priority) || !states.includes(state)) {
      errors.push(`alarms[${index}] requires id, time, message, group, valid priority and state`);
      continue;
    }
    alarms.push({
      id,
      time,
      priority,
      state,
      message,
      group,
      shelved: boolCell(row, "shelved"),
      evaluationSuspended: boolCell(row, "evaluationSuspended"),
      suspendedReason: cell(row, "suspendedReason"),
      suspendedSince: cell(row, "suspendedSince"),
    });
  }
  return { alarms, error: errors.length ? errors.join("; ") : null, configured };
}

export function priorityRank(priority: AlarmPriority): number {
  return priorities.indexOf(priority);
}

export function stateLabel(state: AlarmState): string {
  switch (state) {
    case "active_unacked":
      return "ACTIVE · UNACKED";
    case "active_acked":
      return "ACTIVE · ACKED";
    case "cleared_unacked":
      return "CLEARED · UNACKED";
    default:
      return "INACTIVE";
  }
}

export function stateIcon(state: AlarmState): string {
  switch (state) {
    case "active_unacked":
      return "▲";
    case "active_acked":
      return "●";
    case "cleared_unacked":
      return "✓";
    default:
      return "○";
  }
}

export function isActive(alarm: ConfigAlarm): boolean {
  return alarm.state === "active_unacked" || alarm.state === "active_acked";
}
