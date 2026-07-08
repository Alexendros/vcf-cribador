# Contribuir a vcf-cribador

¡Gracias por tu interés en contribuir!

## Flujo de trabajo

1. **Fork** del repositorio
2. Crea una rama: `git checkout -b feat/mi-feature`
3. Haz tus cambios siguiendo las convenciones del proyecto
4. Asegúrate de que pasan los checks: `make ci`
5. Commit con mensaje descriptivo
6. Push y abre un Pull Request

## Convenciones de código

- **Rust 2021 edition**, MSRV 1.80
- `cargo fmt` obligatorio (hook pre-commit incluido)
- `cargo clippy -- -D warnings` sin errores
- Tests unitarios para cada módulo de dominio y aplicación
- Tests de integración con fixtures reales en `tests/`

## Estructura del proyecto

```
src/domain/         # Entidades, value objects, reglas de negocio puras
src/application/    # Casos de uso (cribar, audit, stats, export)
src/infrastructure/ # Adaptadores (parser, writer, encoding, CSV/JSON)
src/interfaces/     # CLI (clap)
tests/              # Tests de integración con fixtures VCF
```

Ver [`docs/architecture.md`](docs/architecture.md) para la arquitectura completa.

## Antes de enviar un PR

```bash
make ci          # fmt + clippy + test + check + doc
make hooks       # instalar pre-commit hook
```

## Reportar bugs

Usa la plantilla de [bug report](.github/ISSUE_TEMPLATE/bug_report.md). Incluye:
- Comando exacto ejecutado
- Archivo VCF de ejemplo (anonimizado si contiene datos reales)
- Salida esperada vs obtenida
- Versión de vcf-cribador (`vcf-cribador --help`)
