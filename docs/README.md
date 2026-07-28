# Dokumentacja ProScada

Dokumentacja jest celowo podzielona na małe pliki tematyczne. Ten plik jest jedynym indeksem wejściowym.

## Start

- [Wymagania i instalacja](getting-started/prerequisites.md)
- [Uruchamianie w trybie deweloperskim](getting-started/development.md)
- [Pierwszy projekt](getting-started/first-project.md)
- [Budowanie aplikacji](getting-started/building.md)

## Architektura

- [Przegląd systemu](architecture/overview.md)
- [Frontend Svelte](architecture/frontend.md)
- [Rdzeń Rust](architecture/rust-core.md)
- [Model projektu](architecture/project-model.md)
- [Przepływ danych](architecture/data-flow.md)
- [Granice bezpieczeństwa](architecture/security.md)

## Toolbox

- [Katalog 35 kontrolek](toolbox/README.md)
- [Prymitywy, zasoby i układ](toolbox/primitives-assets-layout.md)
- [Wskaźniki i proces](toolbox/indicators-process.md)
- [Sterowanie i wejścia](toolbox/commands-inputs.md)
- [Dane, historia i nawigacja](toolbox/data-navigation.md)
- [Informacja i alarmy](toolbox/feedback-alarms.md)
- [Narzędzia](toolbox/utilities.md)
- [Szablony procesu](toolbox/process-templates.md)

## Funkcje

- [Designer](features/designer.md)
- [Ekran Startowy](features/start-window.md)
- [Style i czcionki projektu](features/design-system.md)
- [Dynamika i animacje](features/dynamics.md)
- [Centralny Alarm Manager](features/alarm-manager.md)
- [Biblioteka własnych komponentów](features/component-library.md)
- [Generator 20 pompowni](features/pump-station-rollout.md)
- [Odczyt i zapis bitów](features/bit-register-io.md)
- [System Użytkowników i Poziomów Uprawnień](features/security-and-users.md)
- [Standard GitHub Dark Mode](design/github-dark-style.md)

## Eksploatacja i referencje

- [Konfiguracja Modbus](operations/modbus.md)
- [Runtime i jakość danych](operations/runtime-quality.md)
- [Audyt i uprawnienia](operations/audit-security.md)
- [Rozwiązywanie problemów](operations/troubleshooting.md)
- [Format projektu](reference/project-files.md)
- [Skróty klawiaturowe](reference/keyboard.md)
- [Walidacja i testy](reference/validation.md)
- [Mapa Water Tank](reference/water-tank-map.md)
- [Zakres zgodności](reference/compliance.md)
- [Słownik](reference/glossary.md)

## AI i LLM

- [Praca z AI w repo](ai/README.md)
- [Mapa kontekstu](ai/context-map.md)
- [Sztab ekspertów](ai/expert-panel.md)
- [Katalog skills](ai/skills-catalog.md)
- [Workflow agenta](ai/workflow.md)
- [Guardrails](ai/guardrails.md)
- [Copilot cloud agent](ai/cloud-agent.md)

## Reguła rozmiaru

`npm run validate:docs` blokuje dokument większy niż 10 KB lub 160 linii oraz wykrywa uszkodzone linki lokalne.
