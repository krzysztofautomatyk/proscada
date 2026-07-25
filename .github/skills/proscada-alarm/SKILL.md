---
name: proscada-alarm
description: Implement or change ProScada alarm behaviour — alarm groups, the state lifecycle (Inactive/ActiveUnacked/ActiveAcked/ClearedUnacked), deadband, on/off delays, latching, and centralized backend acknowledgement. Use when a task touches alarm evaluation, ack/reset, deadband/delay/latch logic, or the alarm HMI widgets. Ensures the HMI never fakes an ACK.
---

# proscada-alarm

## When to use
- Adding/changing alarm evaluation, acknowledgement, latching, deadband, or delays.
- Editing alarm HMI widgets (panel/banner/indicator) or the Alarm Manager editor.

## Key files
- Backend evaluation + ack lifecycle: `src-tauri/src/engine/mod.rs` (`evaluate_alarms`, `ack_alarm`, `AlarmState`, `AlarmInstance`).
- Alarm/group definitions: `src/lib/types.ts` (`AlarmDefinition`, `AlarmGroupDefinition`) and `src-tauri/src/project/mod.rs`.
- Command: `src-tauri/src/commands/mod.rs` (`ack_alarm`).
- HMI: `src/lib/components/widgets/catalog/alarms/*` (+ `alarmModel.ts`), editor `src/lib/components/designer/AlarmManagerEditor.svelte`.

## Lifecycle (authoritative — implemented in the engine)
`Inactive → ActiveUnacked → ActiveAcked → (source clears) → Inactive`; and
`ActiveUnacked → (source clears) → ClearedUnacked → (ack) → Inactive`.
Latched alarms stay active until both cleared **and** acknowledged.

## Procedure
1. Define alarm limits, `deadband`, `on_delay_ms`, `off_delay_ms`, `latching`, `priority`, and group in the project model.
2. In `evaluate_alarms`: skip evaluation when tag quality ≠ `Good`; apply deadband on the return-to-normal edge; honour on/off delay timers before changing state; respect `latching`.
3. Acknowledgement and reset happen ONLY in `ack_alarm` on the backend; enforce role (roles that cannot ack must be rejected) and write an audit entry (`alarm.ack` / `alarm.reset`).
4. HMI widgets call the `ack_alarm` command and render the backend `AlarmInstance.state`.

## Guardrails
- **No fake ACK in the HMI**: widgets must not locally clear/ack; they only display backend state and invoke the command.
- Do not evaluate alarms on bad/uncertain quality data.
- Preserve latching semantics — a latched, cleared, acknowledged alarm returns to `Inactive` only when both conditions are met.
- Every ack/reset must be audited and role-checked.

## Validate
```powershell
cargo test --manifest-path src-tauri/Cargo.toml
npm run check
```
Keep the `latching` and analog `deadband` unit tests green.
