import { invoke } from "@tauri-apps/api/core";
import type {
  AlarmInstance,
  AuditEntry,
  AuditStatus,
  EngineSnapshot,
  Role,
  ScadaProject,
  TagValue,
  UserAccountInput,
  UserSummary,
  WriteReceipt,
} from "$lib/types";
import { computeSystemTagValues } from "./systemTagsService";

const isTauri = () =>
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

/**
 * Browser mock when running `npm run dev` without Tauri.
 *
 * It mirrors the backend's fail-closed defaults on purpose: a mock that starts
 * as Administrator hides exactly the authorization bugs it should expose.
 */
let mockProject: ScadaProject | null = null;
let mockConnected = false;
let mockPoll = 0;
let mockRole: Role = "viewer";
let mockMode = "runtime";
let mockCurrentUser: UserSummary | null = null;
let mockSecurityLevel = 0;
let mockPasswordChangeRequired = false;
const mockPasswords = new Map<string, string>();
const mockPins = new Map<string, string>();
let mockUsers: UserSummary[] = [];

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

    const initialNum = t.initial_value !== undefined && Number.isFinite(Number(t.initial_value)) ? Number(t.initial_value) : null;
    const computedVal = initialNum !== null ? initialNum : isBool ? (on ? 1 : 0) : rawVal * (t.scale || 1) + (t.offset || 0);

    return {
      tag_id: t.id,
      value: computedVal,
      bool_value: isBool ? on : false,
      string_value: isString ? (t.initial_value ?? `MEM_${t.id}_${mockPoll}`) : undefined,
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
    alarms_suspended: !mockConnected,
    password_change_required: mockPasswordChangeRequired,
    requires_bootstrap: mockUsers.length === 0,
    audit_chain_ok: true,
    audit_persisted: true,
    audit_last_error: null,
    alarm_state_persisted: true,
    alarm_state_last_error: null,
    user_realm_persisted: true,
    user_realm_last_error: null,
  };
}

function roleForLevel(level: number): Role {
  if (level >= 1000) return "administrator";
  if (level >= 500) return "engineer";
  if (level >= 100) return "operator";
  return "viewer";
}

function mockResetSession() {
  mockCurrentUser = null;
  mockSecurityLevel = 0;
  mockRole = "viewer";
  mockPasswordChangeRequired = false;
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
    case "load_project": {
      if (mockSecurityLevel < 500) {
        throw new Error("Engineer or Administrator role is required to import a project");
      }
      const incoming = structuredClone(args?.project as ScadaProject);
      // The browser mock keeps its account realm separately, matching Rust:
      // credentials embedded in imported process/design content are ignored.
      incoming.users = [];
      mockProject = incoming;
      mockResetSession();
      return undefined as T;
    }
    case "get_project":
      return (mockProject ? { ...structuredClone(mockProject), users: [] } : null) as T;
    case "save_project_file":
      if (mockSecurityLevel < 500) {
        throw new Error("Engineer or Administrator role is required to save a project file");
      }
      return undefined as T;
    case "save_project_in_memory":
      if (mockSecurityLevel < 500) {
        throw new Error("Engineer or Administrator role is required to edit the project");
      }
      if (mockPasswordChangeRequired) {
        throw new Error("Change the default password before editing the project");
      }
      mockProject = {
        ...structuredClone(args?.project as ScadaProject),
        users: [],
      };
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
    case "set_mode": {
      const requested = String(args?.mode ?? "designer");
      if (requested !== "designer" && requested !== "runtime") {
        throw new Error(`Unknown mode: ${requested}`);
      }
      if (requested === "designer" && mockSecurityLevel < 500) {
        throw new Error("Engineer or Administrator role is required to enter Designer");
      }
      mockMode = requested;
      return undefined as T;
    }
    case "login": {
      const rawTerm = args?.username ?? "";
      const term = String(rawTerm).trim();
      const pwd = String(args?.password ?? "");

      const found = mockUsers.find((u) => {
        if (!u.enabled) return false;
        return (
          u.username.toLowerCase() === term.toLowerCase() &&
          pwd.length > 0 &&
          mockPasswords.get(u.username) === pwd
        );
      });

      if (found) {
        mockCurrentUser = found;
        mockSecurityLevel = found.security_level;
        mockRole = roleForLevel(found.security_level);
        mockPasswordChangeRequired = found.password_change_required === true;
        return found as T;
      }
      throw new Error("Nieprawidłowa nazwa użytkownika lub hasło");
    }
    case "logout": {
      mockResetSession();
      return undefined as T;
    }
    case "change_password": {
      const current = String(args?.currentPassword ?? "");
      const next = String(args?.newPassword ?? "");
      if (!mockCurrentUser) throw new Error("No user is signed in");
      if (next.length < 12) throw new Error("New password must be at least 12 characters long");
      if (next === current) throw new Error("New password must differ from the current one");
      if (mockPasswords.get(mockCurrentUser.username) !== current) {
        throw new Error("Current password is incorrect");
      }
      mockPasswords.set(mockCurrentUser.username, next);
      mockPasswordChangeRequired = false;
      mockCurrentUser = { ...mockCurrentUser, password_change_required: false };
      mockUsers = mockUsers.map((u) =>
        u.id === mockCurrentUser?.id ? { ...u, password_change_required: false } : u,
      );
      return mockCurrentUser as T;
    }
    case "bootstrap_admin": {
      const password = String(args?.password ?? "");
      if (mockUsers.length > 0) {
        throw new Error("Administrator bootstrap is already closed for this project");
      }
      if (password.length < 12) {
        throw new Error("Bootstrap password must be at least 12 characters long");
      }
      const user: UserSummary = {
        id: `usr_${crypto.randomUUID()}`,
        username: "admin",
        display_name: "Administrator",
        security_level: 1000,
        enabled: true,
        has_pin: false,
        password_change_required: false,
      };
      mockUsers = [user];
      mockPasswords.set("admin", password);
      return user as T;
    }
    case "list_users": {
      return mockUsers as T;
    }
    case "save_user": {
      if (mockSecurityLevel < 1000) {
        throw new Error("Administrator permission (Security Level 1000) is required to manage users");
      }
      const input = args?.user as UserAccountInput;
      const idx = mockUsers.findIndex(
        (u) => u.id === input.id || u.username.toLowerCase() === input.username.toLowerCase()
      );
      const password = (input.password ?? "").trim();
      if (idx < 0 && password.length === 0) {
        throw new Error("A password is required when creating an account");
      }
      if (password.length > 0 && password.length < 12) {
        throw new Error("Password must be at least 12 characters long");
      }
      const sum: UserSummary = {
        id: input.id || `usr_${Date.now()}`,
        username: input.username,
        display_name: input.display_name,
        security_level: input.security_level,
        enabled: input.enabled,
        has_pin: Boolean(input.pin) || (idx >= 0 && mockUsers[idx].has_pin),
        password_change_required: false,
      };
      if (password.length > 0) mockPasswords.set(input.username, password);
      if (input.pin) mockPins.set(input.username, input.pin);
      if (idx >= 0) {
        mockUsers[idx] = sum;
      } else {
        mockUsers.push(sum);
      }
      return sum as T;
    }
    case "delete_user": {
      if (mockSecurityLevel < 1000) {
        throw new Error("Administrator permission (Security Level 1000) is required to manage users");
      }
      const id = String(args?.userId ?? "");
      if (mockCurrentUser?.id === id) {
        throw new Error("You cannot delete the account you are signed in with");
      }
      mockUsers = mockUsers.filter((u) => u.id !== id);
      return undefined as T;
    }
    case "get_audit":
      return mockAudit as T;
    case "verify_audit":
      return true as T;
    case "get_audit_status":
      return {
        chain_ok: true,
        sink_path: null,
        persisted: false,
        last_error: "Browser mock — the durable audit sink needs tauri:dev",
        in_memory: mockAudit.length,
        appended: mockAudit.length,
      } as T;
    case "write_tag": {
      if (mockSecurityLevel < 100) throw new Error("Role cannot write process values");
      if (mockMode !== "runtime") {
        throw new Error("Process writes are blocked outside Runtime mode");
      }
      if (mockPasswordChangeRequired) {
        throw new Error("Change the default password before writing to the process");
      }
      if (mockProject?.session_config?.pin_challenge_on_write) {
        const pin = String(args?.pin ?? "");
        if (!mockCurrentUser || mockPins.get(mockCurrentUser.username) !== pin) {
          throw new Error("A valid PIN must accompany this write");
        }
      }
      mockAudit.push({
        id: crypto.randomUUID(),
        ts: new Date().toISOString(),
        actor: mockCurrentUser?.username ?? "guest",
        role: mockRole,
        action: "tag.write",
        detail: `${args?.tagId}=${args?.value}`,
        prev_hash: "GENESIS",
        hash: "mock",
      });
      const definition = mockProject?.tags.find((tag) => tag.id === String(args?.tagId ?? ""));
      const requested = Number(args?.value);
      const verifyReadback = definition?.binding.verify_readback !== false;
      const observed = verifyReadback ? requested : 0;
      return {
        tag_id: String(args?.tagId ?? ""),
        requested_value: requested,
        observed_value: observed,
        raw_readback: Math.max(0, Math.round(observed)),
        protocol: "mock",
        verify_readback: verifyReadback,
        matches: Math.abs(requested - observed) <= 0.0001,
      } as T;
    }
    case "ack_alarm":
      return undefined as T;
    case "test_device": {
      const deviceId = String(args?.deviceId ?? "");
      if (!mockProject?.devices.some((device) => device.id === deviceId)) {
        throw new Error("Device not found in the accepted project");
      }
      return { ok: false, message: "Browser mock — use tauri:dev" } as T;
    }
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
  saveProjectFile: (path: string) => call<void>("save_project_file", { path }),
  getSnapshot: () => call<EngineSnapshot>("get_snapshot"),
  startPolling: (deviceId?: string | null) =>
    call<void>("start_polling", { deviceId: deviceId ?? null }),
  stopPolling: () => call<void>("stop_polling"),
  writeTag: (tagId: string, value: number, pin?: string) =>
    call<WriteReceipt>("write_tag", { tagId, value, pin: pin ?? null }),
  ackAlarm: (defId: string) => call<void>("ack_alarm", { defId }),
  setMode: (mode: string) => call<void>("set_mode", { mode }),
  login: (username: string, password: string) =>
    call<UserSummary>("login", {
      username,
      password,
    }),
  logout: () => call<void>("logout"),
  changePassword: (currentPassword: string, newPassword: string) =>
    call<UserSummary>("change_password", { currentPassword, newPassword }),
  bootstrapAdmin: (password: string) =>
    call<UserSummary>("bootstrap_admin", { password }),
  listUsers: () => call<UserSummary[]>("list_users"),
  saveUser: (user: UserAccountInput) => call<UserSummary>("save_user", { user }),
  deleteUser: (userId: string) => call<void>("delete_user", { userId }),
  getAudit: (limit = 200) => call<AuditEntry[]>("get_audit", { limit }),
  verifyAudit: () => call<boolean>("verify_audit"),
  getAuditStatus: () => call<AuditStatus>("get_audit_status"),
  testDevice: (deviceId: string) =>
    call<{ ok: boolean; message: string }>("test_device", { deviceId }),
  getTagValues: () => call<TagValue[]>("get_tag_values"),
  getAlarms: () => call<AlarmInstance[]>("get_alarms"),
};
