# Changelog

Todas las modificaciones notables de este proyecto se documentan en este archivo.

El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/),
y este proyecto adhiere a [Semantic Versioning](https://semver.org/lang/es/).

## [Unreleased]

## [0.1.0] - 2025-07-07

### Añadido
- Pipeline completo de cribado: parsing → normalización → clasificación → screening → dedup → escritura
- Comando `cribar` con soporte `--dry-run`
- Comando `audit` para auditoría standalone sin modificar VCF
- Comando `stats` con formatos texto, JSON y Markdown
- Comando `export` con formatos CSV y JSON
- Parser vCard 4.0/3.0 RFC 6350 con unfold, desescapado y propiedades agrupadas
- Compatibilidad automática vCard 3.0 → 4.0 (Google, Apple, Proton)
- Detección automática de fuente (ProtonAutosave, GoogleContacts, Apple)
- Transcodificación ISO-8859-1 → UTF-8 (chardetng + encoding_rs)
- Escritor VCF 4.0 con folding a 75 octetos respetando UTF-8 multibyte
- Escritor TSV de auditoría con 11 columnas (trazabilidad completa)
- Clasificación automática C2-C6 con 16 categorías N2
- Eliminación E1-E3 (huerfanos, spam, email-only)
- Deduplicación con Union-Find (cierre transitivo TEL + EMAIL + FN)
- Normalización E.164 para teléfonos españoles (+34)
- Normalización de nombres propios (capitalización, títulos, cargos)
- Configuración externa TOML con soporte replace/append
- CI/CD con GitHub Actions (check + fmt + clippy + test + doc)
- Release automatizado con binario + SHA256
- 132 tests unitarios y de integración

[Unreleased]: https://github.com/alexendros/vcf-cribador/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/alexendros/vcf-cribador/releases/tag/v0.1.0
