# Centralny Alarm Manager

## Grupy

Panel **Alarms** tworzy hierarchię grup, np.:

```text
Zakład A
  Pompownie
    PS_001
    PS_002
```

Grupa ma stabilne ID, parent i opcjonalne objectId.

## Definicja alarmu

Konfiguracja obejmuje tag, grupę, priorytet, komunikat, deadband, ON delay, OFF delay i latching.

## Lifecycle

- pojawienie → Active Unacked;
- ACK → Active Acked;
- zanik przed ACK → Cleared Unacked;
- zanik po ACK → Inactive;
- latched alarm pozostaje aktywny do resetu.

## Widoki

Alarm Panel filtruje stan, priorytet i grupę. Banner pokazuje najważniejszy alarm, a Indicator agreguje obiekt.

## Ograniczenie

Obecny model nie implementuje pełnego shelving/OOS workflow ISA-18.2. UI rozpoznaje shelved w danych konfiguracyjnych, ale governance wymaga dalszego backendu.

