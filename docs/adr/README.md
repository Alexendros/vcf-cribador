# ADR-001: Salida canónica siempre vCard 4.0

**Fecha:** 2026-07-07
**Estado:** Aceptado

---

## Contexto

Las fuentes de entrada pueden ser vCard 3.0 (Google Contacts, Apple iCloud) o vCard 4.0 (ProtonMail). El parser debe aceptar ambas, pero el sistema necesita un formato de salida único para simplificar el writer, la verificación y los exports.

## Decisión

**La salida canónica del pipeline será siempre vCard 4.0 (RFC 6350).**

## Consecuencias

- El parser acepta `VERSION:3.0` y `VERSION:4.0`.
- `v3_compat.rs` adapta propiedades 3.0 a 4.0 (normaliza TYPE a lowercase, ignora AGENT/LABEL/MAILER, convierte `TEL;TYPE=CELL` a `TEL;TYPE=cell`).
- El writer solo emite `VERSION:4.0` y sintaxis 4.0.
- Los contactos de Google/Apple se normalizan a 4.0 sin pérdida de información relevante.

## Alternativas consideradas

- Emitir la misma versión que la entrada: rechazado por complejidad (dos writers, dos juegos de tests).
- Convertir todo a 3.0: rechazado porque 4.0 es el estándar actual y soporta propiedades multi-valor con PREF que ProtonMail usa.

---

# ADR-002: Separación ParsedVCard (infra) / Contact (dominio)

**Fecha:** 2026-07-07
**Estado:** Aceptado

---

## Contexto

El parser produce estructuras con escapes RFC, propiedades binarias (PHOTO), y dependencias de sintaxis vCard. El dominio (cribado, normalización, clasificación) no debería conocer estos detalles.

## Decisión

**Mantener dos modelos separados: `ParsedVCard` en `infrastructure/parser.rs` y `Contact` en `domain/contact.rs`, con una función `ParsedVCard::into_contact()` que haga la traducción.**

## Consecuencias

- `Contact` no contiene `raw: bool`, ni líneas de PHOTO, ni escapes RFC.
- Toda la lógica de dominio opera sobre datos limpios y desescapados.
- El writer reconstruye propiedades raw desde `ParsedVCard` (para binarios) y campos normalizados desde `Contact`.
- Si en el futuro se añade otro formato de entrada (LDIF, CSV), solo hay que escribir un nuevo `XxxRecord::into_contact()`.

## Alternativas consideradas

- Modelo único con flags: rechazado por acoplamiento entre reglas de negocio y detalles RFC.
- Contact extiende ParsedVCard: rechazado porque invertiría la dependencia (dominio dependería de infraestructura).

---

# ADR-003: Union-Find para deduplicación transitiva

**Fecha:** 2026-07-07
**Estado:** Aceptado

---

## Contexto

La detección de duplicados D2 puede crear cadenas transitivas: A coincide con B por TEL, A coincide con C por EMAIL, pero B y C no coinciden directamente. Los tres deben fusionarse.

## Decisión

**Usar Union-Find (Disjoint Set Union) para agrupar contactos por componentes conexos, y materializar la fusión usando `Vec<Option<Contact>>` con índices descendentes para evitar invalidaciones.**

## Consecuencias

- DSU construye el grafo de coincidencias en O(n α(n)).
- La materialización descendente evita bugs de índice que ocurrían con `Vec::remove()`.
- `merged_uids` registra todos los UIDs absorbidos sin perder el `action` original del absorbente.
- D3-D6 se registran como propuestas en NOTE pero no disparan fusión automática.

## Alternativas consideradas

- BFS/DFS sobre HashMap de adyacencia: equivalente en complejidad, pero DSU es más conciso y tiene mejor performance para n grande.
- `Vec::remove()` con ajuste de índices: rechazado por el bug detectado en el análisis del plan.

---

# ADR-004: std::sync::LazyLock, no once_cell

**Fecha:** 2026-07-07
**Estado:** Aceptado

---

## Contexto

Se necesita inicialización perezosa de estructuras estáticas (reglas de clasificación, mapa de propiedades binarias). `once_cell` era la solución canónica antes de Rust 1.80.

## Decisión

**Usar `std::sync::LazyLock`, estabilizado en Rust 1.80. No añadir `once_cell` como dependencia.**

## Consecuencias

- Una dependencia menos.
- MSRV = 1.80 (ya establecido).
- `LazyLock` tiene la misma API que `once_cell::sync::Lazy`.

## Alternativas consideradas

- `once_cell`: rechazado por redundancia con std.
- Inicialización eager: rechazado por costo de compilar 20+ regex en startup.

---

# ADR-005: jiff en lugar de chrono

**Fecha:** 2026-07-07
**Estado:** Aceptado

---

## Contexto

Se necesita manejo de timestamps para `DecisionTrace.timestamp` y `AuditEntry.timestamp`. `chrono` tiene problemas de soundness conocidos y mantenimiento irregular.

## Decisión

**Usar `jiff` (v0.1), una crate moderna de manejo de fechas/horas sin problemas de soundness, con API ergonómica inspirada en Temporal.**

## Consecuencias

- `jiff::Timestamp::now()` para timestamps UTC.
- `jiff::Zoned` para formateo en zona horaria local si se necesita.
- Sin unsafe, sin problemas de soundness.

## Alternativas consideradas

- `time` (v0.3): API menos ergonómica, aunque madura.
- `chrono`: rechazado por soundness issues.
