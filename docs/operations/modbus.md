# Konfiguracja Modbus

## Urządzenie

Urządzenie definiuje host, port, Unit ID, poll interval, timeout i enabled.

Host może być adresem IPv4, IPv6 lub nazwą DNS — nazwa jest rozwiązywana przez
resolver systemowy z limitem czasu równym `timeout_ms`.

## Połączenia

Engine utrzymuje jedno połączenie odczytowe oraz jedno serializowane połączenie
zapisowe na urządzenie. Zapisy nie otwierają nowego gniazda TCP dla każdej komendy,
więc seria poleceń operatora nie wyczerpuje limitu połączeń sterownika. Błąd wymiany
zamyka i odtwarza sesję zapisową.

## Tag

Tag określa:

- device ID;
- typ danych i kolejność słów;
- tabelę;
- adres PDU zero-based;
- opcjonalny bit;
- writable i `min_security_level`;
- scale, offset, decimals i unit.

## Plan odczytu

Engine grupuje ciągłe adresy osobno dla każdej tabeli i rezerwuje wszystkie rejestry
zajmowane przez tag. Odległe adresy, np. HR100 i HR1000, tworzą osobne bloki.

## Zapis bitu

W **Variables / Tags** wybierz:

- `FC22 mask` — wariant zalecany;
- `FC03+FC06 RMW` — wymaga zaznaczenia Single writer.

Opcja Verify wymaga utrzymania wartości podczas natychmiastowego read-back.

## Diagnostyka

Status toolbar pokazuje ONLINE/OFFLINE, liczbę cykli, czas ostatniego cyklu i ostatni błąd.

## Powiązane

- [Typy danych i kolejność słów](data-types.md)

