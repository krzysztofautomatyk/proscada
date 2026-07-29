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
  → Tauri write_tag(tagId, value, opcjonalny PIN)
  → kontrola sesji / Runtime / roli / jakości
  → blokada rejestru
  → ponowna kontrola sesji i projektu
  → FC05 / FC06 / FC22 / kontrolowany RMW
  → odczyt obserwacyjny
  → WriteReceipt + audit
```

## Jakość

Brak odpowiedzi oznacza `Bad`. Stara próbka przechodzi do `Uncertain`. Kontrolki pokazują stan nie tylko kolorem.

## Alarmy

Alarmy są oceniane po udanym pełnym cyklu odczytu urządzenia.
ACK, latching i stan aktywny są atomowo utrwalane w journalu powiązanym z ID
projektu i skrótem definicji. Po restarcie stan pozostaje zawieszony do świeżego
odczytu.
