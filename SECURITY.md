# Seguridad

Flux DTL esta disenado como un laboratorio de liquidacion diferida con una
arquitectura modular y controles explicitos de riesgo. El sistema representa
un protocolo profesional de batch settlement con lanes, epochs, vaults,
oraculo, ordenes, fees y creditos de liquidez.

## Modelo De Seguridad

El protocolo aplica varias capas de control:

- registro de activos habilitados con decimales y parametros de riesgo;
- vaults vinculados a un unico activo y modo operativo;
- lanes configuradas con activos, vaults, operador, fees y limites;
- precios de oraculo con umbral minimo de confianza;
- ordenes asociadas a epoch, lane, owner, recipient, importe y salida minima;
- bloqueo de liquidez de entrada antes de encolar una orden;
- autorizacion de riesgo antes de ejecutar pagos;
- limites de notional por epoch y lane;
- limites de residual y utilizacion del vault destino;
- eventos de ejecucion para trazabilidad operativa.

## Invariantes Esperadas

Durante una operacion normal se espera que:

- una lane solo opere con vaults del activo configurado;
- un asset deshabilitado no pueda usarse para nuevas rutas;
- una orden pendiente solo pueda liquidarse una vez;
- el output del recipient respete `min_out`;
- el vault origen bloquee fondos antes de aceptar la orden;
- el vault destino tenga liquidez suficiente antes del pago;
- los fees se acrediten al operador de la lane;
- los rebates y creditos operativos se contabilicen de forma determinista;
- el epoch refleje las operaciones incluidas en el batch;
- el motor de riesgo autorice la operacion antes de mover balances.

## Validacion Automatizada

La validacion de CI ejecuta:

```bash
cargo fmt --all -- --check
cargo build --all-targets --locked
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
node --test "tests/node/*.test.js"
```

Los mismos comandos estan centralizados en `scripts/ci.sh`.

## Gestion De Dependencias

Las dependencias Rust quedan fijadas en `Cargo.lock`. El proyecto no requiere
dependencias JavaScript para ejecutar la suite Node actual. Dependabot revisa
Cargo, npm y GitHub Actions semanalmente.

## Reporte De Incidencias

Para revisiones internas, documenta cualquier hallazgo con:

- descripcion tecnica reproducible;
- impacto economico u operativo;
- archivos y funciones afectadas;
- precondiciones necesarias;
- pasos de reproduccion;
- resultado esperado frente a resultado observado;
- propuesta de mitigacion;
- comandos de verificacion.

## Alcance

El alcance de revision incluye el binario Rust, la suite Rust, la suite Node,
los scripts de CI y GitHub Actions. Quedan fuera despliegues reales, claves
privadas, infraestructura externa, redes de produccion y dependencias de
servicios no presentes en este repositorio.
