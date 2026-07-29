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

## Jakość źródła

Alarm jest ewaluowany w każdym cyklu, także po utracie łączności. Gdy jakość taga
źródłowego jest inna niż `Good`, instancja dostaje `evaluation_suspended = true`
wraz z powodem i znacznikiem czasu. Stan alarmu pozostaje ostatnim wiarygodnym,
ale jest jawnie oznaczony jako nieaktualny — zamrożona lista alarmów nie może
wyglądać jak lista żywa.

Snapshot silnika udostępnia zbiorcze `alarms_suspended`, które Runtime pokazuje jako
znacznik `ALARMS STALE`.

## Widoki

Alarm Panel filtruje stan, priorytet i grupę. Banner pokazuje najważniejszy alarm, a Indicator agreguje obiekt.

## Ograniczenie

Obecny model nie implementuje pełnego shelving/OOS workflow ISA-18.2. UI rozpoznaje shelved w danych konfiguracyjnych, ale governance wymaga dalszego backendu.

