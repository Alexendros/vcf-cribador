# vcf-cribador

[![CI](https://github.com/Alexendros/vcf-cribador/actions/workflows/ci.yml/badge.svg)](https://github.com/Alexendros/vcf-cribador/actions/workflows/ci.yml)
[![Coverage](https://coveralls.io/repos/github/Alexendros/vcf-cribador/badge.svg)](https://coveralls.io/github/Alexendros/vcf-cribador)
[![Security Audit](https://github.com/Alexendros/vcf-cribador/actions/workflows/audit.yml/badge.svg)](https://github.com/Alexendros/vcf-cribador/actions/workflows/audit.yml)
[![Crates.io](https://img.shields.io/crates/v/vcf-cribador?color=orange)](https://crates.io/crates/vcf-cribador)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.80+-blue.svg)](https://blog.rust-lang.org/2024/07/25/Rust-1.80.0.html)

Criba, normaliza, clasifica y deduplica contactos VCF exportados desde ProtonMail, Google Contacts o Apple iCloud.

Limpia tus contactos VCF exportados desde **ProtonMail**, **Google Contacts** o **Apple iCloud** aplicando reglas deterministas de clasificación (C2-C6) y eliminación (E1-E3), deduplicación con cierre transitivo, y normalización de nombres y teléfonos.

## Quick start

```bash
# Instalar
cargo install vcf-cribador

# Cribar un archivo (conservados → limpio.vcf, trazabilidad → auditoría.tsv)
vcf-cribador cribar mis-contactos.vcf -o limpio.vcf -a auditoria.tsv

# Solo auditar sin modificar
vcf-cribador audit mis-contactos.vcf -o auditoria.tsv

# Estadísticas
vcf-cribador stats limpio.vcf
vcf-cribador stats limpio.vcf -f json
vcf-cribador stats limpio.vcf -f markdown

# Exportar a CSV o JSON
vcf-cribador export limpio.vcf -o contactos.csv
vcf-cribador export limpio.vcf -o contactos.json -f json
```

## Configuración

Opcional: crea un archivo TOML para personalizar el cribado.

```toml
# cribador.toml
[cribado]
prefijo_pais = "+34"         # prefijo telefónico por defecto
replace = false              # false = añade a los defaults, true = reemplaza
conservar_dominios = [       # dominios de email que NUNCA se eliminan
    "@example.org",
    "@admin.gob.es"
]
e2_keywords = [              # palabras clave adicionales para detección de spam
    "pharma",
    "jackpot"
]

[clasificacion]
replace = false              # false = añade a las reglas por defecto, true = reemplaza
[[clasificacion.reglas]]
regex = "(?i)rob[óo]tica|maker"
n1 = "TEC"
n2 = "TEC-HW"
n3 = "HW-MAKER"
```

```bash
vcf-cribador cribar contactos.vcf --config cribador.toml
```

## Pipeline

```
  VCF  ──→  Parse   ──→  Normalize  ──→  Classify  ──→  Screen  ──→  Dedup  ──→  Write
 4.0/3.0    unfold      FN · TEL · ORG    16 categorías     C2-C6          Union-Find     VCF
            unescape     E.164 · N7        N1 + N2          E1-E3          cierre         TSV
            grouped                                                     transitivo      CSV/JSON
```

| Etapa         | Descripción                                                                                           |
| ------------- | ----------------------------------------------------------------------------------------------------- |
| **Parse**     | RFC 6350 §3.2 (unfold), §3.4 (escape). Propiedades agrupadas (`ITEM1.EMAIL`). Compatibilidad v3 → v4. |
| **Normalize** | N1-N7: capitalización de nombres, extracción de títulos, cargos, partículas. T1-T4: E.164 + tipos T4.  |
| **Classify**  | Taxonomía N1/N2/N3: PROF, INST, FIN, FORM, TEC, HOST, TRAN, INMO, SERV, ASOC, SALUD, etc.                |
| **Screen**    | C2-C6: conservar por categoría. E1-E3: eliminar huérfanos, spam, email-only.                          |
| **Dedup**     | Union-Find con cierre transitivo. Coincidencia por TEL exacto, EMAIL fuzzy, FN fuzzy.                 |
| **Write**     | VCF 4.0 con folding 75 octetos. ADR y tipos T4 preservados. TSV de trazabilidad. CSV/JSON export.    |

## Ejemplo real

```
$ vcf-cribador cribar contacts-2025.vcf -o limpio.vcf -a audit.tsv

=== Estadísticas de cribado ===
Total entrada:   475
  Conservados:   221
  Eliminados:    254
  Fusionados:    0
  Cuarentena:    0
  Needs Review:  1

Por categoría:
  FIN-CRYPTO:  3    FIN-FINTEC:  5    INST-AUT:  7
  PROF-JUD:    3    PROF-NOT:    2    PROF-COL:  4
  TEC-COM:     3    SALUD-SOC:   2    ...
```

## Arquitectura

```
src/
├── domain/           Reglas de negocio puras
│   ├── contact.rs    Entidad Contact, StructuredName, CategorySet, Address
│   ├── screening.rs  Motor de cribado C2-E3, DecisionTrace
│   ├── classification.rs  Clasificación N1/N2/N3 por regex
│   ├── normalization.rs   FN/TEL/ORG/ADR normalization
│   ├── audit.rs           Modelo de trazas de auditoría
│   └── verification.rs    Verificación de invariantes de dominio
│   ├── identity.rs        Dedup Union-Find
│   └── rules.rs           Reglas de clasificación
├── application/     Casos de uso
│   ├── cribar.rs    Pipeline completo
│   ├── audit.rs     Auditoría standalone
│   └── stats.rs     Estadísticas (texto/JSON/Markdown)
├── infrastructure/  Adaptadores
│   ├── parser.rs    VCF parser (nom)
│   ├── writer.rs    VCF writer RFC 6350
│   ├── tsv_writer.rs   Auditoría TSV
│   ├── csv_writer.rs   Export CSV
│   ├── json_writer.rs  Export JSON
│   ├── encoding.rs  ISO-8859-1 → UTF-8
│   ├── source.rs    Detección Proton/Google/Apple
│   ├── v3_compat.rs vCard 3.0 → 4.0
│   └── config.rs    Configuración TOML
└── interfaces/      CLI (clap derive)
```

→ [`docs/architecture.md`](docs/architecture.md)

## Documentación

| Documento                                                      | Contenido                                            |
| -------------------------------------------------------------- | ---------------------------------------------------- |
| [`docs/spec.md`](docs/spec.md)                                 | Especificación, invariantes, criterios de aceptación |
| [`docs/domain.md`](docs/domain.md)                             | Lenguaje ubicuo, entidades, rules                    |
| [`docs/architecture.md`](docs/architecture.md)                 | Clean Architecture, capas                            |
| [`docs/implementation-guide.md`](docs/implementation-guide.md) | Guía de implementación por fases                     |
| [`docs/test-plan.md`](docs/test-plan.md)                       | Estrategia de testing, fixtures                      |
| [`docs/events.md`](docs/events.md)                             | Comandos, eventos                                    |
| [`docs/adr/`](docs/adr/)                                       | Architecture Decision Records                        |

## Desarrollo

```bash
git clone https://github.com/Alexendros/vcf-cribador.git
cd vcf-cribador

make hooks     # instalar pre-commit hooks
make ci        # fmt + clippy + test + doc
make release   # build release
```

Ver [`CONTRIBUTING.md`](CONTRIBUTING.md) para la guía de contribución.

## Seguridad

Reporta vulnerabilidades de forma privada. Ver [`SECURITY.md`](SECURITY.md).

Ejecutamos `cargo audit` semanalmente vía GitHub Actions.

## Licencia

MIT OR Apache-2.0 · Ver [`LICENSE`](LICENSE)
