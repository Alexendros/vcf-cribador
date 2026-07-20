# vcf-cribador — Tareas y roadmap

**Versión:** 0.1.0
**Última actualización:** 2026-07-20

---

## Estado actual

| Dimensión              | Estado              |
| ---------------------- | ------------------- |
| CI en `main`           | ✅ Passing (local)  |
| CI en PRs dependabot   | ⚠️ Pendiente        |
| Branch protection      | ⬜ Pendiente        |
| Cobertura de tests     | 129 tests (119 unit + 10 integration) |
| MSRV                   | 1.80                |
| Licencia               | MIT OR Apache-2.0   |

## Roadmap

### v0.1.0 — MVP actual ✅

- [x] Parseo VCF 4.0/3.0 con nom
- [x] Normalización FN, N, TEL, ORG (N1-N7)
- [x] Clasificación en 16 categorías (N2)
- [x] Cribado C2-C6, eliminación E1-E3
- [x] Deduplicación con Union-Find (cierre transitivo)
- [x] Export a VCF 4.0, CSV, JSON
- [x] Auditoría TSV
- [x] CLI con clap derive + autocompletado
- [x] Detección de fuente (Proton, Google, Apple)
- [x] Compatibilidad vCard 3.0 → 4.0
- [x] CI/CD con GitHub Actions (self-hosted)
- [x] Dependabot para cargo y GitHub Actions

### v0.2.0 — Calidad y robustez (siguiente)

- [ ] Branch protection en `main`
- [ ] Resolver CI de PRs dependabot pendientes
- [ ] Cobertura >80% con cargo-tarpaulin + Coveralls
- [ ] Tests de propiedad: generación de VCF aleatorios y verificación de invariantes
- [ ] Benchmark de rendimiento (criterion)
- [ ] Documentación de API en docs.rs
- [ ] Pre-commit hook con clippy + fmt

### v0.3.0 — Features

- [ ] Soporte ADR (direcciones postales)
- [ ] Soporte de categorías personalizadas (X-)
- [ ] Modo interactivo para revisar dudosos (NeedsReview)
- [ ] Integración con libphonenumber para normalización TEL internacional
- [ ] Export a LDIF/CSV de Outlook
- [ ] Pipeline de fusión por lotes (batch merge)

### v1.0.0 — Producción

- [ ] Auditoría de seguridad externa
- [ ] CI multiplataforma (Linux, macOS, Windows) — cuando runners self-hosted lo soporten
- [ ] Publicación en crates.io estable
- [ ] Página de documentación dedicada
- [ ] Test de regresión con corpus grande (>10k contactos)

## Tareas inmediatas

1. **Proteger rama `main`** — branch protection rules vía GitHub API
2. **Resolver PRs dependabot** — rebasar #8 y #12 sobre `main`, verificar CI, mergear
3. **Actualizar dependabot.yml** — ignorar dtolnay/rust-toolchain >=1.81 para preservar MSRV
4. **Corregir warnings de doc** — HTML tag sin escapar en parser.rs
5. **Documentar en README** — badges de CI, tareas, enlaces
6. **Configurar GitHub branch protection** — requiere CI passing, PR approval, linear history

## Notas técnicas

- Los runners son **self-hosted** etiquetados `[self-hosted, ts]`. Asegurar que tengan:
  - Rust toolchain estable + 1.80
  - cargo-tarpaulin (para coverage)
  - cargo-audit (para auditoría de seguridad semanal)
- El CI falla en PRs dependabot pero pasa en local → probablemente caché o toolchain del runner
- dtolnay/rust-toolchain usa versionado semántico: `@1.80` instala Rust 1.80, `@stable` instala el último estable
