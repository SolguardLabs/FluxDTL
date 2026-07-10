# Flux DTL

![banner](./assets/banner.png)

Flux DTL es un binario Rust que modela una capa de transferencia y liquidacion
diferida para flujos multi-activo. El protocolo organiza ordenes por epochs,
lanes de liquidacion, vaults de funding y settlement, precios de oraculo,
controles de riesgo y creditos operativos para participantes de liquidez.

El proyecto esta orientado a auditorias tecnicas de sistemas financieros on
chain y off chain con liquidacion por lotes. La implementacion es
autocontenida, determinista y no requiere servicios externos para ejecutar el
escenario de referencia o la suite de pruebas.

## Componentes

- `accounts`: cuentas y balances por activo.
- `amount`: aritmetica entera, ratios y basis points.
- `asset`: registro de activos, decimales y pesos de riesgo.
- `oracle`: libro de precios determinista con confianza por activo.
- `vault`: reservas, bloqueos, pagos y balances operativos.
- `lanes`: rutas configurables entre activo origen y activo destino.
- `orders`: ordenes de transferencia asociadas a una lane y un epoch.
- `epochs`: acumuladores de batch para input, output, fees, rebates y residual.
- `settlement`: motor de cotizacion y calculo de comisiones.
- `risk`: limites de notional, residual y utilizacion.
- `ledger`: orquestacion del estado, eventos y ejecucion.

## Requisitos

- Rust estable.
- Cargo con soporte para `edition = "2021"`.
- Node.js `>= 20` para la suite JavaScript.
- Bash para ejecutar los scripts locales de CI.

## Uso

Ejecutar el escenario operativo:

```bash
cargo run -- demo
```

Ejecutar el escenario con salida JSON:

```bash
cargo run -- demo-json
```

Mostrar ayuda:

```bash
cargo run -- help
```

## Tests

Ejecutar los tests Rust:

```bash
cargo test --locked
```

Ejecutar los tests Node:

```bash
node --test "tests/node/*.test.js"
```

Ejecutar toda la suite:

```bash
npm run test:all
```

Ejecutar el flujo completo de CI:

```bash
bash scripts/ci.sh
```

## CI

El workflow de GitHub Actions valida:

- formato Rust con `cargo fmt`;
- build completo con `cargo build --all-targets`;
- tests Rust con `cargo test`;
- lint estricto con `cargo clippy -D warnings`;
- tests Node con `node:test`.

Dependabot revisa actualizaciones para Cargo, npm y GitHub Actions.

## Estructura

```text
src/
  accounts/
  amount/
  asset/
  epochs/
  events/
  ids/
  lanes/
  ledger/
  oracle/
  orders/
  risk/
  settlement/
  vault/
tests/
  helpers/
  node/
  rust_cli_demo.rs
scripts/
  ci.sh
  tests.sh
```

## Estado

Flux DTL es un laboratorio autocontenido. El binario no abre puertos, no
consume APIs externas y no requiere infraestructura adicional para reproducir
su comportamiento.
