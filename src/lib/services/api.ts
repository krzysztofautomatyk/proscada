import { invoke } from "@tauri-apps/api/core";
import type {
  AlarmInstance,
  AuditEntry,
  EngineSnapshot,
  Role,
  ScadaProject,
  TagValue,
} from "$lib/types";

const isTauri = () =>
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

/** Browser mock when running `npm run dev` without Tauri. */
let mockProject: ScadaProject | null = null;
let mockConnected = false;
let mockPoll = 0;
let mockRole: Role = "engineer";
let mockMode = "designer";
const mockAudit: AuditEntry[] = [];

function mockSnap(): EngineSnapshot {
  const tags: TagValue[] = (mockProject?.tags ?? []).map((t, i) => {
    const level = t.id === "wt.level_cm" ? 420 + (mockPoll % 40) * 5 : 0;
    const isBool = t.data_type === "bool";
    const on =
      t.id === "wt.p1_run" ||
      t.id === "wt.demand" ||
      t.id === "wt.sim_en" ||
      t.id === "wt.p1_ok" ||
      t.id === "wt.p2_ok";
    return {
      tag_id: t.id,
      value: isBool ? (on ? 1 : 0) : t.id.includes("level") ? level : t.id.includes("sp") ? 200 + i * 50 : i,
      bool_value: isBool ? on : false,
      quality: mockConnected ? "good" : "bad",
      ts: new Date().toISOString(),
      age_ms: mockConnected ? 50 : 9999,
      raw: 0,
    };
  });
  return {
    connected: mockConnected,
    device_id: mockProject?.devices[0]?.id ?? null,
    last_error: mockConnected ? null : "Mock: start polling after tauri:dev + PLC slave",
    poll_count: mockPoll,
    last_poll_ms: 12,
    tags,
    alarms: (mockProject?.alarms ?? []).map((a) => ({
      def_id: a.id,
      name: a.name,
      message: a.message,
      priority: a.priority,
      state: "inactive" as const,
      last_change: new Date().toISOString(),
    })),
    role: mockRole,
    actor: "engineer",
    project_name: mockProject?.name ?? null,
    mode: mockMode,
  };
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    return mockInvoke<T>(cmd, args);
  }
  return invoke<T>(cmd, args);
}

async function mockInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  switch (cmd) {
    case "get_builtin_water_tank":
    case "load_builtin_water_tank": {
      try {
        const res = await fetch("/projects/WaterTank.proscada.json");
        const ct = res.headers.get("content-type") || "";
        if (res.ok && ct.includes("application/json")) {
          mockProject = (await res.json()) as ScadaProject;
        }
      } catch {
        /* fallback below */
      }
      if (!mockProject) {
        mockProject = {
          schema_version: "1.0",
          id: "water_tank_dual_pump",
          name: "Water Tank Dual-Pump Station",
          content_hash: "0000000000000000000000000000000000000000000000000000000000000000",
          forms: [
            {
              id: "Main_Synoptic",
              name: "Main_Synoptic",
              width: 1040,
              height: 700,
              background: "#121316",
              grid: 8,
              widgets: [
                {
                  id: "m_bg",
                  widget_type: "shape",
                  x: 20,
                  y: 20,
                  w: 480,
                  h: 220,
                  z: 1,
                  tag_id: null,
                  group_id: "grp_metrics",
                  config: {
                    background: "#1e1f24",
                    borderColor: "#33353c",
                    borderWidth: 1,
                    borderRadius: 10,
                  },
                },
                {
                  id: "m_level",
                  widget_type: "numeric",
                  x: 40,
                  y: 40,
                  w: 200,
                  h: 60,
                  z: 2,
                  tag_id: "wt.level_cm",
                  group_id: "grp_metrics",
                  config: {
                    label: "Poziom wody (Water Level)",
                    unit: "cm",
                    decimals: 1,
                    fontSize: 16,
                    textColor: "#60a5fa",
                  },
                },
              ],
            },
          ],
          devices: [
            {
              id: "dev_plc1",
              name: "PLC Station 1",
              host: "127.0.0.1",
              port: 502,
              unit_id: 1,
              timeout_ms: 1000,
              enabled: true,
            },
          ],
          tags: [
            {
              id: "wt.level_cm",
              tag_id: "wt.level_cm",
              name: "Water Level",
              data_type: "float",
              register_type: "holding",
              address: 0,
              scale: 1,
              offset: 0,
              decimals: 1,
              unit: "cm",
              description: "Water tank level",
            },
            {
              id: "wt.p1_run",
              tag_id: "wt.p1_run",
              name: "Pump 1 Run",
              data_type: "bool",
              register_type: "holding",
              address: 1,
              scale: 1,
              offset: 0,
              decimals: 0,
              unit: "",
              description: "Pump 1 run signal",
            },
          ],
          alarms: [],
          events: [],
          history: [],
          logs: [],
          users: [],
          roles: [],
          settings: {
            theme: "dark",
            refresh_rate_ms: 200,
          },
        };
      }
      return mockProject as T;
    }
    case "get_project":
      return mockProject as T;
    case "load_project":
      mockProject = args?.project as ScadaProject;
      return undefined as T;
    case "save_project_in_memory":
      mockProject = args?.project as ScadaProject;
      return mockProject as T;
    case "get_snapshot":
      if (mockConnected) mockPoll++;
      return mockSnap() as T;
    case "start_polling":
      mockConnected = true;
      return undefined as T;
    case "stop_polling":
      mockConnected = false;
      return undefined as T;
    case "set_role":
      mockRole = args?.role as Role;
      return undefined as T;
    case "set_mode":
      mockMode = String(args?.mode ?? "designer");
      return undefined as T;
    case "get_audit":
      return mockAudit as T;
    case "verify_audit":
      return true as T;
    case "write_tag":
      mockAudit.push({
        id: crypto.randomUUID(),
        ts: new Date().toISOString(),
        actor: "engineer",
        role: mockRole,
        action: "tag.write",
        detail: `${args?.tag_id}=${args?.value}`,
        prev_hash: "GENESIS",
        hash: "mock",
      });
      return undefined as T;
    case "ack_alarm":
      return undefined as T;
    case "test_device":
      return { ok: false, message: "Browser mock — use tauri:dev" } as T;
    case "get_tag_values":
      return mockSnap().tags as T;
    case "get_alarms":
      return mockSnap().alarms as T;
    default:
      console.warn("mock unhandled", cmd);
      return undefined as T;
  }
}

export const api = {
  getBuiltinWaterTank: () => call<ScadaProject>("get_builtin_water_tank"),
  loadBuiltinWaterTank: () => call<ScadaProject>("load_builtin_water_tank"),
  getProject: () => call<ScadaProject | null>("get_project"),
  loadProject: (project: ScadaProject) => call<void>("load_project", { project }),
  saveProject: (project: ScadaProject) =>
    call<ScadaProject>("save_project_in_memory", { project }),
  getSnapshot: () => call<EngineSnapshot>("get_snapshot"),
  startPolling: (deviceId?: string | null) =>
    call<void>("start_polling", { deviceId: deviceId ?? null }),
  stopPolling: () => call<void>("stop_polling"),
  writeTag: (tagId: string, value: number) =>
    call<void>("write_tag", { tagId, value }),
  ackAlarm: (defId: string) => call<void>("ack_alarm", { defId }),
  setRole: (role: Role, actor: string) => call<void>("set_role", { role, actor }),
  setMode: (mode: string) => call<void>("set_mode", { mode }),
  getAudit: (limit = 200) => call<AuditEntry[]>("get_audit", { limit }),
  verifyAudit: () => call<boolean>("verify_audit"),
  testDevice: (host: string, port: number, unitId: number, timeoutMs: number) =>
    call<{ ok: boolean; message: string }>("test_device", {
      host,
      port,
      unitId,
      timeoutMs,
    }),
  getTagValues: () => call<TagValue[]>("get_tag_values"),
  getAlarms: () => call<AlarmInstance[]>("get_alarms"),
};
