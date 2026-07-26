import type { TagDefinition, DeviceConfig } from "$lib/types";
import type { DevicePollQuery, RegisterMapEntry, RegisterBitEntry, RegisterValidationResult } from "../types/registerMap";

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
 * supporting MULTIPLE tags per single register address (e.g. word tag + 16 bit tags).
 */
export function buildRegisterMap(query: DevicePollQuery, tags: TagDefinition[]): RegisterMapEntry[] {
  const result: RegisterMapEntry[] = [];
  const queryTags = tags.filter(
    (t) => t.device_id === query.deviceId && t.binding.table === query.table
  );

  // Group tags by register address: Map<address, TagDefinition[]>
  const addrToTagsMap = new Map<number, TagDefinition[]>();
  const f32Continuations = new Set<number>();

  for (const tag of queryTags) {
    const addr = tag.binding.address;
    if (!addrToTagsMap.has(addr)) {
      addrToTagsMap.set(addr, []);
    }
    addrToTagsMap.get(addr)!.push(tag);

    if (tag.data_type === "f32") {
      f32Continuations.add(addr + 1);
    }
  }

  for (let addr = query.startAddress; addr <= query.endAddress; addr++) {
    const registeredTags = addrToTagsMap.get(addr) ?? [];
    const isContinuation = f32Continuations.has(addr) && registeredTags.length === 0;

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

    if (isContinuation) {
      // Continuation row of an f32 Float register at (addr - 1)
      const parentAddr = addr - 1;
      const parentTags = addrToTagsMap.get(parentAddr) ?? [];
      const parentTag = parentTags.find((t) => t.data_type === "f32");

      result.push({
        address: addr,
        table: query.table,
        deviceId: query.deviceId,
        queryId: query.id,
        tags: parentTag ? [parentTag] : [],
        primaryTag: parentTag,
        symbol: parentTag ? `↳ (część 2 Float ${parentTag.name})` : "↳ continuation",
        tagId: parentTag?.id ?? "",
        dataType: "f32",
        readonly: parentTag ? !parentTag.binding.writable : true,
        span: 2,
        isSpanContinuation: true,
        parentAddress: parentAddr,
        unit: parentTag?.unit || "",
        scale: parentTag?.scale ?? 1,
        offset: parentTag?.offset ?? 0,
        decimals: parentTag?.decimals ?? 0,
        description: parentTag ? `Rejestr kontynuacji f32 dla ${parentTag.id}` : "",
        liveValue: undefined,
        hexValue: `0x0000`,
      });
      continue;
    }

    // Register has one or more tags mapped to it!
    const primaryTag = registeredTags.find((t) => t.data_type !== "bool") || registeredTags[0];
    const isReadonly = query.table === "input" || query.table === "discrete" || registeredTags.every((t) => !t.binding.writable);

    let bits: RegisterBitEntry[] | undefined = undefined;
    if (query.table === "holding" || query.table === "input") {
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
      span: primaryTag.data_type === "f32" ? 2 : 1,
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
      (t) => t.id === tag.id
    );
    if (isDuplicateId) {
      errors.push(`Tag o ID '${tag.id}' już istnieje w projekcie.`);
    }
  }

  if (!tag.device_id) {
    errors.push("Wymagany jest wybór urządzenia (Device ID).");
  }

  const addr = tag.binding?.address;
  if (addr === undefined || addr < 0 || addr > 65535) {
    errors.push("Adres rejestru Modbus musi zawierać się w przedziale 0..65535.");
  }

  if (query) {
    if (addr !== undefined && (addr < query.startAddress || addr > query.endAddress)) {
      warnings.push(
        `Adres ${addr} znajduje się poza domyślnym zakresem zapytania (${query.startAddress}..${query.endAddress}).`
      );
    }

    if (tag.data_type === "f32" && addr !== undefined && addr + 1 > query.endAddress) {
      warnings.push(`Zmienna f32 zajmuje 2 rejestry (${addr} i ${addr + 1}) - przekracza koniec bloku.`);
    }
  }

  if (tag.data_type === "bool" && tag.binding?.table === "holding") {
    const bit = tag.binding.bit;
    if (bit === undefined || bit === null || bit < 0 || bit > 15) {
      errors.push("Dla zmiennej bitowej w rejestrze Holding Bit Index musi wynosić 0..15 (LSB=0).");
    } else {
      // Check if another bit tag is using the exact same bit on this address
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

  // Read-Modify-Write check for holding register bits
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
