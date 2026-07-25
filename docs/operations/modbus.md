# Konfiguracja Modbus

## Urządzenie

Urządzenie definiuje host, port, Unit ID, poll interval, timeout i enabled.

## Tag

Tag określa:

- device ID;
- typ danych;
- tabelę;
- adres PDU zero-based;
- opcjonalny bit;
- writable;
- scale, offset, decimals i unit.

## Plan odczytu

Engine grupuje ciągłe adresy osobno dla każdej tabeli. Odległe adresy, np. HR100 i HR1000, tworzą osobne bloki.

## Zapis bitu

W **Variables / Tags** wybierz:

- `FC22 mask` — wariant zalecany;
- `FC03+FC06 RMW` — wymaga zaznaczenia Single writer.

Opcja Verify wymaga utrzymania wartości podczas natychmiastowego read-back.

## Diagnostyka

Status toolbar pokazuje ONLINE/OFFLINE, liczbę cykli, czas ostatniego cyklu i ostatni błąd.

