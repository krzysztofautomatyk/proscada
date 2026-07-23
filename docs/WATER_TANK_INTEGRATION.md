# Water Tank — ProScada Integration

**Source of truth (PLC):**  
`PLC LAD SIMULATOR MODBUS TCP/docs/WATER_TANK_MODBUS_MAP.md`

**Slave:** `127.0.0.1:5020`  
**Poll:** FC03 start=`100` quantity=`22` → HR100…HR121

## Tag table (ProScada)

| Tag name | Source | Decode | R/W | Unit |
|----------|--------|--------|-----|------|
| `WT.DI_PACK` | HR100 | u16 | R | — |
| `WT.DO_PACK` | HR101 | u16 | R | — |
| `WT.M_LO` | HR102 | u16 | R | — |
| `WT.M_HI` | HR103 | u16 | R | — |
| `WT.LEVEL_cm` | HR104 | u16 | R/W* | cm |
| `WT.K_x100` | HR105 | u16 | R/W | ×100 |
| `WT.FILL_STEP` | HR106 | u16 | R/W | /tick |
| `WT.PUMP_STEP` | HR107 | u16 | R | /tick |
| `WT.SP_STOP` | HR108 | u16 | R/W | cm |
| `WT.SP_P1_ON` | HR109 | u16 | R/W | cm |
| `WT.SP_P2_ON` | HR110 | u16 | R/W | cm |
| `WT.CAP` | HR111 | u16 | R | — |
| `WT.P1_HH`…`WT.P2_STARTS` | HR114–121 | u16 | R | — |
| `WT.SIM_EN` | HR100 bit0 | bool | R | — |
| `WT.P1_FAULT` | HR100 bit3 | bool | R | — |
| `WT.P2_FAULT` | HR100 bit4 | bool | R | — |
| `WT.P1_LOCK` | HR100 bit5 | bool | R | — |
| `WT.P2_LOCK` | HR100 bit6 | bool | R | — |
| `WT.MAN_P1` | HR100 bit8 | bool | R | — |
| `WT.MAN_P2` | HR100 bit9 | bool | R | — |
| `WT.P1_RUN` | HR101 bit0 | bool | R | — |
| `WT.P2_RUN` | HR101 bit1 | bool | R | — |
| `WT.ALM_HI` | HR101 bit2 | bool | R | — |
| `WT.ALM_FAULT` | HR101 bit3 | bool | R | — |
| `WT.ALM_FAIL` | HR101 bit4 | bool | R | — |
| `WT.DEMAND` | HR102 bit2 | bool | R | — |
| `WT.JOIN_P2` | HR102 bit3 | bool | R | — |
| `WT.P1_OK` | HR102 bit4 | bool | R | — |
| `WT.P2_OK` | HR102 bit5 | bool | R | — |
| `WT.DRAIN` | HR103 bit9 | bool | R | — |

\* Level is process-owned when SIM_EN + RUN on PLC.

## Operator HMI (Runtime)

Light **greyscale dashboard** with **green / yellow / red** accents only (`WaterTankHmi`):

| Control | Behaviour |
|---------|-----------|
| **Process Freeze / Resume** | `FILL_STEP` HR106 → 0 / restore (pauses sim dynamics) |
| **Apply setpoints** | FC06 HR108–110 · `0 ≤ STOP < P1 ≤ P2 ≤ 1000` |
| **Write K** | FC06 HR105 · presets 50 / 100 / 150 |
| **SIM_EN badge** | Read-only (I0) — toggle in PLC Watch |
| **Ack all** | Local ISA-18.2-lite acknowledge |

Synoptic SVG: cutaway tank, P1/P2, dashed SP lines (green STOP · yellow P1 · red P2).

## Lab procedure

1. Start **PLC Ladder Simulator Pro** → toolbar **Water tank** → **RUN** → Modbus slave ON + **Allow SCADA writes**.
2. Start **ProScada** → **Connect Modbus** → **▶ Run**.
3. Use Freeze, setpoints, K; watch KPI cards + synoptic.
4. Force P1 fault in PLC Watch → failover visible on HMI.
