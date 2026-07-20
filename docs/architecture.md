# vcf-cribador — Arquitectura

**Versión:** 0.1.0
**Fecha:** 2026-07-07

---

## Principios

- **Clean Architecture:** dependencias apuntan hacia el centro. `domain` no depende de nada externo. `application` orquesta `domain`. `infrastructure` implementa adaptadores.
- **DDD:** bounded contexts independientes con lenguaje ubicuo compartido.
- **Spec-Driven:** criterios de aceptación e invariantes guían el desarrollo y los tests.
- **Event Storming:** comandos, eventos y excepciones documentan el flujo.

---

## Capas

```
┌──────────────────────────────────────┐
│           interfaces/cli.rs           │  ← Clap (único punto de entrada)
├──────────────────────────────────────┤
│           application/                │  ← Casos de uso (Cribar, Audit, Stats, Export)
├──────────────────────────────────────┤
│  ┌────────────┐  ┌────────────────┐  │
│  │  domain/   │  │ infrastructure/ │  │
│  │            │  │                 │  │  │  │ Contact    │  │ parser.rs       │  │
  │  │ Decision-  │  │ writer.rs       │  │
  │  │ Trace      │  │ encoding.rs     │  │
  │  │ classify() │  │ v3_compat.rs    │  │
  │  │ dedup()    │  │ source.rs       │  │
  │  │ decide()   │  │ config.rs       │  │
  │  │ rules      │  │ csv/json/tsv    │  │
  │  │ audit      │  │                 │  │
  │  │ verify     │  │                 │  │
│  └────────────┘  └────────────────┘  │
└──────────────────────────────────────┘

Regla de dependencia:
  domain ← application ← interfaces
  domain ← infrastructure
  application → infrastructure (vía traits)
```

---

## Bounded Contexts

### 1. IngestionContext

**Responsabilidad:** Leer bytes, detectar codificación, transcodificar a UTF-8, desplegar líneas, parsear vCard 3.0/4.0 → `ParsedVCard`, detectar fuente y versión.

**Módulos:**
- `infrastructure/encoding.rs` — chardetng + encoding_rs
- `infrastructure/parser.rs` — unfold, unescape, nom parsers
- `infrastructure/v3_compat.rs` — adaptación vCard 3.0 → 4.0
- `infrastructure/source.rs` — detección de fuente (PRODID, UID)

**Entrada:** `Vec<u8>`
**Salida:** `Vec<ParsedVCard>`

### 2. QualityContext

**Responsabilidad:** Cribado (E1-E6, C1-C7) con precedencia determinista, normalización (FN, N, TEL, ADR, ORG).

**Módulos:**
- `domain/screening.rs` — `decide()`, `DecisionTrace`, `ScreeningDecision`
- `domain/contact.rs` — `Contact`, `StructuredName`, `Tel`, `Address`
- `domain/verification.rs` — verificación de invariantes I1-I7
- `application/cribar.rs` — `normalize_contact()`

**Entrada:** `Vec<ParsedVCard>`
**Salida:** `Vec<Contact>` con `decision` y campos normalizados

### 3. TaxonomyContext

**Responsabilidad:** Asignar categorías N1/N2/N3 mediante reglas regex. Cargar reglas desde TOML (herencia: añadir o reemplazar).

**Módulos:**
- `domain/rules.rs` — `ClassificationRule`, reglas estándar con N1/N2/N3
- `domain/classification.rs` — `classify()`, `CategorySet`
- `infrastructure/config.rs` — carga de `cribador.toml`

La taxonomía cubre profesionales jurídicos, instituciones públicas, finanzas (incl. crypto/fintech), educación, tecnología, comercio y servicios.

**Entrada:** `Vec<Contact>` + `Option<Config>`
**Salida:** `Vec<Contact>` con `categories` poblado

### 4. IdentityContext

**Responsabilidad:** Detectar duplicados D1-D2 con Union-Find (cierre transitivo), fusionar contactos, registrar propuestas D3-D6.

**Módulos:**
- `domain/identity.rs` — `deduplicate()`, `DuplicateCluster`, `merge_contacts()`

**Entrada:** `Vec<Contact>`
**Salida:** `Vec<Contact>` fusionado + `merged_uids` + propuestas en NOTE

### 5. OutputContext

**Responsabilidad:** Serializar a VCF 4.0, CSV, JSON, TSV. Folding 75 octetos. Preservación de binarios.

**Módulos:**
- `infrastructure/writer.rs` — `write_vcf()`, `fold_line()`
- `infrastructure/csv_writer.rs` — export CSV
- `infrastructure/json_writer.rs` — export JSON
- `infrastructure/tsv_writer.rs` — `write_audit_tsv()`

**Entrada:** `Vec<Contact>` + `Vec<AuditEntry>`
**Salida:** archivos en disco (VCF, CSV, JSON, TSV)

---

## Modelos y su tránsito

```
┌─────────────┐     into_contact()     ┌──────────┐
│ ParsedVCard │ ──────────────────────→ │ Contact  │
│ (infra)     │                        │ (domain) │
└─────────────┘                        └──────────┘
                                              │
                    ┌─────────────────────────┤
                    │                         │
               decide()                  classify()
                    │                         │
                    ▼                         ▼
            ScreeningDecision            CategorySet
                    │                         │
                    └─────────┬───────────────┘
                              │
                         deduplicate()
                              │
                              ▼
                      Vec<Contact> (final)
                              │
              ┌───────────────┼───────────────┐
              │               │               │
         write_vcf()    write_csv()    write_json()
              │               │               │
              ▼               ▼               ▼
         .vcf (4.0)      .csv            .json

  En paralelo: AuditEntry → write_tsv() → .tsv
```

---

## Árbol de dependencias entre módulos

```
main.rs
  → cli.rs
  → tracing_subscriber

cli.rs
  → application::cribar
  → application::audit
  → application::stats
  → application::export

application::cribar
  → infrastructure::encoding
  → infrastructure::parser
  → infrastructure::v3_compat
  → infrastructure::source
  → infrastructure::config
  → domain::screening
  → domain::contact
  → domain::classification
  → domain::identity
  → infrastructure::writer
  → infrastructure::tsv_writer

domain::screening → domain::contact, domain::rules
domain::classification → domain::rules
domain::identity → domain::contact
domain::audit → domain::contact, domain::screening
domain::verification → domain::contact, domain::screening
domain::contact → (sin dependencias)
domain::rules → regex (crate externa, sin lógica de negocio)

infrastructure::parser → nom (crate externa)
infrastructure::encoding → chardetng, encoding_rs
infrastructure::config → toml, domain::rules
```

---

## Traits de infraestructura (dependency inversion)

```rust
// domain/screening.rs — el dominio no conoce la infraestructura
pub trait ScreeningConfigProvider {
    fn conservar_dominios(&self) -> &[String];
    fn e2_keywords(&self) -> &[String];
}
```

La implementación concreta vive en `infrastructure/config.rs` y se inyecta en `application/cribar.rs`.
