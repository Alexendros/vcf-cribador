//! Clasificación automática de contactos.

use crate::domain::contact::{CategorySet, Contact};
use crate::domain::rules::ClassificationRule;

/// Asigna categorías N1/N2/N3 a un contacto.
///
/// Busca coincidencias de los regex en FN, ORG y EMAIL del contacto.
/// Acumula todas las reglas que disparan. Si ninguna dispara,
/// asigna una categoría N1 por defecto basada en heurísticas simples.
pub fn classify(contact: &Contact, rules: &[ClassificationRule]) -> CategorySet {
    let mut n1 = std::collections::HashSet::new();
    let mut n2 = std::collections::HashSet::new();
    let mut n3 = Vec::new();

    let haystack = build_haystack(contact);

    for rule in rules {
        if rule.pattern.is_match(&haystack) {
            n1.insert(rule.n1.clone());
            n2.insert(rule.n2.clone());
            n3.push(rule.n3.clone());
        }
    }

    if n1.is_empty() {
        n1.insert(default_n1(contact).to_string());
    }

    CategorySet { n1, n2, n3 }
}

fn build_haystack(contact: &Contact) -> String {
    let mut parts = vec![contact.fn_value.as_str()];
    if let Some(ref org) = contact.org {
        parts.push(org);
    }
    for email in &contact.emails {
        parts.push(&email.value);
    }
    parts.join(" ")
}

fn default_n1(contact: &Contact) -> &'static str {
    if !contact.tels.is_empty() {
        "PERS"
    } else {
        "TEC"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contact::{SourceDetail, TypedValue};
    use crate::domain::rules::CLASSIFICATION_RULES;
    use crate::domain::screening::ScreeningDecision;

    fn make_contact(fn_val: &str, org: Option<&str>, email: &str) -> Contact {
        Contact {
            uid: "test-uid".into(),
            fn_value: fn_val.into(),
            org: org.map(|s| s.to_string()),
            emails: vec![TypedValue {
                value: email.into(),
                types: vec![],
                pref: 1,
            }],
            tels: vec![],
            structured_name: None,
            org_fullname: None,
            org_legal_form: None,
            title: None,
            role: None,
            note: None,
            addresses: vec![],
            categories: CategorySet::default(),
            source_detail: SourceDetail::Unknown("test".into()),
            decision: ScreeningDecision::Conserved,
            screening_rule: String::new(),
            merged_uids: vec![],
        }
    }

    #[test]
    fn test_juzgado_matched() {
        let c = make_contact(
            "Juzgado Instrucción 9",
            Some("Juzgado Instrucción 9"),
            "test@gva.es",
        );
        let cats = classify(&c, &CLASSIFICATION_RULES);
        assert!(cats.n2.contains("PROF-JUD"));
    }

    #[test]
    fn test_gva_domain_matched() {
        let c = make_contact("Test", Some("Test"), "test@gva.es");
        let cats = classify(&c, &CLASSIFICATION_RULES);
        assert!(cats.n2.contains("INST-AUT"));
    }

    #[test]
    fn test_crypto_matched() {
        let c = make_contact("Bybit Support", Some("Bybit"), "support@bybit.com");
        let cats = classify(&c, &CLASSIFICATION_RULES);
        assert!(cats.n2.contains("FIN-CRYPTO"));
    }

    #[test]
    fn test_multiple_categories() {
        let c = make_contact("Juzgado 9", Some("Juzgado Instrucción 9"), "test@gva.es");
        let cats = classify(&c, &CLASSIFICATION_RULES);
        assert!(cats.n2.contains("PROF-JUD"));
        assert!(cats.n2.contains("INST-AUT"));
    }

    #[test]
    fn test_no_match_defaults() {
        let c = make_contact("Unknown Person", None, "x@example.com");
        let cats = classify(&c, &CLASSIFICATION_RULES);
        assert!(cats.has_n1());
    }

    #[test]
    fn test_n3_populated() {
        let c = make_contact(
            "Juzgado Instrucción 9",
            Some("Juzgado Instrucción 9"),
            "test@example.com",
        );
        let cats = classify(&c, &CLASSIFICATION_RULES);
        assert!(cats.n3.iter().any(|n| n == "JUD-JUZ"));
    }
}
