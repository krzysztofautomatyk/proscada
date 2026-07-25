# Dialog

| Pole | Wartość |
|---|---|
| ID | `OVL-DIALOG` |
| Typ | `dialog` |
| Plik | `catalog/feedback/DialogWidget.svelte` |

Prezentuje decyzję, potwierdzenie lub podgląd formularza.

## Warianty

`modal`, `nonModal`, `confirmation`, `form`.

## Najważniejsze pola

`variant`, `title`, `message`, `confirmLabel`, `cancelLabel`, `modal`.

## Zdarzenia

Runtime emituje `proscada:dialog-action`. App loguje wynik. Dialog nie zapisuje bezpośrednio do PLC.

## Dostępność

Przyciski mają jawne etykiety. Produkcyjny modal powinien dodatkowo egzekwować focus trap na poziomie hosta.

