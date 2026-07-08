//! Normalización de campos de contacto.
//!
//! Reglas N1-N7 para FN, T1-T4 para TEL, y normalización de ORG.

use crate::domain::contact::Tel;
use crate::domain::contact::TelType;

/// Resultado de normalizar FN: (fn_canonico, title_extraido, role_extraido)
pub fn normalize_fn(fn_val: &str) -> (String, Option<String>, Option<String>) {
    let cleaned = fn_val.trim();

    // N6: si es email, no tocar (el screening decide)
    if cleaned.contains('@') {
        return (cleaned.to_string(), None, None);
    }

    let mut remaining = cleaned.to_string();

    // N4: extraer títulos del inicio
    let title = extract_title(&remaining);
    if let Some(ref t) = title {
        remaining = remaining[t.len()..].trim().to_string();
    }

    // N5: extraer cargos del final
    let role = extract_role(&remaining);
    if let Some(ref r) = role {
        let pos = remaining.to_lowercase().rfind(&r.to_lowercase()).unwrap();
        remaining = remaining[..pos].trim().to_string();
    }

    // N7: capitalización con respeto de siglas
    remaining = normalize_capitalization(&remaining);

    (remaining.trim().to_string(), title, role)
}

/// N4: detectar y extraer títulos del inicio del FN.
fn extract_title(fn_val: &str) -> Option<String> {
    let titles = [
        ("Ilmo. Sr.", 1),
        ("Ilma. Sra.", 1),
        ("Excmo. Sr.", 1),
        ("Excma. Sra.", 1),
        ("Ilmo.", 1),
        ("Ilma.", 1),
        ("Excmo.", 1),
        ("Excma.", 1),
        ("Dr.", 1),
        ("Dra.", 1),
        ("D.", 1),
        ("Dña.", 1),
        ("Sr.", 1),
        ("Sra.", 1),
        ("Don ", 0),
        ("Doña ", 0),
        ("Sr. D. ", 2),
        ("Sra. Dña. ", 2),
    ];

    let lower = fn_val.to_lowercase();
    for (title_str, spaces) in &titles {
        if lower.starts_with(&title_str.to_lowercase()) {
            // Verificar que hay espacio después o es final de cadena
            let after = &fn_val[title_str.len()..];
            if after.is_empty() || after.starts_with(' ') {
                return Some(title_str.to_string());
            }
            // Casos como "Dr." sin espacio después de "Dra." → solo si hay espacio
            if *spaces > 0 {
                continue;
            }
        }
    }
    None
}

/// N5: detectar y extraer cargos del final del FN.
fn extract_role(fn_val: &str) -> Option<String> {
    let roles = [
        "Juez",
        "Jueza",
        "Fiscal",
        "Letrado",
        "Letrada",
        "Procurador",
        "Procuradora",
        "Secretario",
        "Secretaria",
        "Abogado",
        "Abogada",
        "Notario",
        "Notaria",
    ];

    let lower = fn_val.to_lowercase().trim().to_string();
    for role in &roles {
        let rlower = role.to_lowercase();
        // El cargo debe estar al final o seguido de nada más (puede estar solo)
        if lower.ends_with(&rlower) {
            // Asegurarse de que es una palabra completa (precedida por espacio o inicio)
            let prefix_len = lower.len() - rlower.len();
            if prefix_len == 0 || lower.as_bytes().get(prefix_len - 1) == Some(&b' ') {
                return Some(role.to_string());
            }
        }
    }
    None
}

/// N1+N7: capitalización con respeto de siglas.
///
/// Palabras de 2-5 letras en ALL CAPS se preservan como siglas (TSJ, ICAV, GVA, AEAT).
/// El resto: primera letra mayúscula, resto minúscula.
fn normalize_capitalization(s: &str) -> String {
    let words: Vec<String> = s
        .split_whitespace()
        .map(|w| {
            // Preservar siglas de 2-5 letras mayúsculas
            if w.len() >= 2 && w.len() <= 5 && w.chars().all(|c| c.is_ascii_uppercase()) {
                return w.to_string();
            }
            // Capitalizar: primera letra mayúscula, resto minúscula
            let mut chars: Vec<char> = w.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_uppercase().next().unwrap_or(*first);
            }
            for c in chars.iter_mut().skip(1) {
                *c = c.to_lowercase().next().unwrap_or(*c);
            }
            chars.into_iter().collect()
        })
        .collect();

    words.join(" ")
}

/// T1+T4: normaliza un número de teléfono a E.164.
///
/// Retorna un Tel con `normalized = true` si se pudo normalizar.
pub fn normalize_tel(value: &str, prefijo_pais: &str) -> Tel {
    // Limpiar: eliminar espacios, guiones, puntos, paréntesis
    let cleaned: String = value
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '.' && *c != '(' && *c != ')')
        .collect();

    // Si ya es E.164
    if cleaned.starts_with('+') {
        let digits_only: String = cleaned.chars().filter(|c| c.is_ascii_digit()).collect();
        if !digits_only.is_empty() {
            return Tel {
                value: format!("+{}", digits_only),
                tel_type: TelType::Other,
                normalized: true,
            };
        }
    }

    // Si empieza por 00 → reemplazar por +
    if let Some(rest) = cleaned.strip_prefix("00") {
        let digits: String = rest.chars().filter(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            return Tel {
                value: format!("+{}", digits),
                tel_type: TelType::Other,
                normalized: true,
            };
        }
    }

    // Si no es puramente numérico → no normalizable
    if !cleaned.chars().all(|c| c.is_ascii_digit()) {
        return Tel {
            value: value.to_string(),
            tel_type: TelType::Other,
            normalized: false,
        };
    }

    // Número español sin prefijo (empieza por 6, 7, 8, 9)
    if cleaned.len() == 9
        && cleaned
            .chars()
            .next()
            .map(|c| c == '6' || c == '7' || c == '8' || c == '9')
            .unwrap_or(false)
    {
        let prefix = prefijo_pais.trim_start_matches('+');
        return Tel {
            value: format!("+{}{}", prefix, cleaned),
            tel_type: TelType::Other,
            normalized: true,
        };
    }

    // Número con prefijo nacional (sin +)
    if cleaned.len() > 9 {
        let prefix = prefijo_pais.trim_start_matches('+');
        return Tel {
            value: format!("+{}{}", prefix, cleaned),
            tel_type: TelType::Other,
            normalized: true,
        };
    }

    // Fallback: no se pudo normalizar
    Tel {
        value: value.to_string(),
        tel_type: TelType::Other,
        normalized: false,
    }
}

/// Normaliza ORG: elimina formas jurídicas del final.
///
/// Retorna (org_limpio, forma_juridica).
pub fn normalize_org(org: &str) -> (String, Option<String>) {
    let legal_forms = [
        "S.L.P.", "S.L.U.", "S.A.U.", "S.C.P.", "S.L.", "S.A.", "C.B.", "S.C.",
    ];

    let trimmed = org.trim();
    let lower = trimmed.to_lowercase();

    for form in &legal_forms {
        let flower = form.to_lowercase();
        if lower.ends_with(&flower) {
            let prefix_len = trimmed.len() - form.len();
            // Verificar que está precedido por espacio o coma+espacio
            if prefix_len > 0 {
                let before = &trimmed[..prefix_len];
                if before.ends_with(' ') || before.ends_with(", ") {
                    let org_clean = before.trim_end().trim_end_matches(',').trim().to_string();
                    let form_clean = form.trim_end_matches('.').to_string();
                    return (org_clean, Some(form_clean));
                }
            }
        }
    }

    (trimmed.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── normalize_fn ──

    #[test]
    fn test_normalize_fn_removes_title_ilmo_sr() {
        let (fn_val, title, role) = normalize_fn("Ilmo. Sr. Juan Pérez");
        assert_eq!(fn_val, "Juan Pérez");
        assert_eq!(title, Some("Ilmo. Sr.".into()));
        assert_eq!(role, None);
    }

    #[test]
    fn test_normalize_fn_removes_title_dr() {
        let (fn_val, title, _) = normalize_fn("Dr. García López");
        assert_eq!(fn_val, "García López");
        assert_eq!(title, Some("Dr.".into()));
    }

    #[test]
    fn test_normalize_fn_removes_role_juez() {
        let (fn_val, title, role) = normalize_fn("Carlos Ruiz Juez");
        assert_eq!(fn_val, "Carlos Ruiz");
        assert_eq!(title, None);
        assert_eq!(role, Some("Juez".into()));
    }

    #[test]
    fn test_normalize_fn_capitalization() {
        let (fn_val, _, _) = normalize_fn("juZGADO INSTRUCCIÓN 9");
        assert_eq!(fn_val, "Juzgado Instrucción 9");
    }

    #[test]
    fn test_normalize_fn_respects_acronyms() {
        let (fn_val, _, _) = normalize_fn("ICAV turno oficio");
        assert_eq!(fn_val, "ICAV Turno Oficio");
    }

    #[test]
    fn test_normalize_fn_respects_acronyms_gva() {
        let (fn_val, _, _) = normalize_fn("conselleria GVA innovación");
        assert_eq!(fn_val, "Conselleria GVA Innovación");
    }

    #[test]
    fn test_normalize_fn_email_unchanged() {
        let (fn_val, title, role) = normalize_fn("info@procuradores.es");
        assert_eq!(fn_val, "info@procuradores.es");
        assert_eq!(title, None);
        assert_eq!(role, None);
    }

    #[test]
    fn test_normalize_fn_combined_title_and_role() {
        let (fn_val, title, role) = normalize_fn("Ilmo. Sr. Carlos Ruiz Juez");
        assert_eq!(fn_val, "Carlos Ruiz");
        assert_eq!(title, Some("Ilmo. Sr.".into()));
        assert_eq!(role, Some("Juez".into()));
    }

    // ── normalize_tel ──

    #[test]
    fn test_normalize_tel_e164_spanish() {
        let tel = normalize_tel("612345678", "+34");
        assert_eq!(tel.value, "+34612345678");
        assert!(tel.normalized);
    }

    #[test]
    fn test_normalize_tel_already_e164() {
        let tel = normalize_tel("+34612345678", "+34");
        assert_eq!(tel.value, "+34612345678");
        assert!(tel.normalized);
    }

    #[test]
    fn test_normalize_tel_with_spaces_and_dashes() {
        let tel = normalize_tel("+34 612-345-678", "+34");
        assert_eq!(tel.value, "+34612345678");
        assert!(tel.normalized);
    }

    #[test]
    fn test_normalize_tel_double_zero_prefix() {
        let tel = normalize_tel("0034612345678", "+34");
        assert_eq!(tel.value, "+34612345678");
        assert!(tel.normalized);
    }

    #[test]
    fn test_normalize_tel_non_numeric() {
        let tel = normalize_tel("AEAT", "+34");
        assert_eq!(tel.value, "AEAT");
        assert!(!tel.normalized);
    }

    #[test]
    fn test_normalize_tel_uk_number() {
        let tel = normalize_tel("+447911123456", "+34");
        assert_eq!(tel.value, "+447911123456");
        assert!(tel.normalized);
    }

    // ── normalize_org ──

    #[test]
    fn test_normalize_org_sl() {
        let (org, form) = normalize_org("Despacho Legal S.L.");
        assert_eq!(org, "Despacho Legal");
        assert_eq!(form.as_deref(), Some("S.L"));
    }

    #[test]
    fn test_normalize_org_slp() {
        let (org, form) = normalize_org("Gráficas Nasve, S.L.P.");
        assert_eq!(org, "Gráficas Nasve");
        assert_eq!(form.as_deref(), Some("S.L.P"));
    }

    #[test]
    fn test_normalize_org_no_legal_form() {
        let (org, form) = normalize_org("Juzgado Instrucción 9");
        assert_eq!(org, "Juzgado Instrucción 9");
        assert_eq!(form, None);
    }

    #[test]
    fn test_normalize_org_partial_match_avoided() {
        // "S.L." dentro de una palabra no debe detectarse como forma jurídica
        let (org, form) = normalize_org("S.Lorenzo Consulting");
        assert_eq!(org, "S.Lorenzo Consulting");
        assert_eq!(form, None);
    }
}
