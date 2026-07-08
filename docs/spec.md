# vcf-cribador — Especificación del producto

**Versión:** 0.1.0
**Fecha:** 2026-07-07
**Estado:** Draft

---

## Objetivos

| ID | Objetivo | Criterio de aceptación |
|----|----------|------------------------|
| **O1** | Parsear VCF vCard 4.0 y 3.0 (Proton, Google, Apple) | 100% contactos parseados de los archivos de referencia (Proton 4.0, Google 3.0, Apple 3.0) |
| **O2** | Cribar aplicando reglas E1-E6 + C1-C7 con precedencia determinista | Tasa de reducción 5-30% sobre el archivo de referencia |
| **O3** | Normalizar FN, N, TEL, ADR, ORG según RFC 6350 y reglas españolas | 0 FN con "@" en salida, 100% TEL en E.164 o marcados `non_normalizable` |
| **O4** | Clasificar automáticamente con taxonomía de 3 niveles | 100% contactos conservados con al menos 1 categoría N1 |
| **O5** | Detectar y fusionar duplicados D1-D2 con cierre transitivo | 0% duplicados D1-D2 residuales tras fusión |
| **O6** | Generar audit.log trazable en TSV | Una entrada por cada contacto procesado, con `DecisionTrace` completo |
| **O7** | Detectar fuente automáticamente (Proton/Google/Apple) y versión vCard | `source_detail` correcto en audit.log y stats |
| **O8** | Exportar a CSV y JSON además de VCF 4.0 | Formatos de exportación válidos y completos |
| **O9** | Cargar reglas personalizadas desde TOML | Reglas de usuario se añaden a las estándar; flag `replace` para sustitución total |

---

## No-objetivos (v1.0)

- No conectarse a APIs externas (CardDAV, ProtonMail, Google People)
- No implementar GUI ni TUI interactiva (pospuesto a v0.4.0)
- No dar soporte a vCard 2.1
- No modificar datos binarios (PHOTO, LOGO, SOUND, KEY): solo preservarlos o ignorarlos

---

## Invariantes de dominio

1. **I1 — Integridad:** Todo `Contact` conservado debe tener `uid`, `ScreeningDecision::Conserved`, `source_detail` y al menos una categoría N1.
2. **I2 — FN canónico:** Ningún `Contact.fn_value` final puede contener `@`.
3. **I3 — TEL E.164:** Todo `Contact.tel` normalizado debe estar en formato E.164 (`+` seguido de dígitos) o marcado explícitamente como `non_normalizable`.
4. **I4 — Salida canónica:** Toda salida es vCard 4.0 (RFC 6350). El folding de líneas respeta 75 octetos y nunca corta en medio de un carácter UTF-8 multibyte.
5. **I5 — Compatibilidad de entrada:** Entrada vCard 3.0 aceptada. Propiedades obsoletas (AGENT, LABEL, MAILER) no se propagan a la salida 4.0.
6. **I6 — Auditabilidad:** Toda acción sobre un contacto genera una entrada inmutable en `audit.tsv` con `DecisionTrace`.
7. **I7 — No destrucción:** El archivo VCF original nunca se sobrescribe. La salida se escribe en una ruta distinta.

---

## Criterios de aceptación por fase

### Fase 1 — Parseo

- [ ] Archivo ProtonMail vCard 4.0 (~2 KB sintético, ~145 contactos): parseado sin errores, sin panics
- [ ] Archivo Google Contacts vCard 3.0 de muestra: parseado sin errores
- [ ] Archivo Apple iCloud vCard 3.0 de muestra: parseado sin errores
- [ ] Archivo vacío (0 contactos): error controlado, no panic
- [ ] Archivo malformado: error descriptivo con línea y contexto
- [ ] Archivo ISO-8859-1: detectado y transcodificado a UTF-8 automáticamente
- [ ] Propiedades binarias (PHOTO, LOGO, SOUND, KEY): preservadas byte-identical
- [ ] Fotos base64 con líneas plegadas: unfold + preservación correcta

### Fase 2 — Cribado

- [ ] Contactos C2 (juzgados) prevalecen sobre E1 (email-only)
- [ ] Contactos E2 (metadatos inapropiados): limpiados pero conservados
- [ ] Contactos E1 sin rescate: eliminados con registro en audit.log
- [ ] Contactos E3 (huérfanos): eliminados
- [ ] Contactos E4/E6: enviados a cuarentena
- [ ] `DecisionTrace` registra regla disparada, evidencia y timestamp

### Fase 3 — Normalización

- [ ] `Ilmo. Sr. Juan Pérez` → FN=`Juan Pérez`, TITLE=`Ilmo. Sr.`
- [ ] `Carlos Ruiz Juez` → FN=`Carlos Ruiz`, ROLE=`Juez`
- [ ] `JUZGADO INSTRUCCIÓN 9` → FN=`Juzgado Instrucción 9`
- [ ] `ICAV turno oficio` → FN=`ICAV Turno Oficio`
- [ ] `612345678` → TEL=`+34612345678`
- [ ] `TYPE=iPhone` → `TYPE=CELL`
- [ ] `Despacho Legal S.L.P.` → ORG=`Despacho Legal`, X-ORG-LEGAL-FORM=`S.L.P.`

### Fase 4 — Clasificación

- [ ] Contacto con ORG `Juzgado Instrucción 9` → categorías `PROF,PROF-JUD`
- [ ] Contacto con EMAIL `@example.org` → categorías `INST,INST-AUT`
- [ ] Contacto con ORG `Exchange Support` → categorías `FIN,FIN-CRYPTO`
- [ ] Contacto combinado (juzgado + @example.org) → `PROF,PROF-JUD,INST,INST-AUT`
- [ ] Contacto sin patrones → categoría N1 inferida por defecto

### Fase 5 — Deduplicación

- [ ] D1: mismo UID → fusión automática, `merged_uids` poblado
- [ ] D2: mismo FN + mismo TEL → fusión automática
- [ ] D2 transitivo: A↔B por TEL, A↔C por EMAIL → A, B, C fusionados
- [ ] `action` original del contacto absorbente se conserva tras fusión
- [ ] D3-D6: propuestas registradas en NOTE, no fusión automática

### Fase 6 — Verificación

- [ ] Invariantes I1-I7 comprobados en el pipeline completo
- [ ] `audit.tsv` contiene una fila por cada contacto procesado
- [ ] `stats` muestra totales correctos (entrada, conservados, eliminados, fusionados)

### Fase 7 — Exportación

- [ ] VCF 4.0 de salida válido según RFC 6350
- [ ] CSV con columnas FN, EMAIL, TEL, CATEGORIES, SOURCE
- [ ] JSON array con todos los contactos conservados

---

## Métricas de calidad

| Métrica | Objetivo |
|---------|----------|
| Cobertura de tests unitarios | ≥ 80% |
| Tests de integración | Mínimo 10 escenarios |
| Tiempo de ejecución (archivo ~2 KB sintético) | < 1 segundo |
| Binario release | < 8 MB |
| Sin panics en inputs malformados | 100% |
