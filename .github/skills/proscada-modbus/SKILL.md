---
name: proscada-modbus
description: Implement or change Modbus TCP master behaviour in the Rust core — polling FC01–FC04, writes FC05/FC06/FC22, single-writer RMW, observed read-back and data quality. Use for modbus or engine polling/write paths, addressing, bit writes and Modbus tests.
---

# proscada-modbus

## When to use
- Adding/altering read polling, tag writes, bit writes, or quality handling in the Rust core.
- Touching `src-tauri/src/modbus/mod.rs` or the polling/write logic in `src-tauri/src/engine/mod.rs`.

## Key files
- Transport + function codes: `src-tauri/src/modbus/mod.rs`.
- Scheduler, write orchestration, quality: `src-tauri/src/engine/mod.rs`.
- Command surface: `src-tauri/src/commands/mod.rs` (`write_tag`, `test_device`, `start_polling`).

## Function-code map (already implemented — reuse, don't reinvent)
- Reads: `read_coils` FC01, `read_discrete` FC02, `read_holding` FC03, `read_input` FC04.
- Writes: `write_coil` FC05, `write_holding` FC06, `write_holding_bit_masked` FC22 (mask write).
- Bit-in-register: `write_holding_bit_rmw` (FC03 → modify → FC06) only for single-writer registers; `bit_write_masks(bit, value)` builds the AND/OR masks.

## Procedure
1. Reuse an existing `read_*`/`write_*` helper; keep new helpers `async` and return `Result<_, ModbusError>`.
2. Every write MUST perform a **read-back**. Require equality when `verify_readback=true`; self-clearing command points may set it false but still use the observed value.
3. For a single bit inside a holding register prefer **FC22** mask write. Only use RMW when the register is `single_writer=true`; otherwise return the existing error steering users to FC22 or a dedicated PLC coil.
4. In the engine, set `Quality::Good` only after a confirmed read; stale tags (age > 3000 ms) degrade to `Uncertain`, failures to `Bad`.
5. Validate bit index in `0..=15`; reject out-of-range with a clear error.

## Guardrails
- Never write blind: observed read-back is mandatory. Do not turn `verify_readback=false` into a skipped read.
- RMW is unsafe with concurrent PLC writers — enforce the `single_writer` binding flag; do not weaken this check.
- Do not mutate multiple bits per masked write; touch only the requested bit.
- Keep addressing explicit (HR/coil) and preserve device timeout/`SocketAddr` handling.

## Validate
```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
```
Covered tests to keep green: `mask_write_sets_only_requested_bit`, `mask_write_resets_only_requested_bit`, `mask_write_rejects_non_physical_bit`.
