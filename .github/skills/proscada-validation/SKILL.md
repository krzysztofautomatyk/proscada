---
name: proscada-validation
description: Run and interpret the ProScada validation gate — svelte-check, widget-catalog and docs validators, Vite build, and Rust cargo tests — and work around a Windows-locked target EXE by using an isolated cargo target dir. Use before finishing any change, when a validator fails, or when cargo build is blocked because the app EXE is running/locked.
---

# proscada-validation

## When to use
- Verifying any change before completion, or diagnosing a failing validator.
- `cargo build` fails on Windows with a locked/`Access is denied` target executable.

## Full gate (run the subset relevant to your change)
```powershell
npm ci                     # exact install from package-lock.json
npm run check              # svelte-check (types)
npm run validate:widgets   # 35/35 controls, renderers, 33 migrations
npm run validate:docs      # modular docs, per-control docs, links
npm run validate:ai        # instructions, agents, skills and workflows
npm run validate:yaml      # parse GitHub YAML deterministically
npm test                   # execute every src/**/*.test.ts file
npm run build              # Vite production build
cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo build --locked --manifest-path src-tauri/Cargo.toml
npm audit --audit-level=high
cargo deny --manifest-path src-tauri/Cargo.toml check advisories licenses sources
```
Scripts live in `package.json` and `scripts/validate-*.mjs`; AI/instruction validators added by the repo (e.g. under `.github/`) run the same way — invoke whatever `validate:*` scripts exist.

## Choose the smallest command
- Widget/registry/renderer change → `npm run validate:widgets` (+ `npm run check`).
- Docs change → `npm run validate:docs`.
- Rust change → `cargo test --manifest-path src-tauri/Cargo.toml`.
- Frontend types/UI → `npm run check` then `npm run build`.

## Locked-EXE workaround (isolated cargo target)
If the built app is running and locks `src-tauri\target`, build/test into a separate directory instead of killing processes:
```powershell
$env:CARGO_TARGET_DIR = Join-Path $env:TEMP "proscada-target-ai"
cargo test --manifest-path src-tauri/Cargo.toml
```
Use a path inside the repo; remove the env var afterwards for normal builds.

## Guardrails
- Do not "fix" validators by weakening their assertions (35/35 canonical controls, 45/45 Toolbox docs, 33 migrations, read-back tests, doc limits).
- Prefer an isolated target dir over force-terminating a locked process.
- Run the targeted validator first; escalate to the full gate only when the change is cross-cutting.
- Treat any non-zero exit as a blocker; read the printed error and fix the root cause.
- `npm run check` rejects every Svelte warning; do not introduce warning
  allowlists that hide debt.
- CI owns the cross-platform `tauri build --no-bundle` smoke matrix. Tagged
  release candidates are unsigned until an organization-specific signing job
  is explicitly configured.
