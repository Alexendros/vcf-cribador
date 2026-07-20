# Plan de tests — Protección de main

## Orden de ejecución

| Paso | Acción                                    | Verificación                          |
|------|-------------------------------------------|---------------------------------------|
| 1    | `cargo fmt --all -- --check`              | Exit 0                                |
| 2    | `cargo clippy --all-features -- -D warnings` | Exit 0                             |
| 3    | `cargo test --all-features`               | 129 tests pass                        |
| 4    | `cargo doc --no-deps --document-private-items` | Exit 0, 0 warnings              |
| 5    | `cargo check --all-features`              | Exit 0                                |
| 6    | Verificar dependabot.yml ignore rules     | dtolnay/rust-toolchain >=1.81 ignorado |
| 7    | Verificar branch protection (GH API)      | Reglas activas en main                |
| 8    | Verificar que README tiene badges CI      | Badges presentes y enlazando           |

## Cobertura

| Módulo                       | ¿Cubre? | Notas                                |
|------------------------------|---------|--------------------------------------|
| src/infrastructure/parser.rs | ✅      | Fix doc warning HTML tag              |
| .github/dependabot.yml       | ✅      | Ignore rule actualizada               |
| .github/workflows/*.yml      | ✅      | Sin cambios (CI ya está bien definido)|
| README.md                    | ✅      | Badges + tareas + roadmap             |
| docs/architecture.md         | ✅      | Contexto CI/deployment                |
| docs/tasks.md                | ✅      | Nueva hoja de ruta                    |
