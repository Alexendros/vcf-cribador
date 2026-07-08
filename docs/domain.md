# vcf-cribador — Modelo de dominio (DDD)

**Versión:** 0.1.0
**Fecha:** 2026-07-07

---

## Lenguaje ubicuo

| Término | Tipo | Definición |
|---------|------|------------|
| **ParsedVCard** | DTO (infra) | Registro bruto tras parser RFC. Contiene propiedades raw, binarios, escapes, y semántica de intercambio (vCard 3.0/4.0). No pertenece al dominio. |
| **Contact** | Entidad | Representación canónica de un contacto. Identidad por `uid`. Sin escapes RFC ni datos binarios. |
| **StructuredName** | Value Object | Componentes del nombre: `family`, `given`, `additional`, `prefix`, `suffix` (RFC 6350). |
| **Tel** | Value Object | Número de teléfono normalizado a E.164, con `tel_type` y flag `normalized`. |
| **TypedValue** | Value Object | Valor con tipos y preferencia (EMAIL, TEL sin normalizar). |
| **CategorySet** | Value Object | Conjunto de categorías N1, N2, N3 asignadas. |
| **ScreeningDecision** | Value Object (enum) | Resultado del cribado: `Conserved`, `Eliminated(E1,E3,E4,E6)`, `Quarantine(E4,E6)`, `NeedsReview(E2,D3,D4,D5,D6)`. |
| **DecisionTrace** | Value Object | Traza inmutable de la decisión: `outcome`, `triggered_rule`, `evidence`, `timestamp`. |
| **DuplicateCluster** | Entidad (efímera) | Componente conexo de contactos que comparten identidad (D1-D2). Existe solo durante la deduplicación. |
| **AuditEntry** | Value Object | Registro inmutable de trazabilidad: timestamp, uid, fn_original, fn_final, action, motivo, categorias, source_detail. |
| **ClassificationRule** | Value Object | Par `(Regex, n1, n2)` que asigna categorías. Cargado desde reglas estándar o TOML. |
| **ScreeningConfig** | Value Object | Configuración de cribado: `conservar_dominios`, `e2_keywords`, `prefijo_pais`. |
| **SourceDetail** | Value Object (enum) | Origen detallado: `ProtonAutosave`, `ProtonImport`, `ProtonWeb`, `Google`, `Apple`, `Unknown(String)`. |

---

## Entidades

### Contact

```
Contact
├── uid: String                          # Identidad
├── fn_value: String                     # Nombre canónico (N1-N7 aplicadas)
├── structured_name: Option<StructuredName>
├── org: Option<String>                  # ORG normalizado (sin formas jurídicas)
├── org_fullname: Option<String>         # Siglas expandidas
├── org_legal_form: Option<String>       # "S.L.", "S.A.", "S.L.P."
├── emails: Vec<TypedValue>
├── tels: Vec<Tel>
├── title: Option<String>                # Extraído de FN (N4)
├── role: Option<String>                 # Extraído de FN (N5)
├── note: Option<String>
├── categories: CategorySet
├── source_detail: SourceDetail
├── decision: ScreeningDecision
├── merged_uids: Vec<String>             # UIDs absorbidos en fusión
```

**Identidad:** `uid` es el identificador canónico. Para vCard 3.0 sin UID, se genera uno determinista basado en FN + EMAIL + TEL.

**Invariantes de entidad:**
- `fn_value` no puede contener `@` (I2)
- Si `decision == Conserved`, `categories` debe tener al menos una categoría N1 (I1)
- `merged_uids` solo se puebla durante la fusión; no se modifica fuera de `identity.rs`

---

## Value Objects

### StructuredName

```
StructuredName
├── family: Vec<String>      # Apellidos
├── given: Vec<String>       # Nombres
├── additional: Vec<String>  # Nombres adicionales
├── prefix: Vec<String>      # Prefijos (Dr., Sr.)
└── suffix: Vec<String>      # Sufijos (Jr., III)
```

Se construye desde `N:family;given;additional;prefix;suffix` (vCard 4.0) o se infiere desde FN con heurística española.

### Tel

```
Tel
├── value: String        # "+34612345678" o valor original si non_normalizable
├── tel_type: TelType    # Cell | Home | Work | Main | Other
└── normalized: bool     # true si se normalizó a E.164
```

### DecisionTrace

```
DecisionTrace
├── outcome: ScreeningDecision
├── triggered_rule: String   # "C2-Juzgado", "E1", "Default"
├── evidence: String         # "ORG contiene 'Juzgado'"
└── timestamp: Timestamp
```

Inmutable una vez creado. Se adjunta al `AuditEntry`.

### CategorySet

```
CategorySet
├── n1: HashSet<String>      # "PROF", "INST", "FIN", ...
├── n2: HashSet<String>      # "PROF-JUD", "INST-AUT", ...
└── n3: Vec<String>          # Etiquetas libres
```

### AuditEntry

```
AuditEntry
├── timestamp: String
├── uid: String
├── fn_original: String
├── fn_final: String
├── action: String           # "CONSERVADO" | "ELIMINADO" | "FUSIONADO" | "MODIFICADO" | "CUARENTENA"
├── motivo: String           # Código de regla + evidencia
├── categorias: String       # "PROF,PROF-JUD"
└── source_detail: String    # "proton-autosave", "google", "apple"
```

---

## Reglas de dominio

### Cribado (screening)

| Código | Regla | Decisión |
|--------|-------|----------|
| C1 | Contacto profesional activo | Conserved |
| C2 | ORG contiene "Juzgado", "Fiscalía", "TSJ", "GVA", "Generalitat" | Conserved |
| C3 | ORG contiene "ICAV", "ICAB" o colegio profesional | Conserved |
| C4 | Entidad financiera con relación vigente | Conserved |
| C5 | Soporte técnico con garantía/suscripción activa | Conserved |
| C6 | Personal con TEL presente | Conserved |
| C7 | Institución educativa con matrícula activa | Conserved |
| E1 | FN es email sin ORG ni TEL | Eliminated |
| E2 | ROLE/GENDER con contenido inapropiado | NeedsReview (limpiar, conservar) |
| E3 | Sin EMAIL, TEL ni ADR | Eliminated |
| E4 | Servicio descontinuado | Quarantine |
| E6 | Inactivo > 5 años | Quarantine |

**Orden de evaluación:** C1-C7 → E2 → E1 → E3 → E4 → E6 → Default (Conserved).

### Normalización

| Código | Regla |
|--------|-------|
| N1 | FN = "Nombre Apellido1 Apellido2" |
| N2 | N estructurado → regenerar FN |
| N4 | Títulos de FN → TITLE |
| N5 | Cargos de FN → ROLE |
| N6 | FN=email sin ORG → E1; con ORG → usar ORG |
| N7 | Capitalización con respeto de siglas |
| T1 | TEL a E.164 (+34XXXXXXXXX) |
| T4 | TYPE normalizado: CELL, HOME, WORK, MAIN |

### Deduplicación

| Nivel | Criterio | Confianza | Acción |
|-------|----------|-----------|--------|
| D1 | Mismo UID | 100% | Fusión automática |
| D2 | Mismo FN + (mismo TEL o mismo EMAIL) | 95% | Fusión automática |
| D3 | Mismo FN + diferente TEL/EMAIL | 80% | Propuesta |
| D4 | Mismo TEL + diferente FN | 70% | Alerta |
| D5 | Mismo EMAIL + diferente FN | 70% | Alerta |
| D6 | Similitud FN > 85% (Levenshtein) | 60% | Propuesta |
