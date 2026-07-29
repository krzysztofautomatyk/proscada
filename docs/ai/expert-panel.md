# Sztab ekspertów AI

## Profile wykonywalne

| Agent | Rola |
|---|---|
| `scada-architect` | architektura przekrojowa |
| `widget-engineer` | kontrolka end-to-end |
| `ot-safety-reviewer` | read-only review OT |
| `documentation-maintainer` | dokumentacja modułowa |
| `ci-validator` | read-only walidacja offline |
| `expert-panel` | pełny sztab, badanie dokumentacji i praca do 9.5/10 |

Profile znajdują się w [`.github/agents`](../../.github/agents/README.md).

## Obowiązkowe review

- Modbus, zapis, alarm engine lub capabilities → `ot-safety-reviewer`;
- nowy typ Toolboxa → `widget-engineer`, potem niezależny review;
- zmiana schema wielu warstw → `scada-architect`;
- reorganizacja docs → `documentation-maintainer`;
- zamknięcie zadania → `ci-validator`.

Agent implementujący nie powinien być jedynym reviewerem własnej logiki safety.

