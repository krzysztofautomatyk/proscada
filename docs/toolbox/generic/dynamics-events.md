# Dynamika i zdarzenia

## Wspólne warunki

`DynamicShell` obsługuje none, always, tag true/false, bit rejestru oraz porównania wartości.

Warunki sterują:

- widocznością;
- blinkiem;
- marquee tekstu;
- AnimationPreset.

## Zdarzenia

Kontrolki emitują typowane zdarzenia:

- `proscada:navigate`;
- `proscada:alarm-action`;
- `proscada:dialog-action`.

`App.svelte` rejestruje listenery na poziomie komponentu głównego.

## Ograniczenia

- brak arbitralnego kodu w nowych kontrolkach;
- blink maksymalnie 2 Hz;
- `prefers-reduced-motion` wyłącza animacje;
- zdarzenie UI nie jest dowodem wykonania procesu.

