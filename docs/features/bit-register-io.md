# Odczyt i zapis bitów

## Odczyt

| Przestrzeń | Funkcja | Zapis |
|---|---|---|
| Coil | FC01 | FC05 |
| Discrete input | FC02 | niedozwolony |
| Holding register | FC03 | FC06 lub FC22 |
| Input register | FC04 | niedozwolony |

Bit holding register ma indeks `0..15`, gdzie bit 0 jest LSB.

## FC22

Preferowany zapis pojedynczego bitu używa:

```text
andMask = NOT (1 << bit)
orMask  = value ? (1 << bit) : 0
```

Pozostałe bity rejestru pozostają niezmienione.

## RMW

FC03 + FC06 jest dopuszczony tylko przy `single_writer=true`. Blokada chroni zapisy wykonywane przez tę instancję SCADA, ale nie zastępuje własności po stronie PLC.

## Read-back

`verify_readback=true` wymaga utrzymania wartości. Dla samokasujących komend ustaw `false`; engine nadal odczytuje stan, lecz nie uznaje szybkiego resetu PLC za błąd.

