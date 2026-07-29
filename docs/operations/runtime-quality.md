# Runtime i jakość danych

## Połączenie

**Connect** uruchamia polling wybranego urządzenia. **Stop** zatrzymuje scheduler i oznacza połączenie jako nieaktywne.

## Quality

- **Good:** świeża, poprawna próbka;
- **Uncertain:** próbka przekroczyła próg wieku;
- **Bad:** błąd komunikacji lub brak poprawnego odczytu.

Data Status Indicator prezentuje dodatkowo Simulation, Stale i Disconnected jako stany operatorskie.

## Prezentacja jakości w HMI

Każda kontrolka związana z tagiem przechodzi przez wspólny kontrakt jakości w
`widgets/shared/quality.ts`. Wartość `Bad`, `Uncertain` lub brak taga powoduje
nałożenie zakreskowanej zasłony i etykiety `BAD` / `STALE` / `NO TAG` przez powlokę
`DynamicShell`.

Żadna kontrolka nie podstawia wiarygodnie wyglądającej wartości zastępczej — zamiast
liczby renderowany jest znacznik `––`.

## Zachowanie przy błędzie

Przy błędzie dowolnego bloku cykl urządzenia jest uznany za nieudany. Tagi przechodzą
do `Bad`, alarmy są nadal ewaluowane, ale oznaczane jako zawieszone.

## Zapisy

Backend odrzuca zapis, gdy:

- aplikacja nie jest w Runtime;
- rola nie ma prawa;
- zalogowane konto ma wymuszoną zmianę hasła;
- tag nie jest writable;
- poziom bezpieczeństwa sesji jest niższy niż `binding.min_security_level`;
- jakość live nie jest Good;
- tabela jest read-only;
- konfiguracja bitu jest niebezpieczna;
- wartość wykracza poza zakres typu docelowego.

