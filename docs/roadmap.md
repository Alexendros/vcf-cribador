# Roadmap vcf-cribador

Hoja de ruta del proyecto, ordenada por fases de implementación y prioridad.

## Fase 0 — Cimientos ✅
- Estructura de proyecto Rust con `Cargo.toml`, licencias y CI.
- Documentación inicial (`docs/`).
- Módulos `domain/` con tipos base.

## Fase 1 — Ingesta ✅
- Parser vCard 4.0/3.0 con `nom`.
- Unfold, unescape y soporte de propiedades agrupadas.
- Detección de fuente (Proton, Google, Apple).
- Transcodificación ISO-8859-1 → UTF-8.

## Fase 2 — Calidad ✅
- Conversión `ParsedVCard` → `Contact`.
- Screening C1-C7 y E1-E3.
- Normalización de nombres, títulos, cargos y teléfonos.

## Fase 3 — Clasificación y salida ✅
- Clasificación automática con taxonomía N1/N2.
- Writer VCF 4.0 con folding 75 octetos.
- Auditoría TSV, export CSV/JSON y estadísticas.

## Fase 4 — Taxonomía N3, tipos T4 y ADR ✅
- Taxonomía de tres niveles (N1/N2/N3).
- Tipos de teléfono T4 completos.
- Soporte de direcciones postales ADR.
- Regla E3 considera direcciones.

## Fase 5 — Pipeline completo y configuración (actual)
- Conectar todas las etapas en `application/cribar.rs`.
- Configuración externa TOML con herencia/append y replace.
- Validación de invariantes con `domain::verification`.

## Fase 6 — Testing y calidad
- Ampliar tests de integración con fixtures reales.
- Alcanzar ≥80 % de cobertura unitaria.
- Integrar `cargo audit` en CI.

## Fase 7 — Release v0.1.0
- Tag `v0.1.0`.
- Binario release < 8 MB.
- Publicación en crates.io (opcional).

## Futuro (post-v0.1.0)
- v0.2.0: soporte vCard 2.1, mejoras en normalización de direcciones.
- v0.3.0: deduplicación avanzada y fusión semiautomática.
- v0.4.0: TUI/GUI interactiva.
- v0.5.0+: integraciones CardDAV/Proton/Google.
