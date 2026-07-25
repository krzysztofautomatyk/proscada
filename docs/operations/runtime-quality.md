# Runtime i jakość danych

## Połączenie

**Connect** uruchamia polling wybranego urządzenia. **Stop** zatrzymuje scheduler i oznacza połączenie jako nieaktywne.

## Quality

- **Good:** świeża, poprawna próbka;
- **Uncertain:** próbka przekroczyła próg wieku;
- **Bad:** błąd komunikacji lub brak poprawnego odczytu.

Data Status Indicator prezentuje dodatkowo Simulation, Stale i Disconnected jako stany operatorskie.

## Zachowanie przy błędzie

Przy błędzie dowolnego bloku cykl urządzenia jest uznany za nieudany. Tagi przechodzą do `Bad`, a alarmy nie są automatycznie czyszczone na podstawie niewiarygodnej próbki.

## Zapisy

Backend odrzuca zapis, gdy:

- aplikacja nie jest w Runtime;
- rola nie ma prawa;
- tag nie jest writable;
- jakość live nie jest Good;
- tabela jest read-only;
- konfiguracja bitu jest niebezpieczna.

