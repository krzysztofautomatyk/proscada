# Wymagania

## Narzędzia

| Narzędzie | Wersja | Zastosowanie |
|---|---:|---|
| Node.js | 22 | frontend, walidatory i Tauri CLI |
| npm | 10.9.2 | zależności i skrypty (`packageManager`) |
| Rust | 1.88 | rdzeń Tauri i Modbus |
| cargo-deny | 0.20.2 | advisory, licencje i źródła zależności Rust |
| WebView2 | aktualny | desktopowy interfejs Windows |

Do komunikacji live potrzebny jest sterownik lub symulator Modbus TCP. Wbudowany projekt Water Tank zakłada `127.0.0.1:5020`.

## Instalacja

```powershell
npm ci
cargo install --locked cargo-deny --version 0.20.2
```

Nie instaluj globalnych kopii Vite ani Tauri. Repozytorium określa używane wersje.

## Profile pracy

- **Frontend mock:** bez PLC i bez natywnego backendu.
- **Tauri development:** pełny frontend i rdzeń Rust.
- **Runtime live:** projekt, urządzenie Modbus i rola operatorska.

## Następny krok

Przejdź do [uruchamiania deweloperskiego](development.md).
