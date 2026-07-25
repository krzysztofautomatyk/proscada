# Przegląd architektury

## Kontenery

| Kontener | Technologia | Odpowiedzialność |
|---|---|---|
| UI | Svelte 5 + TypeScript | Designer, Toolbox, Runtime HMI |
| Desktop | Tauri v2 | okno, pliki i granice natywne |
| Core | Rust | tagi, polling, alarmy, zapis i audyt |
| Projekt | JSON | urządzenia, tagi, ekrany i konfiguracja |

## Główne wzorce

- **Registry/Factory:** jedno źródło katalogu Toolboxa.
- **Strategy:** różne ścieżki odczytu i zapisu Modbus.
- **Composite:** własne kontrolki składają się z kontrolek bazowych.
- **Adapter:** frontend mock i natywne komendy Tauri mają wspólne API.
- **Observer:** snapshot tagów zasila kontrolki i alarmy.

## Granica odpowiedzialności

ProScada nadzoruje i wizualizuje proces. Interlocki, watchdog, permissive i funkcje bezpieczeństwa pozostają w PLC.

## Powiązane dokumenty

- [Frontend](frontend.md)
- [Rdzeń Rust](rust-core.md)
- [Przepływ danych](data-flow.md)

