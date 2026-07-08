# 📇 vcf-cribador — Ficha de producto v0.1.0

---

## Portada / Metadata

| Campo | Valor |
|-------|-------|
| **Nombre** | vcf-cribador |
| **Versión** | 0.1.0 |
| **Estado** | Publicado (crates.io + GitHub) |
| **Licencia** | MIT OR Apache-2.0 (dual) |
| **MSRV** | Rust 1.80+ |
| **Lenguaje** | Rust (edition 2021) |
| **Tipo** | CLI tool |
| **Repositorio** | https://github.com/Alexendros/vcf-cribador |
| **crates.io** | https://crates.io/crates/vcf-cribador |
| **Documentación** | https://docs.rs/vcf-cribador |
| **Binario release** | 3.2 MB (Linux x86-64, stripped, LTO) |
| **Líneas de código** | ~4,500 (src + tests) |
| **Tests** | 129 (unitarios + integración) |

## Descripción

Herramienta CLI para cribar, normalizar, clasificar y deduplicar archivos de contactos **VCF vCard 4.0/3.0** exportados desde **ProtonMail**, **Google Contacts** y **Apple iCloud**.

Aplica un pipeline determinista de 6 etapas con reglas configurables vía TOML, generando salida en VCF 4.0 limpio + auditoría completa en TSV, CSV y JSON.

## Funcionalidades

### Pipeline completo (`cribar`)

```
VCF → Parse → Normalize → Classify → Screen → Dedup → Write
```

| Etapa | Qué hace |
|-------|----------|
| **Parse** | RFC 6350 §3.2 unfold, §3.4 escape, propiedades agrupadas, v3→v4 |
| **Normalize** | N1-N7 FN (títulos, cargos, partículas), T1-T4 TEL (E.164 +34), ORG (siglas, formas jurídicas) |
| **Classify** | 16 categorías N2 en jerarquía de 3 niveles (PROF-JUD, INST-AUT, FIN-CRYPTO, TECH-SW, SALUD-SOC...) |
| **Screen** | C2-C6 conservación por categoría + E1-E3 eliminación (huérfanos, spam, email-only) |
| **Dedup** | Union-Find con cierre transitivo D1-D2 (UID exacto, TEL exacto, EMAIL fuzzy, FN fuzzy) |
| **Write** | VCF 4.0 folding 75 octetos + TSV auditoría (11 columnas) + CSV export + JSON export |

### Comandos adicionales

| Comando | Función |
|---------|---------|
| `audit` | Solo screening + TSV, sin modificar VCF |
| `stats` | Estadísticas en texto, JSON o Markdown |
| `export` | Export CSV o JSON desde pipeline |
| `completions` | Genera autocompletado para bash, zsh, fish |

## Configuración

Archivo TOML opcional con soporte `replace` (reemplazar defaults) / `append` (añadir):

```toml
[cribado]
replace = false
prefijo_pais = "+34"
conservar_dominios = ["@example.org"]
e2_keywords = ["pharma", "jackpot"]
```

## Integraciones y exportación

| Formato | Dirección | Detalle |
|---------|-----------|---------|
| VCF 4.0 | Salida | FN, N, ORG, TEL, EMAIL, NOTE, X-CRIBADO-*, PRODID |
| VCF 3.0/4.0 | Entrada | Auto-detección Proton/Google/Apple |
| TSV | Auditoría | 11 columnas (UID, FN, TEL, EMAIL, ORG, SOURCE, acción, regla, evidencia, merged_uids, categorías) |
| CSV | Export | 8 columnas (FN, N, ORG, TEL, EMAIL, CATEGORIES, SOURCE, CRIBADO_RESULT) |
| JSON | Export | Array completo con todos los campos de Contact |
| TOML | Config | Reglas personalizadas de cribado |
| ISO-8859-1 | Entrada | Transcodificación automática → UTF-8 |

## Tecnologías

| Categoría | Dependencia | Versión |
|-----------|-------------|---------|
| CLI | clap (derive) | 4.5 |
| Parser | nom | 7 |
| Regex | regex | 1.10 |
| CSV | csv | 1.3 |
| Logging | tracing + tracing-subscriber | 0.1 / 0.3 |
| Fechas | jiff | 0.2 |
| Errores | thiserror + anyhow | 2 / 1 |
| Fuzzy | strsim | 0.11 |
| Encoding | chardetng + encoding_rs | 0.1 / 0.8 |
| Config | toml | 0.8 |
| JSON | serde + serde_json | 1 |
| Templating | clap_complete | 4.5 |
| Dev | tempfile | 3 |

## Arquitectura

```
src/
├── domain/           Reglas de negocio puras
│   ├── contact.rs       Entidad Contact, value objects
│   ├── screening.rs     Motor C2-E3, DecisionTrace
│   ├── classification.rs  16 categorías N2
│   ├── normalization.rs    FN/TEL/ORG E.164
│   ├── identity.rs         Union-Find dedup
│   └── rules.rs            Regex clasificación
├── application/     Casos de uso
│   ├── cribar.rs       Pipeline completo
│   ├── audit.rs        Auditoría standalone
│   └── stats.rs        Estadísticas 3 formatos
├── infrastructure/  Adaptadores
│   ├── parser.rs       VCF parser RFC 6350
│   ├── writer.rs       VCF writer + folding
│   ├── tsv_writer.rs   Auditoría 11 columnas
│   ├── csv_writer.rs   Export CSV
│   ├── json_writer.rs  Export JSON
│   ├── encoding.rs     ISO-8859-1 → UTF-8
│   ├── source.rs       Detección Proton/Google/Apple
│   ├── v3_compat.rs    vCard 3.0 → 4.0
│   └── config.rs       TOML con replace/append
└── interfaces/      CLI clap derive
```

**Patrón:** Clean Architecture con dependencias hacia dominio.

## Calidad y CI/CD

| Aspecto | Herramienta |
|---------|-------------|
| CI | GitHub Actions (check stable + MSRV, fmt, clippy, test, doc) |
| Release | GitHub Actions automático (tag v* → build + SHA256 + GitHub Release + crates.io) |
| Security | `cargo audit` semanal (lunes 09:00 Europe/Madrid) |
| Dependabot | Cargo + Actions, semanal, ignora breaking changes |
| Pre-commit | `cargo fmt --check` + `cargo clippy -D warnings` |
| Linting | Clippy estricto sin warnings |
| Tests | 129 tests (119 lib + 10 integración) con fixtures sintéticos |

## Roadmap

| Versión | Features |
|---------|----------|
| **v0.1.0** ✅ | Pipeline completo, 4 comandos CLI, configuración TOML, stats/export, CI/CD, crates.io |
| **v0.2.0** | Reglas C1/C5/C7/E4/E6, campo ADR, validación invariantes I1-I7, Apple fixtures |
| **v0.3.0** | CardDAV sync, watch mode, filtros por categoría |
| **v1.0.0** | API estable, benchmarks, cross-compile macOS/Windows |

## Seguridad y privacidad

- Procesamiento local, sin conexiones externas ni telemetría
- Fixtures de tests 100% sintéticos (datos ficticios)
- Código abierto auditable bajo MIT OR Apache-2.0
- No almacena ni transmite datos de contacto
- Política de vulnerabilidades en SECURITY.md

## Uso rápido

```bash
cargo install vcf-cribador
vcf-cribador cribar contactos.vcf -o limpio.vcf -a audit.tsv
```

## Enlaces

- 🏠 GitHub: https://github.com/Alexendros/vcf-cribador
- 📦 crates.io: https://crates.io/crates/vcf-cribador
- 📚 Documentación: https://docs.rs/vcf-cribador
