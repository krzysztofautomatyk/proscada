# Praca z AI w repozytorium ProScada

## Punkt wejścia

1. Przeczytaj [`AGENTS.md`](../../AGENTS.md).
2. Otwórz [mapę kontekstu](context-map.md).
3. Wybierz skill z [katalogu skills](skills-catalog.md).
4. Przy zmianie wysokiego ryzyka użyj właściwego [profilu eksperta](expert-panel.md).
5. Wykonaj kroki z [AI workflow](workflow.md).

## Konfiguracja repo

- `.github/copilot-instructions.md` — instrukcje wspólne;
- `.github/instructions/` — reguły ścieżkowe;
- `.github/agents/` — wyspecjalizowane profile;
- `.github/skills/` — powtarzalne playbooki;
- `copilot-setup-steps.yml` — deterministyczne środowisko;
- `ci.yml` — bramka PR;
- `llms.txt` — indeks kontekstu.

## Bezpieczeństwo

Cloud agent i CI pracują offline względem OT. Nie mogą łączyć się z PLC, skanować sieci ani używać produkcyjnych sekretów.

## Walidacja

`npm run validate:ai` sprawdza instrukcje, profile, skills, workflow, piny akcji i ograniczenia reviewerów.

