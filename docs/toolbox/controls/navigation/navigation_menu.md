# Navigation Menu

| Pole | Wartość |
|---|---|
| ID | `NAV-MENU` |
| Typ | `navigation_menu` |
| Plik | `catalog/navigation/NavigationMenuWidget.svelte` |

Tworzy hierarchiczne menu ekranów.

## Najważniejsze pola

`title`, `items`.

Każdy wiersz ma format:

```text
Pumping/Pump stations=screen:Pumps
```

Slash w etykiecie tworzy wizualne zagnieżdżenie.

## Zachowanie

Wybrana pozycja jest oznaczona lokalnie, a poprawny target emituje `proscada:navigate`.

## Bezpieczeństwo

Target jest walidowany tak samo jak Navigation Link. Niepoprawna pozycja jest oznaczona jako blocked.
