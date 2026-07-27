import { invoke } from "@tauri-apps/api/core";
import type {
  AlarmInstance,
  AuditEntry,
  EngineSnapshot,
  Role,
  ScadaProject,
  TagValue,
  UserAccountInput,
  UserSummary,
} from "$lib/types";
import { computeSystemTagValues } from "./systemTagsService";

const isTauri = () =>
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

/** Browser mock when running `npm run dev` without Tauri. */
let mockProject: ScadaProject | null = null;
let mockConnected = false;
let mockPoll = 0;
let mockRole: Role = "administrator";
let mockMode = "designer";
let mockCurrentUser: UserSummary | null = {
  id: "usr_admin",
  username: "admin",
  display_name: "Administrator",
  security_level: 1000,
  enabled: true,
  has_pin: true,
};
let mockSecurityLevel = 1000;
let mockUsers: UserSummary[] = [
  {
    id: "usr_admin",
    username: "admin",
    display_name: "Administrator",
    security_level: 1000,
    enabled: true,
    has_pin: true,
  },
  {
    id: "usr_operator",
    username: "operator",
    display_name: "Operator Zmianowy",
    security_level: 100,
    enabled: true,
    has_pin: true,
  },
];

const mockAudit: AuditEntry[] = [];

function mockSnap(): EngineSnapshot {
  const sysTags = computeSystemTagValues({
    connected: mockConnected,
    pollCount: mockPoll,
    mode: mockMode,
    role: mockRole,
  });

  const plcTags: TagValue[] = (mockProject?.tags ?? []).map((t, i) => {
    const isLevelTag =
      t.binding.address === 104 ||
      t.id.toLowerCase().includes("level") ||
      t.id.toLowerCase().includes("poziom") ||
      t.name.toLowerCase().includes("poziom");

    const baseLevel = 420 + (mockPoll % 40) * 5;
    const rawVal = isLevelTag ? baseLevel : t.id.includes("sp") ? 200 + i * 50 : 10 + i * 5;
    const isBool = t.data_type === "bool";
    const isString = t.data_type === "string";
    const on =
      t.id === "wt.p1_run" ||
      t.id === "wt.demand" ||
      t.id === "wt.sim_en" ||
      t.id === "wt.p1_ok" ||
      t.id === "wt.p2_ok";

    const computedVal = isBool ? (on ? 1 : 0) : rawVal * (t.scale || 1) + (t.offset || 0);

    return {
      tag_id: t.id,
      value: computedVal,
      bool_value: isBool ? on : false,
      string_value: isString ? `MEM_${t.id}_${mockPoll}` : undefined,
      quality: mockConnected ? "good" : "bad",
      ts: new Date().toISOString(),
      age_ms: mockConnected ? 50 : 9999,
      raw: rawVal,
    };
  });

  const tags = [...sysTags, ...plcTags];
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
      group_id: a.group_id ?? "",
      state: "inactive" as const,
      last_change: new Date().toISOString(),
    })),
    role: mockRole,
    actor: mockCurrentUser?.username ?? "guest",
    current_user: mockCurrentUser,
    security_level: mockSecurityLevel,
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
        if (res.ok) {
          mockProject = (await res.json()) as ScadaProject;
          return mockProject as T;
        }
      } catch {
        /* fallback */
      }
      return mockProject as T;
    }
    case "load_project":
      mockProject = args?.project as ScadaProject;
      return undefined as T;
    case "get_project":
      return mockProject as T;
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
    case "login": {
      const rawTerm = args?.username_or_pin ?? args?.usernameOrPin ?? "";
      const term = String(rawTerm).trim();
      const pwd = String(args?.password ?? "").trim();

      const found = mockUsers.find((u) => {
        if (!u.enabled) return false;
        if (term === "1234" && u.username === "admin") return true;
        if (term === "1111" && u.username === "operator") return true;
        if (u.username.toLowerCase() === term.toLowerCase()) {
          if (!pwd || pwd === "admin123" || pwd === "operator123") return true;
        }
        return false;
      });

      if (found) {
        mockCurrentUser = found;
        mockSecurityLevel = found.security_level;
        mockRole =
          found.security_level >= 1000
            ? "administrator"
            : found.security_level >= 500
            ? "engineer"
            : found.security_level >= 100
            ? "operator"
            : "viewer";
        return found as T;
      }
      throw new Error("Nieprawidłowa nazwa użytkownika, hasło lub PIN");
    }
    case "logout": {
      mockCurrentUser = null;
      mockSecurityLevel = 0;
      mockRole = "viewer";
      return undefined as T;
    }
    case "verify_pin": {
      const pin = String(args?.pin ?? "");
      return (pin === "1234" || pin === "1111") as T;
    }
    case "list_users": {
      return mockUsers as T;
    }
    case "save_user": {
      const input = args?.user as UserAccountInput;
      const idx = mockUsers.findIndex(
        (u) => u.id === input.id || u.username.toLowerCase() === input.username.toLowerCase()
      );
      const sum: UserSummary = {
        id: input.id || `usr_${Date.now()}`,
        username: input.username,
        display_name: input.display_name,
        security_level: input.security_level,
        enabled: input.enabled,
        has_pin: Boolean(input.pin),
      };
      if (idx >= 0) {
        mockUsers[idx] = sum;
      } else {
        mockUsers.push(sum);
      }
      return sum as T;
    }
    case "delete_user": {
      const id = String(args?.user_id ?? "");
      mockUsers = mockUsers.filter((u) => u.id !== id);
      return undefined as T;
    }
    case "get_audit":
      return mockAudit as T;
    case "verify_audit":
      return true as T;
    case "write_tag":
      mockAudit.push({
        id: crypto.randomUUID(),
        ts: new Date().toISOString(),
        actor: mockCurrentUser?.username ?? "engineer",
        role: mockRole,
        action: "tag.write",
        detail: `${args?.tagId}=${args?.value}`,
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
  login: (usernameOrPin: string, password?: string) =>
    call<UserSummary>("login", {
      usernameOrPin,
      username_or_pin: usernameOrPin,
      password: password ?? null,
    }),
  logout: () => call<void>("logout"),
  verifyPin: (pin: string) => call<boolean>("verify_pin", { pin }),
  listUsers: () => call<UserSummary[]>("list_users"),
  saveUser: (user: UserAccountInput) => call<UserSummary>("save_user", { user }),
  deleteUser: (userId: string) => call<void>("delete_user", { user_id: userId }),
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
