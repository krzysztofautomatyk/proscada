# ProScada — Compliance & Residual Risk Notes

## Intended use

Supervisory visualization and operator interaction for **lab / training / integration** of the Water Tank Dual-Pump process simulated by PLC Ladder Simulator Pro.

## Residual risks (accepted for v1)

| Risk | Mitigation | Residual |
|------|------------|----------|
| Network spoofing of Modbus | Localhost default; no encryption on Modbus | Medium outside air-gap |
| Unauthorized write | Role gate + confirm + audit | Low on single seat |
| Stale HMI decisions | Quality + age banner + last-good hold | Low with trained operators |
| Designer changes in ops | Mode separation Runtime vs Designer | Low |
| Audit tampering | Hash chain verification on load | Medium without OS hardening |

## Software safety classification (self-assessed, non-certified)

IEC 62304-style class **B** intent for operator-facing display software when used for non-therapeutic industrial training. **Not certified.**

## Records

- Expert council: `docs/00-expert-council.md`
- Architecture: `docs/ARCHITECTURE.md`
- Integration map: `docs/WATER_TANK_INTEGRATION.md`
