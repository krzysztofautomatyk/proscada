# ProScada — System Architecture

## 1. Context

```
┌─────────────────────┐  Modbus TCP   ┌──────────────────────────┐
│ PLC Ladder Sim Pro  │◀── master ───│        ProScada           │
│ Water Tank LAD      │   :5020      │  Designer │ Runtime       │
│ Modbus TCP slave    │──────────────│  Rust engine + Svelte UI  │
└─────────────────────┘              └──────────────────────────┘
```

## 2. Containers

| Container | Tech | Responsibility |
|-----------|------|----------------|
| `proscada-ui` | Svelte 5 + TS | VS-style shell, designer canvas, runtime HMI |
| `proscada-core` | Rust (Tauri) | Modbus master, tag store, alarms, audit, project I/O |
| Project files | JSON | Tags, devices, forms, widgets, alarm defs |

## 3. Data path (hot)

```
Scheduler (250–1000 ms)
    → Modbus FC03 HR100 qty=22
    → Decode bit packs + registers → TagStore
    → Quality evaluation (timeout / exception → Bad)
    → Alarm evaluator
    → UI poll / event snapshot (JSON for v1 engineering path)
```

Write path:

```
Operator action → Role check → Confirm → FC06/FC16
    → Audit chain entry → Tag optimistic update / re-poll
```

## 4. Designer (Visual Studio metaphor)

| VS concept | ProScada |
|------------|----------|
| Solution Explorer | Project tree (Devices, Tags, Forms, Alarms) |
| Toolbox | Widget palette (displays, indicators, gauges, controls) |
| Form Designer | Absolute canvas, snap grid, move/resize handles |
| Properties window | Selected widget/property bag |
| Output window | Connection log, poll stats, errors |
| Start Debugging | Switch to Runtime mode |

## 5. Security model (v1)

- Roles: `Viewer` < `Operator` < `Engineer` < `Administrator`
- Writes: Operator+
- Project edit: Engineer+
- Audit export / role change: Administrator
- CSP: self only; no remote scripts
- Modbus target defaults to loopback

## 6. Water Tank mapping (canonical)

See [WATER_TANK_INTEGRATION.md](./WATER_TANK_INTEGRATION.md) — mirrors
`PLC LAD SIMULATOR MODBUS TCP/docs/WATER_TANK_MODBUS_MAP.md`.
