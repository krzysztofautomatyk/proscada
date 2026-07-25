# ProScada Agent Skills

Small, practical [Agent Skills](https://docs.github.com/copilot) that teach an AI LLM how to make **high-assurance** changes to this Tauri + Svelte 5 + Rust SCADA project. Each skill is one folder with a `SKILL.md` (YAML `name` + `description`, a clear "when to use", a step-by-step procedure, guardrails, key files, and the minimal validation commands).

## Catalog

| Skill | Use it when the task is about… |
| --- | --- |
| [proscada-widget](proscada-widget/SKILL.md) | Adding/modifying a canonical Toolbox control: Registry, `WidgetView`, dedicated renderer, Properties and per-item docs. |
| [proscada-modbus](proscada-modbus/SKILL.md) | Modbus master: polling FC01–FC04, writes FC05/FC06/FC22, single-writer RMW, observed read-back, quality and Rust tests. |
| [proscada-alarm](proscada-alarm/SKILL.md) | Alarm groups, state lifecycle, deadband/delays/latching, centralized backend ack — HMI never fakes an ACK. |
| [proscada-component-template](proscada-component-template/SKILL.md) | ComponentTemplate, parameter substitution, `.pscctrl` SHA-256, bulk CSV, pump-station rollout. |
| [proscada-designer](proscada-designer/SKILL.md) | Designer UX: Toolbox/Properties/canvas/panels, accessibility, scrollbars, Designer-vs-Runtime boundary. |
| [proscada-docs](proscada-docs/SKILL.md) | Modular docs: small files, per-control pages, indexes, `validate:docs`, no monolithic docs. |
| [proscada-validation](proscada-validation/SKILL.md) | Running the gate: `check`/`validate:widgets`/`validate:docs`/build, Rust tests, isolated cargo target when the EXE is locked. |
| [proscada-security](proscada-security/SKILL.md) | Fail-closed writes, mode/RBAC/quality, CSP, safe imports, PLC safety boundary, hash-chained audit. |

## How skills are chosen
See [docs/ai/skills-catalog.md](../../docs/ai/skills-catalog.md) for the task → skill mapping. Most non-trivial changes combine a domain skill with **proscada-security** (safety invariants) and **proscada-validation** (the gate).

## Conventions
- One control = one `type`, one renderer file, one doc — the canonical set stays at **35**.
- Every process write is gated (mode + RBAC + quality) and observed by read-back; equality is required according to `verify_readback`.
- Writes, role/mode changes and alarm ACK/reset are audited; do not claim unimplemented persistent SOE.
- Docs stay small (≤10 KB / ≤160 lines); no secrets or machine-absolute paths anywhere.
