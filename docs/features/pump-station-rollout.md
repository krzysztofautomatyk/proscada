# Generator 20 pompowni

## Template

Wbudowany komponent **Pompownia 2P + 2F + 1S** zawiera:

- dwie pompy;
- dwa pływaki;
- sondę poziomu;
- alarm roll-up;
- compact faceplate.

## CSV

Wymagane kolumny:

```csv
objectId,name,tagPrefix,alarmGroup,location,deviceId,baseAddress
```

Generator sprawdza duplikaty, liczbę pól i adres bazowy. Commit tworzy atomowo widgety, tagi, grupy alarmów oraz alarmy.

Po wygenerowaniu zapisz projekt przed przejściem do Runtime, aby Rust engine wczytał nową mapę tagów.

## Mapa jednej instancji

- `baseAddress`: statusy pomp, faults i pływaki;
- `baseAddress + 1`: poziom;
- `baseAddress + 2`: bity komend.

Komendy są samokasujące: `verify_readback=false`, ale engine nadal wykonuje odczyt obserwacyjny.

## Relacja do Water Tank

Fabryczny projekt Water Tank używa wyłącznie tagów istniejących w jego mapie PLC i nie definiuje zdalnych `P1_StartCmd`/`P1_StopCmd`. Generic Faceplate ma wtedy przyciski komend zablokowane.

Generator pompowni jest osobnym modelem obiektu. Tworzy writable `P1_StartCmd`, `P1_StopCmd`, `P2_StartCmd` i `P2_StopCmd` pod adresami wynikającymi z CSV. Nie wolno dopisywać tych bitów do Water Tank bez zgodnej mapy PLC.

## Układ

Instancje są rozmieszczane po cztery w wierszu. Ekran powiększa się automatycznie dla 20 obiektów.
