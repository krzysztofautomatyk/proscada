import type { TagDefinition, DeviceConfig, TagDataType } from "$lib/types";
import type { DevicePollQuery, RegisterMapEntry, RegisterBitEntry, RegisterValidationResult } from "../types/registerMap";
import { SYSTEM_DEVICE_ID } from "./systemTagsService";

/**
 * Calculates how many 16-bit registers a data type occupies.
 */
export function getDataTypeRegisterSpan(dataType?: TagDataType | string, stringLength?: number): number {
  switch (dataType) {
    case "u32":
    case "i32":
    case "f32":
      return 2;
    case "u64":
    case "i64":
    case "f64":
      return 4;
    case "string":
      return Math.max(1, Math.ceil((stringLength || 32) / 2));
    default:
      return 1;
  }
}

/**
 * Extracts available poll query blocks for a given PLC device.
 */
export function extractDevicePollQueries(device?: DeviceConfig | null): DevicePollQuery[] {
  if (!device) return [];

  const deviceId = device.id;
  const deviceName = device.name;

  if (device.queries && device.queries.length > 0) {
    return device.queries.map((q) => {
      const endAddress = q.start_address + Math.max(1, q.count) - 1;
      const prefix = q.table === "holding" ? "4x" : q.table === "input" ? "3x" : q.table === "coil" ? "0x" : "1x";
      const tableLabel =
        q.table === "holding"
          ? "Holding Registers"
          : q.table === "input"
            ? "Input Registers"
            : q.table === "coil"
              ? "Coils"
              : "Discrete Inputs";

      return {
        id: q.id,
        name: q.name,
        deviceId,
        deviceName,
        table: q.table,
        startAddress: q.start_address,
        count: q.count,
        endAddress,
        pollMs: q.poll_ms ?? device.poll_ms,
        displayLabel: `Modbus TCP ${prefix} R${q.start_address}-${endAddress} (${tableLabel}: ${q.name})`,
      };
    });
  }

  // Fallback query blocks if device has no explicit queries registered yet
  return [
    {
      id: `${deviceId}_default_4x_100_120`,
      name: "Domyślny Blok Holding 4x (R100-120)",
      deviceId,
      deviceName,
      table: "holding",
      startAddress: 100,
      count: 21,
      endAddress: 120,
      pollMs: device.poll_ms,
      displayLabel: `Modbus TCP 4x R100-120 (Holding Registers)`,
    },
    {
      id: `${deviceId}_default_4x_0_50`,
      name: "Blok Rejestrów Holding 4x (R0-50)",
      deviceId,
      deviceName,
      table: "holding",
      startAddress: 0,
      count: 51,
      endAddress: 50,
      pollMs: device.poll_ms,
      displayLabel: `Modbus TCP 4x R0-50 (Holding Registers)`,
    },
    {
      id: `${deviceId}_default_3x_0_50`,
      name: "Blok Rejestrów Wejściowych 3x (R0-50)",
      deviceId,
      deviceName,
      table: "input",
      startAddress: 0,
      count: 51,
      endAddress: 50,
      pollMs: device.poll_ms,
      displayLabel: `Modbus TCP 3x R0-50 (Input Registers - Readonly)`,
    },
    {
      id: `${deviceId}_default_0x_0_32`,
      name: "Blok Cewek Coil 0x (0-32)",
      deviceId,
      deviceName,
      table: "coil",
      startAddress: 0,
      count: 33,
      endAddress: 32,
      pollMs: device.poll_ms,
      displayLabel: `Modbus TCP 0x 0-32 (Coils)`,
    },
  ];
}

/**
 * Builds register map rows for the specified poll query range and tags,
 * supporting MULTIPLE tags per single register address (e.g. word tag + 16 bit tags)
 * and extended multi-register data types (u32/i32/f32, u64/i64/f64, string).
 */
export function buildRegisterMap(query: DevicePollQuery, tags: TagDefinition[]): RegisterMapEntry[] {
  const result: RegisterMapEntry[] = [];
  const queryTags = tags.filter(
    (t) => t.device_id === query.deviceId && t.binding.table === query.table
  );

  const addrToTagsMap = new Map<number, TagDefinition[]>();
  const multiRegContinuations = new Map<number, { parentAddr: number; tag: TagDefinition }>();

  for (const tag of queryTags) {
    const addr = tag.binding.address;
    if (!addrToTagsMap.has(addr)) {
      addrToTagsMap.set(addr, []);
    }
    addrToTagsMap.get(addr)!.push(tag);

    const span = getDataTypeRegisterSpan(tag.data_type, tag.binding.string_length);
    if (span > 1) {
      for (let offset = 1; offset < span; offset++) {
        multiRegContinuations.set(addr + offset, { parentAddr: addr, tag });
      }
    }
  }

  for (let addr = query.startAddress; addr <= query.endAddress; addr++) {
    const registeredTags = addrToTagsMap.get(addr) ?? [];
    const continuation = multiRegContinuations.get(addr);
    const isContinuation = !!continuation && registeredTags.length === 0;

    if (registeredTags.length === 0 && !isContinuation) {
      // Unmapped register
      result.push({
        address: addr,
        table: query.table,
        deviceId: query.deviceId,
        queryId: query.id,
        tags: [],
        symbol: `—`,
        tagId: ``,
        dataType: "unmapped",
        readonly: query.table === "input" || query.table === "discrete",
        span: 1,
        isSpanContinuation: false,
        unit: "",
        scale: 1,
        offset: 0,
        decimals: 0,
        description: `Rejestr ${query.table} ${addr} wolny`,
        liveValue: undefined,
        hexValue: `0x0000`,
      });
      continue;
    }

    if (isContinuation && continuation) {
      const parentTag = continuation.tag;
      const totalSpan = getDataTypeRegisterSpan(parentTag.data_type, parentTag.binding.string_length);
      const partIndex = addr - continuation.parentAddr + 1;

      result.push({
        address: addr,
        table: query.table,
        deviceId: query.deviceId,
        queryId: query.id,
        tags: [parentTag],
        primaryTag: parentTag,
        symbol: `↳ (część ${partIndex}/${totalSpan} ${parentTag.data_type.toUpperCase()} ${parentTag.name})`,
        tagId: parentTag.id,
        dataType: parentTag.data_type,
        readonly: !parentTag.binding.writable,
        span: totalSpan,
        isSpanContinuation: true,
        parentAddress: continuation.parentAddr,
        unit: parentTag.unit || "",
        scale: parentTag.scale ?? 1,
        offset: parentTag.offset ?? 0,
        decimals: parentTag.decimals ?? 0,
        description: `Rejestr kontynuacji ${parentTag.data_type} dla ${parentTag.id}`,
        liveValue: undefined,
        hexValue: `0x0000`,
      });
      continue;
    }

    const primaryTag = registeredTags.find((t) => t.data_type !== "bool") || registeredTags[0];
    const isReadonly =
      query.table === "input" ||
      query.table === "discrete" ||
      query.table === "system" ||
      registeredTags.every((t) => !t.binding.writable);

    let bits: RegisterBitEntry[] | undefined = undefined;
    if (query.table === "holding" || query.table === "input" || query.table === "memory") {
      bits = Array.from({ length: 16 }, (_, i) => {
        const bitTag = registeredTags.find((t) => t.data_type === "bool" && t.binding.bit === i);
        return {
          bitIndex: i,
          tagId: bitTag?.id,
          tagName: bitTag?.name,
          description: bitTag?.description || (bitTag ? `Bit ${i}: ${bitTag.name}` : `Bit ${i} rezerwa`),
          readonly: bitTag ? !bitTag.binding.writable : isReadonly,
          state: false,
        };
      });
    }

    let symbolDisplay = primaryTag.name;
    if (registeredTags.length > 1) {
      const bitCount = registeredTags.filter((t) => t.data_type === "bool").length;
      if (bitCount > 0 && primaryTag.data_type !== "bool") {
        symbolDisplay = `${primaryTag.name} (+${bitCount} bitów)`;
      } else if (registeredTags.length > 1) {
        symbolDisplay = `${registeredTags[0].name} + ${registeredTags.length - 1} tagów`;
      }
    }

    const overallType =
      registeredTags.length === 1
        ? registeredTags[0].data_type
        : registeredTags.every((t) => t.data_type === "bool")
          ? "bool"
          : "multi";

    const span = getDataTypeRegisterSpan(primaryTag.data_type, primaryTag.binding.string_length);

    result.push({
      address: addr,
      table: query.table,
      deviceId: query.deviceId,
      queryId: query.id,
      tags: registeredTags,
      primaryTag,
      symbol: symbolDisplay,
      tagId: primaryTag.id,
      dataType: overallType,
      readonly: isReadonly,
      span,
      isSpanContinuation: false,
      unit: primaryTag.unit || "",
      scale: primaryTag.scale ?? 1,
      offset: primaryTag.offset ?? 0,
      decimals: primaryTag.decimals ?? 0,
      description: primaryTag.description || `${registeredTags.length} zmiennych na rejestrze R${addr}`,
      bits,
      liveValue: undefined,
      hexValue: `0x0000`,
    });
  }

  return result;
}

/**
 * Validates a single tag definition against project rules, Modbus register ranges, and data type spans.
 */
export function validateRegisterTag(
  tag: Partial<TagDefinition>,
  existingTags: TagDefinition[],
  query?: DevicePollQuery | null
): RegisterValidationResult {
  const errors: string[] = [];
  const warnings: string[] = [];

  if (!tag.name || !tag.name.trim()) {
    errors.push("Nazwa zmiennej (Friendly Name) jest wymagana.");
  }

  if (!tag.id || !tag.id.trim()) {
    errors.push("Identyfikator tagu (Tag ID) jest wymagany.");
  } else {
    const isDuplicateId = existingTags.some(
      (t) => t.id.toLowerCase() === tag.id!.toLowerCase()
    );
    if (isDuplicateId) {
      errors.push(`Tag o ID '${tag.id}' już istnieje w projekcie.`);
    }
  }

  if (!tag.device_id && tag.binding?.table !== "memory" && tag.binding?.table !== "system") {
    errors.push("Wymagany jest wybór urządzenia (Device ID).");
  }

  const addr = tag.binding?.address;
  if (tag.binding?.table !== "memory" && tag.binding?.table !== "system") {
    if (addr === undefined || addr < 0 || addr > 65535) {
      errors.push("Adres rejestru Modbus musi zawierać się w przedziale 0..65535.");
    }
  }

  const span = getDataTypeRegisterSpan(tag.data_type, tag.binding?.string_length);

  if (query && addr !== undefined) {
    if (addr < query.startAddress || addr > query.endAddress) {
      warnings.push(
        `Adres ${addr} znajduje się poza domyślnym zakresem zapytania (${query.startAddress}..${query.endAddress}).`
      );
    }

    if (span > 1 && addr + span - 1 > query.endAddress) {
      warnings.push(
        `Zmienna ${tag.data_type} zajmuje ${span} rejestry (${addr}..${addr + span - 1}) - przekracza koniec bloku.`
      );
    }
  }

  if (tag.data_type === "bool" && (tag.binding?.table === "holding" || tag.binding?.table === "memory")) {
    const bit = tag.binding.bit;
    if (bit === undefined || bit === null || bit < 0 || bit > 15) {
      errors.push("Dla zmiennej bitowej Bit Index musi wynosić 0..15 (LSB=0).");
    } else if (tag.binding?.table === "holding") {
      const sameBitTag = existingTags.find(
        (t) =>
          t.device_id === tag.device_id &&
          t.binding.table === "holding" &&
          t.binding.address === addr &&
          t.data_type === "bool" &&
          t.binding.bit === bit &&
          t.id !== tag.id
      );
      if (sameBitTag) {
        errors.push(`Bit ${bit} na rejestrze R${addr} jest już zajęty przez tag '${sameBitTag.name}' (${sameBitTag.id}).`);
      }
    }
  }

  if (
    tag.binding?.table === "holding" &&
    tag.binding?.bit !== undefined &&
    tag.binding?.bit !== null &&
    tag.binding?.writable
  ) {
    if (tag.binding.bit_write_mode === "read_modify_write" && !tag.binding.single_writer) {
      warnings.push("RMW na bicie wymaga 'single_writer: true' ze względów bezpieczeństwa procesu SCADA.");
    }
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
  };
}

/**
 * Toggles readonly state for tag binding.
 */
export function setTagReadonly(tag: TagDefinition, readonly: boolean): TagDefinition {
  return {
    ...tag,
    binding: {
      ...tag.binding,
      writable: !readonly,
    },
  };
}
