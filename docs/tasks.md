# Tareas del proyecto vcf-cribador

Este documento enumera las tareas completadas, en curso y pendientes del proyecto. Se actualiza junto con cada fase de desarrollo.

## Estado actual

- Fase 0 — Estructura y scaffold ✅
- Fase 1 — Parser vCard 4.0/3.0 ✅
- Fase 2 — Screening y normalización ✅
- Fase 3 — Clasificación + Writer VCF ✅
- Fase 4 — Taxonomía N3, tipos T4 y ADR ✅
- Fases 5-7 — Pipeline completo, configuración TOML y release 🔄

## Fase 4 completada

- [x] Ampliar `CategorySet` con niveles N1, N2 y N3.
- [x] Añadir campo `n3` a `ClassificationRule` y poblarlo en `classify()`.
- [x] Expandir `CLASSIFICATION_RULES` con profesiones, instituciones, finanzas, educación, tecnología, comercio y servicios.
- [x] Expandir `TelType` a `cell`, `home`, `work`, `main`, `fax`, `pager`, `text`, `video`, `other`.
- [x] Mapear tipos de vCard en `map_tel_type()` y preservar el tipo en la normalización.
- [x] Emitir tipos de teléfono correctos en el writer VCF.
- [x] Añadir value object `Address` y campo `addresses: Vec<Address>` a `Contact`.
- [x] Parsear propiedades `ADR` en `parser.rs`.
- [x] Escribir direcciones en VCF, CSV y JSON.
- [x] Actualizar regla E3 (huérfanos) para considerar direcciones postales.
- [x] Ajustar todos los literales `Contact` en tests y producción.
- [x] Pasar `cargo test`, `cargo clippy -- -D warnings` y `cargo fmt --check`.

## En curso / pendientes inmediatos

- [ ] Actualizar `docs/architecture.md` y `README.md` con ADR, T4 y taxonomía N3.
- [ ] Completar pipeline en `application/cribar.rs` si quedan stubs por resolver.
- [ ] Implementar configuración TOML completa (`infrastructure/config.rs`).
- [ ] Añadir tests de integración con fixtures reales.
- [ ] Release v0.1.0.

## Backlog técnico

- [ ] Mejorar normalización de direcciones (país por defecto, código postal español).
- [ ] Soporte para vCard 2.1 (descartado para v1.0, reconsiderar en v0.2.0).
- [ ] GUI/TUI interactiva (v0.4.0).
- [ ] Conexión con APIs CardDAV/Proton/Google (v0.5.0+).
