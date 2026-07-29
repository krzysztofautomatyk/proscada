# Contributing to ProScada

## Start

1. Read `AGENTS.md`.
2. Install Node 22 and Rust 1.88 from repository pins.
3. Run `npm ci`.
4. Select the relevant guide in `docs/` and skill in `.github/skills/`.

## Change workflow

1. Identify source of truth and compatibility impact.
2. Implement the smallest coherent change.
3. Add a test for the changed risk.
4. Update small, linked documentation.
5. Run relevant validators.
6. Request independent review for OT-sensitive paths.

## Mandatory review

Changes to these paths require OT/security review:

- `src-tauri/src/engine/`;
- `src-tauri/src/modbus/`;
- `src-tauri/src/commands/`;
- `src-tauri/capabilities/`;
- `src/lib/services/scriptRuntime.ts`;
- `.github/workflows/`.

## Full local gate

```powershell
npm ci
npm run check
npm run validate:widgets
npm run validate:docs
npm run validate:ai
npm run validate:yaml
npm test
npm run build
cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo build --locked --manifest-path src-tauri/Cargo.toml
npm audit --audit-level=high
cargo deny --manifest-path src-tauri/Cargo.toml check advisories licenses sources
```

Do not connect tests or agents to a production PLC or OT network.
