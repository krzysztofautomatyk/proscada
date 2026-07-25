# Mapa Water Tank

## Urządzenie

- host: `127.0.0.1`;
- port: `5020`;
- tabela: holding registers;
- zoptymalizowany blok: HR100–121.

## Główne rejestry

| HR | Tag | Znaczenie |
|---:|---|---|
| 100 | DI_PACK | wejścia binarne |
| 101 | DO_PACK | wyjścia binarne i alarmy |
| 102–103 | M_LO / M_HI | markery |
| 104 | LEVEL_cm | poziom |
| 105 | K_x100 | współczynnik dopływu |
| 106 | FILL_STEP | dopływ |
| 108–110 | SP_* | setpointy |
| 114–121 | run time / starts | liczniki pomp |

## Bity

Przykłady: P1_RUN = HR101.0, P2_RUN = HR101.1, ALM_HI = HR101.2.

## Uruchomienie

Włącz projekt Water Tank w PLC Ladder Simulator Pro, uruchom slave i zezwól na zapisy SCADA, jeśli testujesz setpointy.

