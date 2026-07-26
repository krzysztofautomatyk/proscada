import type { TagDefinition, DeviceConfig, ModbusQueryConfig, TagDataType, TagBinding } from "../types";

export interface DevicePollQuery {
  id: string;
  name: string;
  deviceId: string;
  deviceName: string;
  table: TagBinding["table"];
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
  table: TagBinding["table"];
  deviceId: string;
  queryId: string;
  tags: TagDefinition[]; // All tags sharing this physical register address
  primaryTag?: TagDefinition;
  symbol: string;
  tagId: string;
  dataType: TagDataType | "multi" | "unmapped";
  readonly: boolean; // true = app cannot write
  span: number; // 1 for u16/i16/bool, 2 for f32/u32/i32, 4 for f64/u64/i64
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
