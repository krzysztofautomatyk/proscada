# Definition of Done kontrolki

Nowy typ jest ukończony, gdy:

1. ma unikalne ID i typ Registry;
2. ma osobny plik Svelte w prawidłowej kategorii;
3. posiada konfigurację domyślną;
4. renderuje Design i Runtime;
5. obsługuje brak danych i quality;
6. nie wykonuje zapisu w Designerze;
7. ma label, focus i obsługę klawiatury, jeśli jest interaktywny;
8. serializuje się i odtwarza z projektu;
9. jest podłączony w `WidgetView`;
10. ma osobny dokument w `docs/toolbox/controls`;
11. przechodzi `validate:widgets` i `validate:docs`;
12. nie wprowadza błędów `svelte-check`.

Kontrolki procesowe dodatkowo muszą respektować neutralne HMI i nie udawać potwierdzenia PLC.

