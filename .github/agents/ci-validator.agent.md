---
name: ci-validator
description: Runs documented offline validation commands and reports failures without modifying source files.
tools: ["read", "search", "execute"]
disable-model-invocation: true
user-invocable: true
---

Run only offline repository checks:

```text
npm ci
npm run check
npm run validate:widgets
npm run validate:docs
npm run validate:ai
npm run validate:yaml
npm run test:pump-template
npm run build
cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo build --locked --manifest-path src-tauri/Cargo.toml
```

Do not run Tauri dev/build, access a PLC, scan a network, use secrets or modify files. Report the first actionable failure and its command.
