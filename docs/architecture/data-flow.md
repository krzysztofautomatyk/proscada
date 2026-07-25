# Przepływ danych

## Odczyt

```text
Project tags
  → dynamiczny plan bloków Modbus
  → FC01 / FC02 / FC03 / FC04
  → dekodowanie raw + bit + scale
  → TagValue(value, bool, quality, timestamp, age)
  → Alarm Engine
  → EngineSnapshot
  → Svelte stores
  → kontrolki
```

## Zapis

```text
Kontrolka
  → onWrite(tagId, value)
  → Tauri write_tag
  → kontrola Runtime / roli / jakości
  → blokada rejestru
  → FC05 / FC06 / FC22 / kontrolowany RMW
  → odczyt obserwacyjny
  → audit
```

## Jakość

Brak odpowiedzi oznacza `Bad`. Stara próbka przechodzi do `Uncertain`. Kontrolki pokazują stan nie tylko kolorem.

## Alarmy

Alarmy są oceniane po udanym pełnym cyklu odczytu urządzenia.

