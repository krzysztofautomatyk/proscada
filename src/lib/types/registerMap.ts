import type { TagDefinition, DeviceConfig, ModbusQueryConfig } from "../types";

export interface DevicePollQuery {
  id: string;
  name: string;
  deviceId: string;
  deviceName: string;
  table: "holding" | "input" | "coil" | "discrete";
  startAddress: number;
  count: number;
  endAddress: number;
  pollMs?: number | null;
  displayLabel: string;
}

export interface RegisterBitEntry {
  bitIndex: number; // 0..15
  tagId?: string;
  tagName?: string;
  description?: string;
  state?: boolean | number;
  readonly?: boolean;
}

export interface RegisterMapEntry {
  address: number;
  table: "holding" | "input" | "coil" | "discrete";
  deviceId: string;
  queryId: string;
  tags: TagDefinition[]; // All tags sharing this physical register address
  primaryTag?: TagDefinition;
  symbol: string;
  tagId: string;
  dataType: "bool" | "u16" | "i16" | "f32" | "multi" | "unmapped";
  readonly: boolean; // true = app cannot write
  span: number; // 1 for u16/i16/bool, 2 for f32
  isSpanContinuation: boolean;
  parentAddress?: number;
  unit: string;
  scale: number;
  offset: number;
  decimals: number;
  description: string;
  bits?: RegisterBitEntry[];
  liveValue?: string | number;
  hexValue?: string;
}

export interface RegisterValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}
