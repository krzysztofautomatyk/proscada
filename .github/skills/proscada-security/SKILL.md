---
name: proscada-security
description: Enforce ProScada high-assurance invariants — fail-closed writes, mode/RBAC/quality gates, observed read-back, CSP, safe imports, the PLC boundary and hash-chained audit. Use for writes, roles, modes, imports, CSP/capabilities and security review.
---

# proscada-security

## When to use
- Any change to writes, roles, engine mode, imports, CSP/capabilities, or audit.
- Security review of the control path from HMI down to the PLC.

## Key files
- Write gating: `src-tauri/src/engine/mod.rs` (`write_tag`), commands `src-tauri/src/commands/mod.rs`.
- Roles/mode: `Role` in `src/lib/types.ts`, `set_role`/`set_mode` in engine/commands.
- Audit (hash-chained): `src-tauri/src/audit/mod.rs` (`append`, `verify_audit`).
- CSP + capabilities: `src-tauri/tauri.conf.json` (`security.csp`), `src-tauri/capabilities/`.
- Integrity: `src-tauri/src/project/mod.rs` (`content_hash`, SHA-256).

## Invariants (fail closed)
1. **Mode**: process writes are rejected unless engine `mode == "runtime"`. Never bypass for Designer/preview.
2. **Quality**: refuse to write a tag whose quality is not `Good`.
3. **RBAC**: check `Role` before writes and before alarm ack/reset; roles without permission are rejected.
4. **Read-back**: every Modbus write is observed; mismatch fails when `verify_readback=true`, while self-clearing commands may disable equality only.
5. **Single-writer RMW**: bit RMW only when `single_writer=true`; otherwise require FC22 or a dedicated coil.
6. **Audit**: writes, mode/role changes and alarm ack/reset append to the in-memory hash chain; keep `verify_audit` valid and do not claim persistent SOE.

## Procedure
1. Add new privileged actions behind the same mode + RBAC + quality checks; return a clear error on denial (never silently succeed).
2. Keep the Tauri command surface narrow and audited; do not widen `capabilities/` or CSP without necessity.
3. For imports/`.pscctrl`/project files, validate structure and SHA-256 before trusting; reject on mismatch.
4. Append an audit entry for any state-changing operation.

## Guardrails
- Fail closed on any doubt (mode, role, quality, checksum) — deny, don't default to allow.
- No secrets, tokens, or machine-absolute paths in code, config, project files, or templates.
- Do not loosen CSP (`default-src 'self'`, no remote script) or add broad filesystem/network capabilities.
- Preserve the PLC safety boundary: the HMI cannot reach the device except through the gated, audited, read-back-verified write path.
- Never remove or fake audit entries.

## Validate
```powershell
cargo test --manifest-path src-tauri/Cargo.toml
npm run check
```
Confirm write-gate and read-back tests stay green and `verify_audit` still holds.
