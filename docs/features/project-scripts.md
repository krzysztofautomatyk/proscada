# Skrypty projektowe

Skrypt projektowy to **deterministyczny język akcji**, a nie JavaScript.

Powody są dwa: spakowana aplikacja działa pod CSP `script-src 'self'`, które blokuje
`eval` i `new Function`, a `AGENTS.md` zakazuje wykonywania dowolnego kodu z pliku
projektu. Parser i runtime są w `src/lib/services/scriptRuntime.ts`.

## Składnia

Jedna instrukcja na linię. Puste linie oraz linie zaczynające się od `#` lub `//`
są ignorowane.

```text
# bezwarunkowe akcje
writeTag "wt.sp_p1_on" 700
ackAlarm "alm.level_high"
navigate "form_main"
log "sekwencja zakończona"

# akcja warunkowa
if "wt.level_cm" >= 800 then writeTag "wt.sp_stop" 1
```

## Dozwolone akcje

| Akcja | Argumenty | Efekt |
| :--- | :--- | :--- |
| `writeTag` | id taga, liczba | zapis przez backendową bramkę `write_tag` |
| `ackAlarm` | id definicji alarmu | ACK w centralnym silniku alarmów |
| `navigate` | id ekranu | przełączenie ekranu w Runtime |
| `log` | tekst | wpis w panelu Output |

## Warunki

`if "<tag>" <operator> <liczba> then <akcja>`

Operatory: `==`, `!=`, `>`, `>=`, `<`, `<=`.

Warunek **nie jest spełniony**, gdy tag nie istnieje albo jego jakość jest inna niż
`Good`. Działanie na niepewnej wartości jest gorsze niż brak działania.

## Błędy

Nierozpoznana instrukcja, brakujący argument, nieliczbowa wartość lub niedomknięty
literał tekstowy to twardy błąd z numerem linii. Ten sam parser jest używany przez
walidację projektu, więc Designer nie oznaczy jako poprawny skryptu, który runtime
odrzuci.

## Migracja

Skrypty napisane wcześniej w JavaScript nie sparsują się i zgłoszą błąd. Trzeba je
zapisać ponownie w powyższej składni.

## Powiązane

- [Granice bezpieczeństwa](../architecture/security.md)
- [Dynamiki kontrolek](dynamics.md)
