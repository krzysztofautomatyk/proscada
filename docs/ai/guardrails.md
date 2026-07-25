# Guardrails dla AI

## Zakazane skróty

- usunięcie kontroli Runtime, roli, writable, quality lub read-back;
- RMW bez single-writer;
- zapis do input/discrete;
- lokalny fake ACK albo fake process success;
- arbitralny JS w kontrolce lub komponencie;
- rozszerzenie Tauri scope bez review;
- sekret w projekcie, logu, skillu lub workflow;
- połączenie agenta z PLC lub siecią OT;
- wyłączenie testu, aby CI przeszło;
- monolityczna dokumentacja.

## Ręczne review

- `scriptRuntime.ts` i `new Function`;
- `set_role`, `set_mode` i nowe Tauri commands;
- `src-tauri/capabilities/default.json`;
- zapis Modbus i plan pollingu;
- łańcuch audytu;
- import `.pscctrl`.

Testy używają fixture, mocka lub lokalnego symulatora, nigdy produkcyjnego urządzenia.

