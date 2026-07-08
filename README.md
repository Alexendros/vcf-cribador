# vcf-cribador

Criba, normaliza, clasifica y deduplica contactos VCF vCard 4.0/3.0 exportados desde ProtonMail, Google Contacts y Apple iCloud.

## Instalación

```bash
cargo install --path .
```

## Uso

```bash
# Pipeline completo
vcf-cribador cribar contacts.vcf -o limpio.vcf -a audit.tsv

# Solo auditar (sin modificar)
vcf-cribador audit contacts.vcf -o audit.tsv

# Estadísticas post-cribado
vcf-cribador stats limpio.vcf

# Exportar a CSV/JSON
vcf-cribador export limpio.vcf -o contacts.csv -f csv
```

## Arquitectura

```
domain/          # Reglas de negocio puras (Contact, ScreeningDecision, Dedup)
application/     # Casos de uso (Cribar, Audit, Stats, Export)
infrastructure/  # Adaptadores (parser RFC, writer VCF, encoding, CSV/JSON/TSV)
interfaces/      # CLI (Clap)
```

Ver [`docs/architecture.md`](docs/architecture.md) para el diseño completo.

## Documentación

| Documento | Contenido |
|-----------|-----------|
| [`docs/spec.md`](docs/spec.md) | Objetivos, invariantes, criterios de aceptación |
| [`docs/domain.md`](docs/domain.md) | Lenguaje ubicuo, entidades, value objects, reglas |
| [`docs/architecture.md`](docs/architecture.md) | Clean Architecture, bounded contexts, capas |
| [`docs/events.md`](docs/events.md) | Comandos, eventos, excepciones |
| [`docs/test-plan.md`](docs/test-plan.md) | Estrategia de testing, fixtures |
| [`docs/adr/README.md`](docs/adr/README.md) | Architecture Decision Records |

## Licencia

MIT OR Apache-2.0
