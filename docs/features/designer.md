# Designer

## Panele

- **Solution:** struktura projektu;
- **Toolbox:** kontrolki i własne komponenty;
- **Outline:** obiekty ekranu;
- **Properties:** geometria, tag, styl, animacja i konfiguracja;
- **Styles:** centralny design system;
- **Components:** biblioteka komponentów;
- **Alarms:** grupy i definicje.

Toolbox i Properties mają widoczne pionowe paski oraz przyciski przewijania góra/dół.

## Canvas

Designer obsługuje snap do siatki, zaznaczanie wielu elementów, przesuwanie, zmianę rozmiaru, grupowanie i Z-order.

## Persistence

Każda kontrolka zapisuje `widget_type`, geometrię, tag i `config`. Registry odtwarza właściwy renderer po ponownym otwarciu.

## Bezpieczeństwo

Design nie wykonuje zapisu procesu. Podgląd nie jest potwierdzeniem działania PLC.

