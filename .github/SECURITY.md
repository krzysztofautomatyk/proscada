# Security policy

## Reporting

Report suspected vulnerabilities through a private GitHub Security Advisory. Do not open a public issue containing exploit details, credentials or OT topology.

## Supported scope

The current main branch receives security fixes. ProScada is a lab and integration product, not a certified safety system.

## Sensitive areas

- Modbus write authorization and read-back;
- Runtime/role/quality gates;
- Tauri commands and capabilities;
- project scripts using `new Function`;
- project and `.pscctrl` import;
- alarm acknowledgement and audit integrity.

## Research rules

Do not test against a production PLC, scan an OT network or use real credentials. Use local fixtures, browser mock or an isolated simulator.

## Disclosure contents

Include affected version, path, impact, minimal reproduction using synthetic data and recommended mitigation. Remove secrets and customer information.

