# Narzędzia

## QR Code

Kontrolka generuje kod QR lokalnie, bez komunikacji z usługą zewnętrzną.

Źródłem może być:

- tekst statyczny z `config.text`;
- bieżąca wartość przypisanego taga.

## Konfiguracja

| Pole | Znaczenie |
|---|---|
| `source` | `static` albo `tag` |
| `text` | payload statyczny |
| `caption` | podpis pod kodem |
| `errorCorrection` | L, M, Q albo H |
| `margin` | szerokość quiet zone |

Pusty payload lub błąd generatora jest pokazany jawnie zamiast pustego obrazu.

## Bezpieczeństwo

QR Code nie wykonuje zawartości payloadu i nie pobiera zdalnych zasobów. Operator powinien traktować zeskanowany adres jako dane niezaufane.

