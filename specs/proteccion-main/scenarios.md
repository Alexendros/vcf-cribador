# Escenarios — Protección de main

## Happy paths

| ID  | Escenario                                                      | Resultado esperado                     |
|-----|----------------------------------------------------------------|----------------------------------------|
| H1  | `make ci` en local pasa sin errores                            | Exit 0, 0 lint errors, 0 test failures |
| H2  | PR #12 rebasado en main → CI verde                             | Todos los checks ✅                    |
| H3  | PR #8 rebasado en main → CI verde                              | Todos los checks ✅                    |
| H4  | Branch protection aplicada a main                              | Push directo denegado, PR requerido    |
| H5  | `dependabot.yml` ignora dtolnay/rust-toolchain >=1.81          | No más PRs que suban MSRV             |

## Edge cases

| ID  | Escenario                                                      | Resultado esperado                     |
|-----|----------------------------------------------------------------|----------------------------------------|
| E1  | PR #12 tiene conflictos de merge tras rebase                   | Resolver conflictos manualmente        |
| E2  | PR #8 cambia dtolnay/rust-toolchain@1.80 → @1.99              | Rechazar el cambio, mantener 1.80      |
| E3  | Branch protection bloquea merge sin CI verde                   | Merge imposible hasta CI pase          |
| E4  | Dependabot genera nuevo PR con dtolnay/rust-toolchain >=1.81   | CI falla en MSRV job, revisar ignore   |

## Errores esperados

| ID  | Error                                                          | Manejo                                 |
|-----|----------------------------------------------------------------|----------------------------------------|
| R1  | GitHub API token sin permisos para branch protection           | Mostrar error, pedir token con repo:admin |
| R2  | Self-hosted runner sin herramienta (cargo-tarpaulin, etc.)     | Instalar herramienta en runner          |
| R3  | Doc warning por HTML tag no escapado                           | Añadir backticks en doc comment         |
