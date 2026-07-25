# Wskaźniki i wizualizacja procesu

## Wskaźniki

- **State Indicator:** mapuje bit, enum lub wartość na tekst, ikonę i kolor.
- **Numeric Value:** prezentuje liczbę, jednostkę, precyzję i jakość.
- **Meter:** wariant bar, progress, gauge lub pionowy.
- **Data Status:** Live, Simulation, Uncertain, Stale, Bad i Disconnected.

## Wizualizacja procesu

- **Process Symbol:** pompa, zawór, silnik, zbiornik lub czujnik.
- **Faceplate:** status obiektu, quality, permissive, alarmy i dozwolone komendy.

## ISA-101

Tło pozostaje neutralne. Kolory alarmowe są zarezerwowane dla stanów wymagających uwagi.

Stan musi być rozpoznawalny przez co najmniej dwa kanały, np. tekst + ikonę, a nie sam kolor.

## Quality

Przy jakości `Bad` kontrolka pokazuje wzór i etykietę jakości. Ostatnia wartość nie może wyglądać jak bieżąca próbka Good.

