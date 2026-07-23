/**
 * Telemetry adapter contract — ready for MQTT / OPC-UA without pulling deps yet.
 * ProScada Runtime feeds tags via Modbus; this interface documents the OT path.
 */
import type { WaterTankViewModel } from "$lib/stores/waterTank.svelte";

export interface TankTelemetryPayload {
  levelPercent?: number;
  levelCm?: number;
  inflowLps?: number;
  outflowLps?: number;
  p1Run?: boolean;
  p2Run?: boolean;
  p1Fault?: boolean;
  p2Fault?: boolean;
  demand?: boolean;
  simEn?: boolean;
}

export interface TelemetryAdapter {
  connect(): Promise<void> | void;
  disconnect(): void;
  /** Subscribe to live updates (optional when driven by parent tagMap). */
  onTelemetry?(cb: (p: TankTelemetryPayload) => void): () => void;
}

/** Placeholder MQTT adapter — swap body for mqtt.js in production. */
export function createMockMqttAdapter(
  _brokerUrl = "ws://localhost:8083/mqtt",
): TelemetryAdapter {
  return {
    connect() {
      console.info("[telemetry] Mock MQTT adapter ready (Modbus is primary path)");
    },
    disconnect() {
      /* no-op */
    },
  };
}

/** Map view-model snapshot to a publishable telemetry JSON (edge → HMI reverse). */
export function viewModelToTelemetry(vm: WaterTankViewModel): TankTelemetryPayload {
  return {
    levelCm: vm.levelCm,
    levelPercent: vm.levelPercent,
    p1Run: vm.p1Run,
    p2Run: vm.p2Run,
    p1Fault: vm.p1Fault,
    p2Fault: vm.p2Fault,
    demand: vm.demand,
    simEn: vm.simEn,
  };
}
