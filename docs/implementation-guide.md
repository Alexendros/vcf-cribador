# vcf-cribador — Guía de implementación

**Versión:** 0.1.0
**Fecha:** 2026-07-07

---

## Cómo leer esta guía

Cada fase mapea una sección del plan canónico (`~/.commandcode/plans/protocolo-cribado-contactos-vcf.md`) a los módulos `src/` que deben implementarse. Las referencias a secciones del plan usan su numeración (§9.x, §2, §3, etc.).

**Documentos de referencia por rol:**

| Si necesitas... | Lee... |
|-----------------|--------|
| Objetivos, invariantes, criterios de aceptación | [`docs/spec.md`](spec.md) |
| Modelo de dominio, value objects, reglas | [`docs/domain.md`](domain.md) |
| Arquitectura, capas, bounded contexts | [`docs/architecture.md`](architecture.md) |
| Comandos, eventos, flujo del pipeline | [`docs/events.md`](events.md) |
| Estrategia de testing, fixtures | [`docs/test-plan.md`](test-plan.md) |
| Decisiones de diseño justificadas | [`docs/adr/README.md`](adr/README.md) |
| Plan completo (spec técnica Rust) | `~/.commandcode/plans/protocolo-cribado-contactos-vcf.md` §9 |
| Protocolo de cribado (reglas E/C, taxonomía, normalización) | `~/.commandcode/plans/protocolo-cribado-contactos-vcf.md` §1-§5 |

---

## Fase 0 — Ya completada ✅

- [x] Estructura de proyecto, `Cargo.toml`, `.gitignore`
- [x] Documentación completa (`docs/`)
- [x] Módulos `domain/` con tipos, reglas y lógica core
- [x] `domain::parser` con `unfold`, `unescape`, `is_binary` + 13 tests
- [x] `domain::classification` con `classify()` + 5 tests
- [x] `domain::identity` con `deduplicate()` Union-Find + 6 tests
- [x] CLI con Clap (4 subcomandos, stubs)
- [x] `cargo check` limpio, `cargo test` → 26/26 ✅

---

## Fase 1 — Parser vCard 4.0/3.0 → `ParsedVCard`

**Objetivo:** Leer un archivo VCF y producir `Vec<ParsedVCard>`.

**Referencia del plan:** §9.5 (parser.rs con unescape corregido), §9.5 (v3_compat.rs)

**Módulos a implementar:**

### 1.1 `infrastructure::parser` — nom parser

```
src/infrastructure/parser.rs   ← expandir desde stubs actuales
```

Tareas:
- [ ] `parse_vcards(input: &str) -> Result<Vec<ParsedVCard>, CribaError>`
- [ ] Combinators nom: `vcard`, `property`, `param`, `value`
- [ ] Gramática: `vcard = BEGIN:VCARD CRLF properties END:VCARD CRLF`
- [ ] Propiedades agrupadas: `ITEM1.EMAIL;PREF=1:...` → `group="ITEM1"`, `name="EMAIL"`
- [ ] Parámetros multi-valor: `TEL;TYPE=CELL,VOICE:+34...`
- [ ] `N` estructurado: `N:family;given;additional;prefix;suffix`
- [ ] Propiedades binarias → `raw_lines` (PHOTO, LOGO, SOUND, KEY, data: URIs)
- [ ] `unescape` aplicado solo a propiedades no raw
- [ ] Tests: `test_parse_single_vcard_4_0`, `test_parse_vcard_3_0`, `test_parse_grouped`, `test_parse_n_structured`, `test_parse_photo`, `test_parse_params_multi_value`, `test_vcf_referencia`

### 1.2 `infrastructure::v3_compat` — adaptación 3.0 → 4.0

```
src/infrastructure/v3_compat.rs
```

Tareas:
- [ ] Normalizar TYPE a lowercase (`TYPE=CELL` → `TYPE=cell`)
- [ ] Filtrar propiedades obsoletas: AGENT, LABEL, MAILER
- [ ] Mapa `V3_TO_V4_PROPERTY_MAP` para renombrar propiedades
- [ ] Tests: `test_v3_type_lowercase`, `test_v3_agent_ignored`

### 1.3 `infrastructure::source` — detección de fuente

```
src/infrastructure/source.rs
```

Tareas:
- [ ] `detect_source(prodids: &[String], uids: &[String]) -> (ContactSource, VCardVersion)`
- [ ] Buscar en PRODID: "ProtonMail"/"Proton AG" → Proton, "Google" → Google, "Apple" → Apple
- [ ] Buscar en UID: "proton-autosave" → Proton, "proton-import" → ProtonImport, "proton-web" → ProtonWeb
- [ ] Detectar `VERSION:3.0` vs `VERSION:4.0`
- [ ] Tests: todos los de `test_detect_*` en test-plan.md

### 1.4 `infrastructure::encoding` — transcodificación

```
src/infrastructure/encoding.rs
```

Tareas:
- [ ] `ensure_utf8(input: &[u8]) -> Result<String, CribaError>`
- [ ] chardetng para detectar codificación
- [ ] encoding_rs para transcodificar ISO-8859-1/Windows-1252 → UTF-8
- [ ] Tests: `test_utf8_passthrough`, `test_iso_to_utf8`

**Criterio de aceptación:** `cargo test` con el archivo `sample-contacts.vcf` → parseado sin errores, >5 contactos.

---

## Fase 2 — `ParsedVCard::into_contact()` + Screening

**Objetivo:** Convertir `ParsedVCard` → `Contact` y aplicar `decide()`.

**Referencia del plan:** §9.4 (model.rs), §3 (screening), §0.4 (DecisionTrace y decide())

**Módulos a implementar:**

### 2.1 `ParsedVCard::into_contact()`

```
src/infrastructure/parser.rs   ← añadir impl ParsedVCard
```

Tareas:
- [ ] Mapear `fn_raw` → `fn_value` (aplicando `unescape`)
- [ ] Extraer `structured_name` desde `n_raw`
- [ ] Mapear `org_raw` → `org`
- [ ] Mapear `emails_raw` → `emails` (Vec<TypedValue>)
- [ ] Mapear `tels_raw` → `tels` (Vec<Tel>, sin normalizar aún)
- [ ] Mapear `title_raw`, `role_raw`, `note_raw`
- [ ] Asignar `source_detail` desde `detect_source()`
- [ ] Tests: `test_into_contact_unescapes`, `test_into_contact_drops_agent`

### 2.2 `domain::screening::decide()` — implementar

```
src/domain/screening.rs   ← expandir desde stubs actuales
```

Tareas:
- [ ] `pub fn decide(contact: &Contact, config: &ScreeningConfig) -> DecisionTrace`
- [ ] Evaluar en orden: C1-C7 → E2 → E1 → E3 → E4 → E6 → Default
- [ ] `check_c2_juzgado()`: buscar "Juzgado", "Fiscalía", "TSJ", "GVA", "Generalitat" en ORG
- [ ] `check_e2()`: buscar keywords en ROLE/GENDER/NOTE
- [ ] `is_email_only()`: FN contiene "@", sin ORG, sin TEL
- [ ] `is_huerfano()`: sin EMAIL, sin TEL, sin ADR
- [ ] Tests: todos los de `test_c2_*`, `test_e1_*`, `test_e2_*`, `test_e3_*` en spec.md

### 2.3 Normalización (`application::cribar`)

```
src/application/cribar.rs   ← expandir
src/domain/contact.rs       ← añadir métodos de normalización si es necesario
```

Tareas:
- [ ] `normalize_fn(fn_val: &str) -> (String, Option<String>, Option<String>)` — N1-N7
  - Eliminar títulos (N4): Dr., Dra., Ilmo., Sr., Sra., Excmo., D., Dña.
  - Eliminar cargos (N5): Juez, Fiscal, Letrado, Procurador, Secretario
  - Normalizar capitalización (N7): cada palabra con mayúscula inicial, respetar siglas
- [ ] `normalize_tel(tel: &str, prefijo_pais: &str) -> Tel` — T1-T4
  - Limpiar espacios, guiones, puntos
  - Detectar prefijo o asumir país por defecto
  - Normalizar TYPE
- [ ] `normalize_org(org: &str) -> (String, Option<String>, Option<String>)` — ORG
  - Eliminar formas jurídicas: S.L., S.A., S.L.P., S.C.P.
  - Guardar en `org_legal_form`
- [ ] Tests: spec.md fase 3

---

## Fase 3 — Clasificación + Writer VCF

**Objetivo:** Clasificar contactos y serializar a VCF 4.0.

### 3.1 `domain::classification` — ya parcialmente implementado

```
src/domain/classification.rs   ← revisar/expandir
```

Ya tiene `classify()` con 5 tests. Verificar que cubre todos los patrones de la taxonomía.

### 3.2 `infrastructure::writer` — serialización VCF 4.0

```
src/infrastructure/writer.rs   ← implementar
```

Tareas:
- [ ] `write_vcf(contacts: &[Contact], parsed: &[ParsedVCard]) -> String`
- [ ] Emitir `BEGIN:VCARD`/`END:VCARD`
- [ ] Para propiedades no modificadas (PHOTO, NOTE, etc.) → emitir desde `ParsedVCard.raw_lines`
- [ ] Para propiedades modificadas (FN, TEL, ORG) → emitir desde `Contact`
- [ ] Añadir `CATEGORIES`, `X-CRIBADO-DATE`, `X-CRIBADO-VERSION`, `X-CRIBADO-ACCION`
- [ ] `fold_line(line: &str) -> String`: plegar a 75 octetos sin partir UTF-8 multibyte
- [ ] Tests: `test_write_vcf_4_0_version`, `test_folding_75_octets`, `test_folding_no_multibyte_split`, `test_photo_roundtrip`, `test_v3_props_not_emitted`

---

## Fase 4 — Auditoría + Export + Stats

### 4.1 `infrastructure::tsv_writer` — audit log

```
src/infrastructure/tsv_writer.rs
```

Tareas:
- [ ] `AuditLog::record(entry: AuditEntry)`
- [ ] `AuditLog::write_tsv(path: &Path) -> Result<(), CribaError>`
- [ ] Columnas: TIMESTAMP, UID, FN_ORIGINAL, FN_FINAL, ACCION, MOTIVO, CATEGORIAS, SOURCE_DETAIL
- [ ] Tests: `test_tsv_output`, `test_source_detail`

### 4.2 Export CSV/JSON

```
src/infrastructure/csv_writer.rs
src/infrastructure/json_writer.rs
```

Tareas:
- [ ] `export_csv(contacts: &[Contact], path: &Path)` — columnas FN, EMAIL, TEL, CATEGORIES, SOURCE
- [ ] `export_json(contacts: &[Contact], path: &Path)` — array JSON con serde
- [ ] Tests: `test_export_csv`, `test_export_json`

### 4.3 `application::stats`

```
src/application/stats.rs
```

Tareas:
- [ ] `compute_stats(contacts: &[Contact]) -> Stats`
- [ ] Totales: entrada, conservados, eliminados, fusionados, modificados
- [ ] Por categoría N1/N2
- [ ] Por origen (source_detail)
- [ ] Display formateado (texto, JSON, Markdown)

---

## Fase 5 — Pipeline completo (`application::cribar`)

**Objetivo:** Conectar todas las fases en `application::cribar::execute()`.

Tareas:
- [ ] Leer archivo → `ensure_utf8` → `unfold` → `parse_vcards`
- [ ] `detect_source` + `ParsedVCard::into_contact()` para cada uno
- [ ] Para cada Contact: `decide()` → filtrar eliminados → `normalize_*()` → `classify()`
- [ ] `deduplicate()` sobre conservados
- [ ] `write_vcf()` + `write_tsv()`
- [ ] `compute_stats()` + mostrar resumen
- [ ] Tests de integración: pipeline completo con archivo real

---

## Fase 6 — Configuración TOML

```
src/infrastructure/config.rs
```

Tareas:
- [ ] `load_config(path: Option<&Path>) -> Result<ScreeningConfig, CribaError>`
- [ ] Cargar `cribador.toml`
- [ ] Herencia: añadir reglas de usuario a las estándar (default)
- [ ] `replace = true` → sustituir todas las reglas
- [ ] `conservar_dominios`, `e2_keywords`, `prefijo_pais`
- [ ] Tests: `test_load_toml_append`, `test_load_toml_replace`

---

## Fase 7 — Release v0.1.0

- [ ] `cargo build --release` → binario < 8 MB
- [ ] `cargo test` → todos pasando (≥60 unitarios + integración)
- [ ] `cargo clippy` → sin warnings
- [ ] `cargo fmt` → formateado
- [ ] Probar con `sample-contacts.vcf` real
- [ ] Git tag `v0.1.0`

---

## Orden recomendado de implementación

```
Fase 1 (parser) → Fase 2 (screening + normalización)
→ Fase 3 (classify + writer) → Fase 4 (audit + export + stats)
→ Fase 5 (pipeline) → Fase 6 (config) → Fase 7 (release)
```

Cada fase es independiente para testing pero dependiente para integración.

---

## Método de verificación por razonamiento puro

Al finalizar cada fase, los resultados del pipeline deben contrastarse contra
un análisis independiente sin usar el programa — solo comandos shell,
scripting externo (Python/awk) y razonamiento sobre las reglas.

### Procedimiento

1. **Script externo**: implementar las mismas reglas del pipeline en un
   script Python o awk que opere directamente sobre el VCF raw
2. **Contar por categoría**: el script debe clasificar cada contacto y
   producir conteos C2, C3, C4, C6, E1, E3, Default
3. **Comparar totales**: los conteos del script deben coincidir con los
   del pipeline
4. **Si no coinciden**:
   - Listar UIDs de cada categoría en ambos lados
   - Encontrar los contactos discrepantes (diff de conjuntos)
   - Examinar los VCARDs crudos de los discrepantes para entender
     qué regla se aplica en cada lado
   - Corregir el script o el pipeline según corresponda

### Ejemplo: verificación Fase 2

```
Script Python (reglas replicadas):
  TOTAL: 475
  C6: 140
  E1: 254
  Default: 81
  → Conservados: 221, Eliminados: 254

Pipeline Rust:
  TOTAL: 475
  Conservados: 221
  Eliminados: 254
  → Coincidencia exacta ✅
```

### Errores típicos detectados por este método

- Script no hace unfold → propiedades en líneas plegadas invisibles
- Script no detecta propiedades agrupadas (ITEM1.EMAIL → cuentan como EMAIL)
- Regla C4 con chequeo de email además de ORG (script solo miraba ORG)
- ORG con `\n` dentro (plegado): el script lo captura, el parser Rust también

