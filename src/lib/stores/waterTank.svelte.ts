/**
 * Water Tank HMI store — Single Source of Truth for isometric scene.
 * Fed from Modbus tag map (ProScada engine), not mock timers.
 */
import type { AlarmInstance, TagValue } from "$lib/types";

export type AlarmPriority = "CRITICAL" | "HIGH" | "MEDIUM" | "LOW";

export interface HmiAlarm {
  id: string;
  text: string;
  priority: AlarmPriority;
  acknowledged: boolean;
}

export interface WaterTankViewModel {
  levelCm: number;
  levelPercent: number;
  spStop: number;
  spP1: number;
  spP2: number;
  kX100: number;
  fillStep: number;
  p1Run: boolean;
  p2Run: boolean;
  p1Fault: boolean;
  p2Fault: boolean;
  p1Ok: boolean;
  p2Ok: boolean;
  demand: boolean;
  joinP2: boolean;
  almHi: boolean;
  almFault: boolean;
  almFail: boolean;
  simEn: boolean;
  drain: boolean;
  p1Starts: number;
  p2Starts: number;
  p1Hours: number;
  p2Hours: number;
  qualityLevel: string;
  connected: boolean;
  overflowActive: boolean;
  levelState: "ok" | "warn" | "alarm";
  stationLabel: string;
  stationColor: string;
  hmiAlarms: HmiAlarm[];
}

function n(map: Map<string, TagValue>, id: string, d = 0) {
  return map.get(id)?.value ?? d;
}
function b(map: Map<string, TagValue>, id: string) {
  return map.get(id)?.bool_value ?? false;
}
function q(map: Map<string, TagValue>, id: string) {
  return map.get(id)?.quality ?? "bad";
}

/** Pure projector: tagMap + engine alarms → view model (no side effects). */
export function projectWaterTank(
  tagMap: Map<string, TagValue>,
  connected: boolean,
  engineAlarms: AlarmInstance[] = [],
): WaterTankViewModel {
  const levelCm = n(tagMap, "wt.level_cm");
  const levelPercent = Math.max(0, Math.min(100, levelCm / 10));
  const spStop = n(tagMap, "wt.sp_stop", 200);
  const spP1 = n(tagMap, "wt.sp_p1_on", 700);
  const spP2 = n(tagMap, "wt.sp_p2_on", 800);
  const almHi = b(tagMap, "wt.alm_hi");
  const almFault = b(tagMap, "wt.alm_fault");
  const almFail = b(tagMap, "wt.alm_fail");
  const demand = b(tagMap, "wt.demand");
  const p1Run = b(tagMap, "wt.p1_run");
  const p2Run = b(tagMap, "wt.p2_run");

  let levelState: "ok" | "warn" | "alarm" = "ok";
  if (almHi || levelCm >= spP2) levelState = "alarm";
  else if (demand || levelCm >= spP1) levelState = "warn";

  let stationLabel = "NORMAL";
  let stationColor = "#16A34A";
  if (almFail) {
    stationLabel = "STATION FAIL";
    stationColor = "#DC2626";
  } else if (almFault) {
    stationLabel = "PUMP FAULT";
    stationColor = "#DC2626";
  } else if (almHi) {
    stationLabel = "HIGH LEVEL";
    stationColor = "#DC2626";
  } else if (demand) {
    stationLabel = "DEMAND";
    stationColor = "#EAB308";
  }

  const hmiAlarms: HmiAlarm[] = [];
  if (almFail)
    hmiAlarms.push({
      id: "alm_fail",
      text: "STATION FAIL — no pump available",
      priority: "CRITICAL",
      acknowledged: false,
    });
  if (almFault)
    hmiAlarms.push({
      id: "alm_fault",
      text: "PUMP FAULT",
      priority: "HIGH",
      acknowledged: false,
    });
  if (almHi)
    hmiAlarms.push({
      id: "alm_hi",
      text: "HIGH LEVEL ALARM",
      priority: "HIGH",
      acknowledged: false,
    });
  if (levelPercent > 95)
    hmiAlarms.push({
      id: "overflow",
      text: "OVERFLOW RISK (>95%)",
      priority: "CRITICAL",
      acknowledged: false,
    });
  if (levelPercent < 5 && connected)
    hmiAlarms.push({
      id: "low",
      text: "LOW LEVEL",
      priority: "MEDIUM",
      acknowledged: false,
    });

  // Merge engine alarm states for ack tracking
  for (const a of engineAlarms) {
    if (a.state === "inactive") continue;
    const existing = hmiAlarms.find((h) => h.id === a.def_id);
    if (existing) {
      existing.acknowledged = a.state === "active_acked";
    } else {
      hmiAlarms.push({
        id: a.def_id,
        text: a.message || a.name,
        priority:
          a.priority === "critical"
            ? "CRITICAL"
            : a.priority === "high"
              ? "HIGH"
              : a.priority === "medium"
                ? "MEDIUM"
                : "LOW",
        acknowledged: a.state === "active_acked",
      });
    }
  }

  return {
    levelCm,
    levelPercent,
    spStop,
    spP1,
    spP2,
    kX100: n(tagMap, "wt.k_x100", 150),
    fillStep: n(tagMap, "wt.fill_step", 10),
    p1Run,
    p2Run,
    p1Fault: b(tagMap, "wt.p1_fault"),
    p2Fault: b(tagMap, "wt.p2_fault"),
    p1Ok: b(tagMap, "wt.p1_ok"),
    p2Ok: b(tagMap, "wt.p2_ok"),
    demand,
    joinP2: b(tagMap, "wt.join_p2"),
    almHi,
    almFault,
    almFail,
    simEn: b(tagMap, "wt.sim_en"),
    drain: b(tagMap, "wt.drain_regime"),
    p1Starts: n(tagMap, "wt.p1_starts"),
    p2Starts: n(tagMap, "wt.p2_starts"),
    p1Hours: n(tagMap, "wt.p1_hh"),
    p2Hours: n(tagMap, "wt.p2_hh"),
    qualityLevel: q(tagMap, "wt.level_cm"),
    connected,
    overflowActive: levelPercent > 95 || almHi,
    levelState,
    stationLabel,
    stationColor,
    hmiAlarms,
  };
}

export function levelAccent(state: "ok" | "warn" | "alarm"): string {
  if (state === "alarm") return "#DC2626";
  if (state === "warn") return "#EAB308";
  return "#16A34A";
}

export function pumpColor(run: boolean, fault: boolean): string {
  if (fault) return "#DC2626";
  if (run) return "#16A34A";
  return "#6B7280";
}
