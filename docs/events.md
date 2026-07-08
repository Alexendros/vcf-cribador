# vcf-cribador — Comandos, eventos y excepciones

**Versión:** 0.1.0
**Fecha:** 2026-07-07

---

## Comandos

| Comando | Descripción | Origen |
|---------|-------------|--------|
| `CribarContacts` | Ejecuta el pipeline completo: parseo → cribado → normalización → clasificación → deduplicación → verificación → exportación | CLI `cribar` |
| `AuditContacts` | Solo analiza el VCF sin modificarlo; genera audit.log | CLI `audit` |
| `ComputeStats` | Calcula estadísticas de un VCF ya procesado | CLI `stats` |
| `ExportContacts` | Exporta un VCF cribado a CSV o JSON | CLI `export` |

---

## Eventos

```
Comando CribarContacts
│
├── VCFDetected { path, size_bytes }
│
├── EncodingChecked { encoding, action }
│   ├── Acción: "passthrough" (ya UTF-8)
│   └── Acción: transcoded_from(original_encoding)
│
├── VCFParsed { version, contact_count, source }
│   ├── version: "3.0" | "4.0"
│   └── source: "proton" | "google" | "apple" | "unknown"
│
├── ContactScreened { uid, decision }
│   ├── decision: Conserved | Eliminated(code) | NeedsReview(reason) | Quarantine(code)
│   └── Por cada contacto del VCF
│
├── ContactNormalized { uid, changes[] }
│   ├── changes: ["fn_cleaned", "tel_to_e164", "org_stripped", ...]
│   └── Solo si hubo modificaciones
│
├── ContactClassified { uid, categories[] }
│   └── Al menos una categoría N1 por contacto conservado
│
├── DuplicatesMerged { base_uid, absorbed_uids[], count }
│   └── Emitido una vez por pipeline, con el total de fusiones
│
├── VerificationPassed { invariants[], warnings[] }
│   └── Si todas las invariantes I1-I7 se cumplen
│
├── VCFExported { output_path, contact_count }
│   └── VCF 4.0 escrito
│
└── AuditWritten { audit_path, entry_count }
    └── TSV con una fila por contacto procesado
```

---

## Excepciones

| Excepción | Descripción | Datos |
|-----------|-------------|-------|
| `ParseError` | Error de sintaxis vCard | `{ line, column, context, reason }` |
| `EncodingError` | Codificación no soportada o corrupta | `{ detected_encoding, action }` |
| `VerificationFailed` | Una invariante no se cumple | `{ invariant_id, contact_uid, detail }` |
| `ConfigError` | Error de sintaxis en TOML | `{ path, line, reason }` |
| `IOError` | Error de lectura/escritura | `{ path, kind }` |

---

## Eventos de decisión manual

Estos eventos se emiten cuando el sistema no puede decidir automáticamente. En v1.0 se registran en `audit.tsv` y en NOTE del contacto. En v0.4.0+ alimentarán el modo `review` interactivo.

```
NeedsReview { uid, reason, suggested_action }
├── reason: E2InappropriateMetadata
│   └── suggested_action: "sanitize_role_and_gender_fields"
├── reason: D3DuplicateCandidate
│   └── suggested_action: "review_manual_merge"
├── reason: D6FuzzyDuplicate
│   └── suggested_action: "review_fuzzy_match"

QuarantineExpired { uid, original_code, days_in_quarantine }
└── Emitido tras 180 días en cuarentena → sugiere eliminación definitiva
```

---

## Flujo CribarContacts

```
1. Leer bytes del archivo
2. ensure_utf8() → EncodingChecked
3. unfold_lines() → texto desplegado
4. parse_vcards() → Vec<ParsedVCard> → VCFParsed
5. Para cada ParsedVCard:
   a. into_contact() → Contact
   b. decide(contact) → ContactScreened
   c. Si Conserved: normalize(contact) → ContactNormalized
   d. Si Conserved: classify(contact) → ContactClassified
6. deduplicate(contacts) → DuplicatesMerged
7. verify(contacts) → VerificationPassed | VerificationFailed
8. write_vcf(contacts) → VCFExported
9. write_audit(entries) → AuditWritten
```

---

## Decisiones en el flujo

| Punto de decisión | Condición | Rama |
|-------------------|-----------|------|
| Contacto E1 sin rescate | FN es email, sin ORG, sin TEL | Eliminar → audit |
| Contacto E2 | ROLE/GENDER con keywords | Limpiar campos, conservar → audit |
| Contacto C2-C7 | ORG/dominio institucional | Conservar aunque FN sea email |
| Duplicados D1 | Mismo UID | Fusión automática |
| Duplicados D2 | Mismo FN + mismo TEL/EMAIL | Fusión automática con cierre transitivo |
| Duplicados D3-D6 | Coincidencia parcial | Propuesta en NOTE, no fusión automática |
| Invariante I1 fallida | Contacto conservado sin N1 | Asignar categoría por defecto → warning |
