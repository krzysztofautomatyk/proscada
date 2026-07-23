# ProScada

**High-assurance SCADA engineering workstation & operator runtime** with **Modbus TCP master**, a **Visual Studio Professional / WinForms-style form designer**, and a factory project for the **Water Tank Dual-Pump** station from **PLC Ladder Simulator Pro**.

| Layer | Stack |
|--------|--------|
| Desktop shell | **Tauri v2** |
| Real-time core | **Rust** (Modbus master, tag engine, ISA-18.2-lite alarms, hash-chained audit) |
| UI | **Svelte 5** + TypeScript |
| Fieldbus | **Modbus TCP master** → PLC LAD SIM slave (`127.0.0.1:5020`) |

> **Disclaimer:** Training / lab / integration tool. Practices inspired by military cyber (IEC 62443), medical software lifecycle (IEC 62304), and ISA-101/18.2 HMI/alarm discipline. **Not certified** as a safety PLC, medical device, or military system. See [docs/COMPLIANCE.md](./docs/COMPLIANCE.md).

---

## Expert council

Architecture was defined by a multi-discipline panel (OT architect, C5I protocols, medical SW, cyber, alarm rationalization, process SME, VS designer UX). Full record:

- [docs/00-expert-council.md](./docs/00-expert-council.md)
- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)
- [docs/WATER_TANK_INTEGRATION.md](./docs/WATER_TANK_INTEGRATION.md)

---

## Features

### Designer (Visual Studio metaphor)

- **Solution Explorer** — devices, tags, forms, alarms  
- **Toolbox** — drag-and-drop widgets (label, numeric, lamp, tank, panel, write button, bar)  
- **Form Designer** — absolute canvas, snap-to-grid, 8-handle resize, z-order  
- **Properties** — WinForms-style property grid  
- **Output** — connection log, alarms, hash-chained audit trail  

### Runtime

- Live tag quality (**Good / Uncertain / Bad**) + age  
- Modbus block read **HR100–121** (full Water Tank image)  
- Operator writes with **confirm + role gate + audit**  
- Alarm list with ack (ISA-18.2 lite)  

### Assurance posture

- Roles: Viewer / Operator / Engineer / Administrator  
- Mode separation: Designer vs Runtime  
- Append-only **SHA-256 hash-chained** audit log  
- Project content hash  
- CSP deny-by-default (Tauri)  

---

## Quick start

### Prerequisites

- Node.js 20+  
- Rust 1.77+  
- PLC Ladder Simulator Pro (optional but recommended for live data)

### Install & run

```bash
cd ProScada
npm install
npm run tauri:dev
```

Frontend-only (mock data):

```bash
npm run dev
# → http://localhost:1430
```

### Lab procedure (Water Tank)

1. Start **PLC Ladder Simulator Pro**  
2. Toolbar → **Water tank** → **RUN** → Modbus TCP slave **ON** (port **5020**)  
3. Enable **Allow SCADA writes** if you need setpoints / K from ProScada  
4. In ProScada: **Connect Modbus** → **▶ Run**  
5. Watch LEVEL, P1/P2 lamps, DEMAND, alarms  

---

## Project layout

```
ProScada/
  docs/                 Expert council, architecture, compliance
  projects/             Exported .proscada.json samples
  src/                  Svelte 5 UI (VS shell + designer + runtime)
  src-tauri/            Rust core (Modbus, engine, audit, project)
```

Factory project is embedded in Rust (`water_tank_project()`) and auto-loaded on startup.

---

## Keyboard

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd+S` | Save project (recompute hash) |
| `Delete` | Delete selected widget (Designer) |

---

## Integration map (summary)

| HR | Tag | Meaning |
|----|-----|---------|
| 100 | DI_PACK | Inputs I0–I15 packed |
| 101 | DO_PACK | Outputs Q0–Q15 packed |
| 104 | LEVEL_cm | Level 0…1000 |
| 105 | K_x100 | Inflow factor |
| 108–110 | SP_* | Stop / P1 / P2 setpoints |
| bits | P1_RUN, ALM_*, DEMAND… | Decoded in ProScada tags |

Full table: [docs/WATER_TANK_INTEGRATION.md](./docs/WATER_TANK_INTEGRATION.md)  
PLC source: `PLC LAD SIMULATOR MODBUS TCP/docs/WATER_TANK_MODBUS_MAP.md`

---

## License

[MIT](./LICENSE)
