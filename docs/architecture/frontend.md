# Frontend Svelte

## Warstwy

```text
App / shell
  ├─ Designer panels
  ├─ Widget Registry
  ├─ WidgetView dispatcher
  ├─ catalog/<toolbox-group>/*.svelte
  └─ shared helpers
```

## Katalog kontrolek

`src/lib/components/widgets/registry/catalog.ts` definiuje:

- identyfikator kanoniczny;
- typ serializowany;
- kategorię Toolboxa;
- rozmiar i konfigurację domyślną;
- capabilities;
- aliasy migracyjne.

Każdy z 35 typów ma osobny renderer. Skrypt `validate:widgets` sprawdza tę relację.

## Wspólne elementy

`shared/` zawiera typowane propsy, bezpieczne odczyty konfiguracji, QualityBadge, WidgetCard i EmptyState.

`DynamicShell` realizuje widoczność, blink, animacje projektu i tokeny stylu.

## Stan aplikacji

Store `app.ts` zarządza projektem, zaznaczeniem, zapisem, komponentami złożonymi i logami.

