# Navigation Link

| Pole | Wartość |
|---|---|
| ID | `NAV-LINK` |
| Typ | `navigation_link` |
| Plik | `catalog/navigation/NavigationLinkWidget.svelte` |

Przechodzi do bezpiecznej trasy aplikacji.

## Najważniejsze pola

`label`, `target`, `params`.

Dozwolone targety zaczynają się od `/` albo `screen:`. `params` jest obiektem JSON.

## Runtime

Kontrolka emituje `proscada:navigate` z targetem, parametrami i ID źródła. App wybiera ekran po ID lub nazwie.

## Bezpieczeństwo

Niepoprawny lub zewnętrzny target jest blokowany. Designer nie wykonuje nawigacji.

