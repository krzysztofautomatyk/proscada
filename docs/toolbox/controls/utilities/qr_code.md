# QR Code

| Pole | Wartość |
|---|---|
| ID | `UTIL-QRCODE` |
| Typ | `qr_code` |
| Plik | `catalog/utilities/QRCodeWidget.svelte` |

Generuje kod QR lokalnie przez bibliotekę `qrcode`.

## Najważniejsze pola

`source`, `text`, `caption`, `errorCorrection`, `margin`.

`source=static` używa tekstu. `source=tag` koduje bieżącą wartość taga.

## Walidacja

Pusty payload oraz błąd generatora są pokazane jako jawny komunikat. Error correction przyjmuje L, M, Q lub H.

## Bezpieczeństwo

Kontrolka nie wykonuje payloadu i nie wysyła go do usługi zewnętrznej.

