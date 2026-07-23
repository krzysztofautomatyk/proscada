# ProScada — Expert Council (World-Class Staff)

> **Status:** Normative for architecture decisions  
> **Product:** ProScada — High-assurance SCADA Engineering + Runtime  
> **Integration target:** PLC Ladder Simulator Pro · Water Tank Dual-Pump · Modbus TCP `:5020`

This document convenes a virtual expert panel. Decisions below are binding for the ProScada codebase.

---

## 1. Convened panel

| Role | Expert | Mandate |
|------|--------|---------|
| Chief Architect (OT/SCADA) | Dr. Elena Vasquez | System boundaries, determinism, C4 topology |
| Industrial Protocols | Capt. Marcus Hale (ex-Navy C5I) | Modbus TCP master discipline, fail-closed writes |
| HMI / Human Factors | Prof. Ingrid Bergström | ISA-101 display hierarchy, operator cognition |
| Medical Device SW (IEC 62304) | Dr. Kenji Okada | Software safety class, lifecycle, residual risk |
| Military Cyber / IEC 62443 | Col. (ret.) Sarah Okonkwo | Zones/conduits, least privilege, audit immutability |
| Alarm Rationalization (ISA-18.2) | Eng. Tomás Rivera | Alarm states, flood control, priority |
| Real-time Data Quality | Dr. Amir Hassan | Quality codes, stale detection, last-good hold |
| Designer UX (WinForms/VS) | Alexandra Chen | Visual Studio Professional shell metaphor |
| Water Process SME | Eng. Piotr Kowalski | Wet-well dual-pump control, tag semantics |
| QA / Validation | Dr. Nadia Ferreira | Verification evidence, lab acceptance tests |

---

## 2. Mission statement

> **ProScada is an engineering workstation and operator runtime for supervisory visualization of industrial processes over Modbus TCP — with military-grade audit discipline and medical-device software hygiene — without claiming SIL/SIL-2/medical CE certification.**

It is designed to **supervise** the Water Tank Dual-Pump station running in **PLC Ladder Simulator Pro**, not to replace a safety PLC or certified medical device.

---

## 3. Binding decisions (ADR summary)

| ID | Decision | Rationale |
|----|----------|-----------|
| **ADR-01** | Desktop shell: **Tauri v2** + **Svelte 5** + **Rust** core | Same stack family as PLC LAD SIM; small attack surface; native Modbus |
| **ADR-02** | ProScada is **Modbus TCP master only** | PLC sim is the slave (`127.0.0.1:5020`); clear authority boundary |
| **ADR-03** | Split modes: **Designer** (engineering) vs **Runtime** (operator) | IEC 62304 design control; prevents accidental config during ops |
| **ADR-04** | Designer UX = **Visual Studio Professional / WinForms** metaphor | Toolbox, Form Designer, Properties grid, Solution Explorer, Output |
| **ADR-05** | Project file: versioned JSON **`.proscada.json`** + content hash | Portable, reviewable, CI-friendly |
| **ADR-06** | Tag quality: **Good / Uncertain / Bad** + data age | OPC UA simplified; operators never trust silent stale data |
| **ADR-07** | Writes: **confirm dialog** + **role gate** + **audit chain** | Fail-closed; every process write is attributable |
| **ADR-08** | Alarms: ISA-18.2 **lite** lifecycle | Active / Acked / Cleared / Shelved (v1 subset) |
| **ADR-09** | Hash-chained **append-only audit log** | Military/medical traceability; tamper-evident |
| **ADR-10** | Single FC03 block read **HR100–121** for Water Tank | One poll → full station image (from PLC map) |
| **ADR-11** | No closed-loop safety claims | Training / lab / integration; E-stop remains hardwired elsewhere |
| **ADR-12** | Localhost-first; no remote script loading | CSP deny-by-default; air-gap friendly |

---

## 4. Non-goals (explicit)

- Not a safety PLC, not SIL-rated, not FDA/MDR medical device certification.
- Not OPC UA / MQTT in v1 (architecture reserved).
- Not multi-user HA cluster in v1 (single workstation).
- Not proprietary historian DB in v1 (in-memory ring + exportable audit).

---

## 5. Acceptance criteria (lab)

1. With PLC LAD SIM **Water tank** running + Modbus slave ON, ProScada Runtime shows live **LEVEL**, **P1/P2 RUN**, and alarms within **1 s** of process change.
2. Designer can open, edit, save the Water Tank form and reload geometry without loss.
3. Operator write of **K_x100** (HR105) is audited and requires Operator+ role when writes enabled on PLC.
4. Disconnecting PLC marks tags **Bad/stale** and freezes last-good numeric display with visible degraded banner.
5. Audit chain verifies on startup.

---

## 6. Compliance posture (honest)

| Framework | Alignment | Certification |
|-----------|-----------|---------------|
| IEC 62443-4-2 (product) | Least privilege, audit, secure defaults | **Not certified** |
| IEC 62304 (medical SW) | Lifecycle docs, risk notes, separation eng/runtime | **Not certified** |
| ISA-18.2 / ISA-101 | Alarm states, display hierarchy | **Aligned practices** |
| NIST SP 800-82 (ICS) | Defense in depth lab guidance | **Aligned practices** |

**Disclaimer:** ProScada implements *engineering practices inspired by* high-assurance domains. It is **not** a certified military, nuclear, or medical product.

---

*Council chair: Dr. Elena Vasquez · ProScada v1.0*
