# State Indicator

| Pole | Wartość |
|---|---|
| ID | `IND-STATE` |
| Typ | `state_indicator` |
| Plik | `catalog/indicators/StateIndicatorWidget.svelte` |

Łączy funkcje Bit Lamp i WordLamp.

## Warianty

- `bit` — odczyt bitu `bitIndex` z wartości;
- enum/range — mapowanie liczby na stan.

## Najważniejsze pola

`variant`, `title`, `bitIndex`, `states`, `unknownLabel`.

Format stanu:

```text
wartość|etykieta|kolor|ikona
```

## Quality

Quality Bad wymusza odrębny wzór i komunikat. Nieznana wartość używa stanu UNKNOWN.

