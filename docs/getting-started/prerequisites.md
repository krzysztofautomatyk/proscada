# Wymagania

## Narzędzia

| Narzędzie | Minimalna wersja | Zastosowanie |
|---|---:|---|
| Node.js | 22 | frontend, walidatory i Tauri CLI |
| npm | dostarczony z Node.js | zależności i skrypty |
| Rust | 1.88 | rdzeń Tauri i Modbus |
| WebView2 | aktualny | desktopowy interfejs Windows |

Do komunikacji live potrzebny jest sterownik lub symulator Modbus TCP. Wbudowany projekt Water Tank zakłada `127.0.0.1:5020`.

## Instalacja

```powershell
npm install
```

Nie instaluj globalnych kopii Vite ani Tauri. Repozytorium określa używane wersje.

## Profile pracy

- **Frontend mock:** bez PLC i bez natywnego backendu.
- **Tauri development:** pełny frontend i rdzeń Rust.
- **Runtime live:** projekt, urządzenie Modbus i rola operatorska.

## Następny krok

Przejdź do [uruchamiania deweloperskiego](development.md).
