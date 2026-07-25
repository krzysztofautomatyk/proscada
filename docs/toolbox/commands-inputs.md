# Sterowanie i wejścia

## Command Button

Warianty:

- set;
- reset;
- toggle;
- momentary;
- wartość;
- akcja typowana.

Przycisk nie komunikuje sukcesu procesu na podstawie samego kliknięcia. Wynik transportu i sprzężenie procesu są odrębnymi stanami.

Momentary wymaga watchdoga w PLC. Bez potwierdzonego watchdoga kontrolka blokuje działanie.

## Wejścia

| Kontrolka | Warianty |
|---|---|
| Numeric Input | field, slider, stepper |
| Boolean Input | checkbox, switch |
| Select Input | select, chips |
| Text Input | text, number |
| Date-Time Range | absolute, relative |

## Walidacja

Wartości numeryczne są sprawdzane względem min, max, step i skończoności. String write nie jest obsługiwany przez bieżący backend Modbus.

Zapisy są blokowane w Designerze i przy jakości innej niż Good.

