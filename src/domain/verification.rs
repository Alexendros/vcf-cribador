//! Verificación de invariantes del dominio.

use std::path::Path;

use crate::domain::contact::Contact;
use crate::domain::screening::ScreeningDecision;

/// Verifica las invariantes de dominio sobre la lista de contactos procesados.
///
/// Retorna una lista de advertencias (no falla) para que el pipeline
/// decida si continuar o abortar. Esto mantiene el flujo robusto ante
/// inputs inesperados sin perder datos.
pub fn verify(contacts: &[Contact], input: Option<&Path>, output: Option<&Path>) -> Vec<String> {
    let mut warnings = Vec::new();

    for contact in contacts {
        // I1 — Integridad de contactos conservados
        if contact.decision == ScreeningDecision::Conserved {
            if contact.uid.trim().is_empty() {
                warnings.push(format!("I1: contacto sin UID (FN='{}')", contact.fn_value));
            }
            if !contact.categories.has_n1() {
                warnings.push(format!(
                    "I1: contacto conservado sin categoría N1 (uid={})",
                    contact.uid
                ));
            }
            if matches!(contact.source_detail, crate::domain::contact::SourceDetail::Unknown(ref s) if s.is_empty())
            {
                warnings.push(format!(
                    "I1: contacto conservado sin source_detail (uid={})",
                    contact.uid
                ));
            }
        }

        // I2 — FN canónico (solo para contactos que se conservan/revisan)
        if matches!(
            contact.decision,
            ScreeningDecision::Conserved | ScreeningDecision::NeedsReview(_)
        ) && contact.fn_value.contains('@')
        {
            warnings.push(format!(
                "I2: FN contiene '@' en contacto uid={}",
                contact.uid
            ));
        }

        // I3 — TEL E.164 o marcado como no normalizable
        if contact.decision == ScreeningDecision::Conserved {
            for tel in &contact.tels {
                if tel.normalized && !tel.value.starts_with('+') {
                    warnings.push(format!(
                        "I3: TEL normalizado sin prefijo '+' (uid={})",
                        contact.uid
                    ));
                }
            }
        }
    }

    // I7 — No destrucción: la salida no debe coincidir con la entrada
    if let (Some(input), Some(output)) = (input, output) {
        if input == output {
            warnings.push("I7: la ruta de salida coincide con la de entrada".into());
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contact::{CategorySet, Contact, SourceDetail};
    use crate::domain::screening::ScreeningDecision;

    fn make_contact(decision: ScreeningDecision) -> Contact {
        Contact {
            uid: "u1".into(),
            fn_value: "Test".into(),
            structured_name: None,
            org: None,
            org_fullname: None,
            org_legal_form: None,
            emails: vec![],
            tels: vec![],
            title: None,
            role: None,
            note: None,
            addresses: vec![],
            categories: CategorySet::default(),
            source_detail: SourceDetail::Unknown("test".into()),
            decision,
            screening_rule: String::new(),
            merged_uids: vec![],
        }
    }

    #[test]
    fn test_verify_valid_conserved() {
        let mut c = make_contact(ScreeningDecision::Conserved);
        c.categories.n1.insert("PERS".into());
        let warnings = verify(&[c], None, None);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_verify_missing_n1() {
        let c = make_contact(ScreeningDecision::Conserved);
        let warnings = verify(&[c], None, None);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("I1"));
    }

    #[test]
    fn test_verify_fn_with_at() {
        let mut c = make_contact(ScreeningDecision::Conserved);
        c.fn_value = "info@test.com".into();
        c.categories.n1.insert("PERS".into());
        let warnings = verify(&[c], None, None);
        assert!(warnings.iter().any(|w| w.contains("I2")));
    }

    #[test]
    fn test_verify_input_equals_output() {
        let c = make_contact(ScreeningDecision::Eliminated(
            crate::domain::screening::ElimCode::E1,
        ));
        let path = Path::new("/tmp/test.vcf");
        let warnings = verify(&[c], Some(path), Some(path));
        assert!(warnings.iter().any(|w| w.contains("I7")));
    }
}
