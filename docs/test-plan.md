# vcf-cribador — Plan de testing

**Versión:** 0.1.0
**Fecha:** 2026-07-07

---

## Estrategia

| Capa | Framework | Enfoque |
|------|-----------|---------|
| **Unitarios** | `cargo test` (built-in) | Cada módulo de `domain/` e `infrastructure/` testado en aislamiento |
| **Integración** | `cargo test` en `tests/` | Pipeline completo con archivos VCF reales |
| **Invariantes** | Tests dedicados en `domain/` | Validan I1-I7 contra mocks y datos sintéticos |
| **Fuzz** | `cargo fuzz` (futuro) | Parser con inputs malformados |

---

## Tests unitarios por módulo

### `domain::contact`

| Test | Descripción |
|------|-------------|
| `test_structured_name_from_n` | `N:Apellidos;Nombre;;;` → StructuredName correcto |
| `test_tel_e164` | Tel normalizado con prefijo |
| `test_tel_non_normalizable` | Tel sin prefijo marcado como no normalizable |
| `test_category_set_n1_required` | CategorySet sin N1 → error |

### `domain::screening`

| Test | Descripción |
|------|-------------|
| `test_c2_juzgado_overrides_e1` | Contacto con FN=email pero ORG="Juzgado Instrucción 9" → Conserved |
| `test_e2_sanitize_but_preserve` | Contacto con ROLE ofensivo → NeedsReview, datos de contacto intactos |
| `test_e1_email_only_no_rescue` | FN="info@unknown.com", sin ORG, sin TEL → Eliminated(E1) |
| `test_e3_huerfano` | Sin EMAIL, sin TEL → Eliminated(E3) |
| `test_e4_quarantine` | Servicio descontinuado → Quarantine(E4) |
| `test_default_conserved` | Contacto sin regla específica → Conserved |
| `test_decision_trace_immutable` | DecisionTrace no modificable tras creación |
| `test_precedence_order` | C2 se evalúa antes que E1 |
| `test_all_conserved_have_n1` | Invariante I1 |
| `test_fn_no_at_sign` | Invariante I2 |

### `domain::classification`

| Test | Descripción |
|------|-------------|
| `test_juzgado_matched` | ORG="Juzgado Instrucción 9" → contiene "PROF-JUD" |
| `test_domain_matched` | EMAIL en dominio institucional → categoría correcta |
| `test_multiple_categories` | ORG + EMAIL combinados → múltiples categorías |
| `test_crypto_matched` | ORG financiero → contiene "FIN-CRYPTO" |
| `test_no_match_defaults` | Sin ORG ni patrones → N1 por defecto |
| `test_all_rules_compile` | Todos los regex de reglas estándar compilan sin error |

### `domain::identity`

| Test | Descripción |
|------|-------------|
| `test_d1_same_uid` | 2 contactos con mismo UID → 1, merged_uids=[uid2] |
| `test_d2_same_fn_tel` | Mismo FN + mismo TEL → fusión automática |
| `test_d2_same_fn_email` | Mismo FN + mismo EMAIL → fusión automática |
| `test_d2_transitive` | A↔B por TEL, A↔C por EMAIL → A, B, C fusionados |
| `test_d2_transitive_complex` | A↔B por TEL, B↔C por EMAIL, D↔C por TEL → A,B,C,D fusionados |
| `test_action_preserved_on_merge` | Contacto Modified + fusión → action sigue Modified |
| `test_no_duplicates` | 3 contactos distintos → sin fusión |
| `test_merge_preserves_most_complete_fn` | FN más largo sobrevive |
| `test_merge_union_tels` | TELs de ambos en resultado |
| `test_merge_union_emails` | EMAILs de ambos en resultado |

### `infrastructure::parser`

| Test | Descripción |
|------|-------------|
| `test_unfold_crlf_space` | Líneas plegadas con espacio |
| `test_unfold_crlf_tab` | Líneas plegadas con tab |
| `test_unescape_double_backslash` | `\\` → `\` |
| `test_unescape_escaped_semicolon` | `\;` → `;` |
| `test_unescape_escaped_comma` | `\,` → `,` |
| `test_unescape_escaped_newline` | `\n` → U+000A |
| `test_unescape_backslash_semicolon_bug` | `\\;` → `\;` (regresión) |
| `test_unescape_mixed_escapes` | Combinación compleja |
| `test_unescape_no_escapes` | Texto sin escapes intacto |
| `test_parse_single_vcard_4_0` | VCard 4.0 mínimo |
| `test_parse_multi_vcard_4_0` | 3 VCards concatenados → Vec de 3 |
| `test_parse_single_vcard_3_0` | VCard 3.0 con Google PRODID |
| `test_parse_grouped_property` | `ITEM1.EMAIL;PREF=1:x@y.com` → group="ITEM1" |
| `test_parse_n_structured` | `N:Apellidos;Nombre;;;` → StructuredName |
| `test_parse_params_multi_value` | `TEL;TYPE=CELL,VOICE:+34...` → TYPE=[CELL, VOICE] |
| `test_photo_raw_preserved` | PHOTO → raw_lines poblado, sin desescapar |
| `test_logo_sound_key_raw` | LOGO, SOUND, KEY → preservados |
| `test_data_uri_raw` | `EMAIL;VALUE=uri:data:...` → raw |
| `test_into_contact_unescapes` | ParsedVCard con escapes → Contact sin escapes |
| `test_into_contact_drops_agent` | AGENT en v3 → no aparece en Contact |

### `infrastructure::writer`

| Test | Descripción |
|------|-------------|
| `test_write_vcf_4_0_version` | Salida contiene `VERSION:4.0` |
| `test_folding_75_octets` | Línea >75 bytes → plegada con CRLF SPACE |
| `test_folding_no_multibyte_split` | Carácter UTF-8 de 2+ bytes no se parte |
| `test_photo_roundtrip` | PHOTO raw → preservado byte-identical |
| `test_v3_props_not_emitted` | AGENT, LABEL, MAILER no aparecen en salida |

### `infrastructure::encoding`

| Test | Descripción |
|------|-------------|
| `test_utf8_passthrough` | Entrada UTF-8 → sin cambios |
| `test_iso_to_utf8` | Entrada ISO-8859-1 → UTF-8 correcto |
| `test_invalid_utf8_replacement` | Bytes inválidos → U+FFFD |

### `infrastructure::source`

| Test | Descripción |
|------|-------------|
| `test_detect_proton_prodid` | PRODID contiene "ProtonMail" → Proton |
| `test_detect_google_prodid` | PRODID contiene "Google Inc" → Google |
| `test_detect_apple_prodid` | PRODID contiene "Apple Inc." → Apple |
| `test_detect_proton_autosave_uid` | UID contiene "proton-autosave" → Proton |
| `test_detect_proton_import_uid` | UID contiene "proton-import" → ProtonImport |
| `test_detect_proton_web_uid` | UID contiene "proton-web" → ProtonWeb |
| `test_detect_unknown` | Sin PRODID ni UID reconocible → Unknown |

---

## Tests de integración (`tests/`)

| Test | Descripción | Archivo de prueba |
|------|-------------|-------------------|
| `test_cribar_proton_real` | Pipeline completo con archivo ProtonMail | `sample-contacts-2026-07-07.vcf` |
| `test_cribar_google_v3` | Pipeline con VCF Google vCard 3.0 | `fixtures/google_sample.vcf` |
| `test_cribar_apple_v3` | Pipeline con VCF Apple vCard 3.0 | `fixtures/apple_sample.vcf` |
| `test_cribar_dry_run` | `--dry-run` no crea archivos, reporta en stdout | Cualquier VCF |
| `test_pipeline_completo` | cribar → stats → verificar criterios §spec | sample-contacts |
| `test_archivo_vacio` | VCF sin VCARDs → error controlado | `fixtures/empty.vcf` |
| `test_archivo_malformado` | Texto no VCF → error descriptivo | `fixtures/malformed.txt` |
| `test_encoding_iso` | VCF ISO-8859-1 → transcodifica y procesa | `fixtures/iso_sample.vcf` |
| `test_config_custom_rules` | `--config custom.toml` → reglas extra aplicadas | `fixtures/custom.toml` |
| `test_export_csv_roundtrip` | cribar → export csv → columnas correctas | sample-contacts |
| `test_export_json_roundtrip` | cribar → export json → JSON válido | sample-contacts |
| `test_dedup_transitivo_real` | Verificar fusión transitiva en datos reales | sample-contacts |
| `test_stats_accuracy` | stats refleja conteos reales de salida | sample-contacts |
| `test_audit_tsv_completeness` | audit.tsv tiene una fila por contacto | sample-contacts |

---

## Fixtures necesarias

```
tests/fixtures/
├── empty.vcf                 # VCF sin contactos
├── malformed.txt             # Texto no VCF
├── iso_sample.vcf            # VCF en ISO-8859-1
├── google_sample.vcf         # Export de Google Contacts (v3.0)
├── apple_sample.vcf          # Export de iCloud (v3.0)
├── custom.toml               # Configuración con reglas extra
├── single_v4.vcf             # Un solo contacto vCard 4.0
├── duplicates.vcf            # Contactos con duplicados D1, D2, transitivos
├── edge_cases.vcf            # Escapes, binarios, títulos, cargos, email-only
└── proton_sample.vcf         # Subconjunto anonimizado del archivo real
```

---

## Cobertura objetivo

| Tipo | Objetivo |
|------|----------|
| Líneas | ≥ 80% |
| Ramas | ≥ 75% |
| Domain | 100% |
| Infrastructure (parser) | ≥ 90% |
| Infrastructure (writer) | ≥ 85% |
