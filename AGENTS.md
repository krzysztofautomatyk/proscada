# ProScada — instrukcje dla agentów AI

## Misja

Rozwijaj ProScada jako wysokiej niezawodności Designer i Runtime SCADA. Priorytety: poprawność procesu, jawna jakość danych, bezpieczne zapisy, centralne alarmy i utrzymywalna architektura.

ProScada nie jest safety PLC. Watchdog, interlock, permissive i funkcje bezpieczeństwa pozostają w sterowniku.

## Przeczytaj przed zmianą

1. `docs/README.md`
2. dokument domenowy wskazany przez zadanie;
3. odpowiedni skill z `.github/skills/`;
4. istniejący Registry, renderer lub moduł Rust przed dodaniem kodu.

## Mapa repozytorium

| Ścieżka | Odpowiedzialność |
|---|---|
| `src/` | Svelte 5, Designer i Runtime |
| `src/lib/components/widgets/registry/` | źródło prawdy Toolboxa |
| `src/lib/components/widgets/catalog/` | osobne renderery kontrolek |
| `src/lib/components/widgets/shared/` | kontrakty i helpery generyczne |
| `src-tauri/src/` | engine, Modbus, alarmy, audyt i schema |
| `docs/` | mała dokumentacja modułowa |
| `scripts/` | walidatory kontraktów repo |
| `.github/skills/` | playbooki zadań dla agentów |
| `src-tauri/capabilities/` | natywne uprawnienia Tauri |

## Nienaruszalne reguły

- Nie wykonuj zapisu procesu poza Runtime.
- Backend egzekwuje rolę, `min_security_level` tagu, writable i quality `Good`.
- Rola pochodzi wyłącznie z `login`; nie ma komendy ustawiającej rolę z UI.
- Silnik startuje jako `Viewer` bez użytkownika; brak snapshotu = brak uprawnień.
- Wejście w Designer wymaga roli Engineer lub Administrator.
- `save_project_in_memory` nigdy nie nadpisuje bazy użytkowników.
- Wartość poza zakresem typu jest odrzucana, nigdy obcinana.
- Bit holding register ma zakres `0..15`, bit 0 = LSB.
- Preferuj FC22; RMW wymaga `single_writer=true`.
- Samokasująca komenda używa `verify_readback=false`, lecz nadal jest odczytywana obserwacyjnie.
- UI nie może deklarować sukcesu procesu na podstawie samego kliknięcia lub ACK transportu.
- Alarm należy do centralnego engine, nie do lokalnego ekranu.
- Nie dodawaj arbitralnego JS, remote script, `javascript:` ani sekretów do projektu/komponentu.
- Skrypty projektowe to deterministyczny język akcji w `scriptRuntime.ts`; nie przywracaj `eval` ani `new Function`.
- Agent i CI nie mogą łączyć się z PLC, skanować sieci OT ani używać sekretów produkcyjnych.
- Zachowaj parytet nowej komendy: Rust command, `generate_handler!`, `api.ts`, mock i typy.
- Nie rozszerzaj Tauri capabilities ani scope plików bez threat modelu i ręcznego review.
- Nie edytuj ręcznie `src-tauri/gen/schemas/`.
- Nie cofaj zmian użytkownika ani nie używaj destrukcyjnych komend Git.

## Kontrolki

Każdy kanoniczny typ wymaga:

1. wpisu Registry;
2. oddzielnego pliku Svelte w folderze kategorii;
3. gałęzi Factory w `WidgetView.svelte`;
4. konfiguracji domyślnej;
5. Properties lub jawnego raw config;
6. obsługi quality/empty/error;
7. osobnego dokumentu `docs/toolbox/controls/.../<widget_type>.md`;
8. przejścia walidatorów.

Nie twórz monolitycznych rendererów obsługujących wiele niepowiązanych typów.

## Workflow zmian

1. Zbadaj istniejący wzorzec i wpływ na schema.
2. Zmień najmniejszy spójny obszar.
3. Dodaj test dla ryzyka, nie tylko happy path.
4. Zaktualizuj dokumentację i indeks.
5. Uruchom najmniejszą adekwatną walidację.
6. Przed zakończeniem uruchom pełną bramkę dla zmienionych warstw.

## Komendy odbiorcze

```powershell
npm run check
npm run validate:widgets
npm run validate:docs
npm run validate:ai
npm run validate:yaml
npm test
npm run build
cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Przy zablokowanym `src-tauri\target\debug\proscada.exe` ustaw tymczasowy `CARGO_TARGET_DIR` zamiast zabijania obcych procesów.

## Dokumentacja

- Jeden temat = jeden mały plik.
- Limit: 10 KB i 160 linii.
- Każda pozycja Toolboxa ma osobny plik.
- Linki muszą przechodzić `validate:docs`.
- Opisuj wyłącznie zachowanie istniejące w kodzie.

## Definition of Done

Zmiana jest gotowa, gdy kod, test, dokumentacja i wszystkie referencje są spójne; walidacje przechodzą; nie ma cichego fallbacku ani niezweryfikowanego twierdzenia o bezpieczeństwie.
