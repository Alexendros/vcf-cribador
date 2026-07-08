//! Decisión de cribado y traza determinista.

use jiff::Timestamp;

use crate::domain::contact::Contact;

/// Resultado del cribado para un contacto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreeningDecision {
    Conserved,
    Eliminated(ElimCode),
    NeedsReview(ReviewReason),
    Quarantine(ElimCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElimCode {
    E1,
    E3,
    E4,
    E6,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewReason {
    E2InappropriateMetadata,
    D3DuplicateCandidate,
    D6FuzzyDuplicate,
}

/// Configuración del cribado.
#[derive(Debug, Clone)]
pub struct ScreeningConfig {
    pub conservar_dominios: Vec<String>,
    pub e2_keywords: Vec<String>,
    pub prefijo_pais: String,
}

impl Default for ScreeningConfig {
    fn default() -> Self {
        Self {
            conservar_dominios: vec![],
            e2_keywords: vec![
                "pervert".into(),
                "dealer".into(),
                "prostit".into(),
                "droga".into(),
                "coca".into(),
                "narco".into(),
                "puta".into(),
                "whore".into(),
            ],
            prefijo_pais: "+34".into(),
        }
    }
}

/// Traza inmutable de una decisión de cribado.
#[derive(Debug, Clone)]
pub struct DecisionTrace {
    pub outcome: ScreeningDecision,
    pub triggered_rule: String,
    pub evidence: String,
    pub timestamp: Timestamp,
}

impl DecisionTrace {
    pub fn conserved(rule: &str, evidence: &str) -> Self {
        Self {
            outcome: ScreeningDecision::Conserved,
            triggered_rule: rule.to_string(),
            evidence: evidence.to_string(),
            timestamp: Timestamp::now(),
        }
    }

    pub fn eliminated(code: ElimCode, evidence: &str) -> Self {
        Self {
            outcome: ScreeningDecision::Eliminated(code),
            triggered_rule: format!("{:?}", code),
            evidence: evidence.to_string(),
            timestamp: Timestamp::now(),
        }
    }

    pub fn needs_review(reason: ReviewReason, rule: &str, evidence: &str) -> Self {
        Self {
            outcome: ScreeningDecision::NeedsReview(reason),
            triggered_rule: rule.to_string(),
            evidence: evidence.to_string(),
            timestamp: Timestamp::now(),
        }
    }

    pub fn quarantine(code: ElimCode, evidence: &str) -> Self {
        Self {
            outcome: ScreeningDecision::Quarantine(code),
            triggered_rule: format!("{:?}", code),
            evidence: evidence.to_string(),
            timestamp: Timestamp::now(),
        }
    }
}

/// Evalúa las reglas de cribado en orden determinista.
///
/// Precedencia: C1-C7 → E2 → E1 → E3 → E4 → E6 → Default(Conserved)
pub fn decide(contact: &Contact, config: &ScreeningConfig) -> DecisionTrace {
    // ── C1-C7: conservación obligatoria ──
    if let Some(trace) = check_c2(contact) {
        return trace;
    }
    if let Some(trace) = check_c3(contact) {
        return trace;
    }
    if let Some(trace) = check_c4(contact) {
        return trace;
    }
    if let Some(trace) = check_c6(contact) {
        return trace;
    }

    // ── E2: metadatos inapropiados → NeedsReview (se limpian, se conserva) ──
    if let Some(trace) = check_e2(contact, &config.e2_keywords) {
        return trace;
    }

    // ── E1: email-only sin identidad ──
    if is_email_only(contact) {
        return DecisionTrace::eliminated(ElimCode::E1, "FN es email y sin ORG/TEL");
    }

    // ── E3: huérfano ──
    if is_huerfano(contact) {
        return DecisionTrace::eliminated(ElimCode::E3, "Sin EMAIL ni TEL");
    }

    // ── Default ──
    DecisionTrace::conserved("Default", "Sin regla específica aplicable")
}

// ── Reglas C ──

/// C2: ORG contiene "Juzgado", "Fiscalía", "TSJ", "GVA", "Generalitat"
fn check_c2(contact: &Contact) -> Option<DecisionTrace> {
    let org = contact.org.as_deref().unwrap_or("");
    let org_lower = org.to_lowercase();

    let keywords = [
        "juzgado",
        "fiscalía",
        "fiscalia",
        "tsj",
        "gva",
        "generalitat",
    ];
    for kw in &keywords {
        if org_lower.contains(kw) {
            return Some(DecisionTrace::conserved(
                "C2-Juzgado",
                &format!("ORG contiene '{}'", kw),
            ));
        }
    }
    None
}

/// C3: ORG contiene "ICAV", "ICAB" o colegio profesional
fn check_c3(contact: &Contact) -> Option<DecisionTrace> {
    let org = contact.org.as_deref().unwrap_or("");
    let org_lower = org.to_lowercase();

    let keywords = [
        "icav",
        "icab",
        "colegio de abogados",
        "colegio de procuradores",
    ];
    for kw in &keywords {
        if org_lower.contains(kw) {
            return Some(DecisionTrace::conserved(
                "C3-Colegio",
                &format!("ORG contiene '{}'", kw),
            ));
        }
    }
    None
}

/// C4: Entidad financiera con relación vigente
fn check_c4(contact: &Contact) -> Option<DecisionTrace> {
    let org = contact.org.as_deref().unwrap_or("");
    let org_lower = org.to_lowercase();

    let keywords = [
        "banco",
        "caja",
        "caixa",
        "bbva",
        "santander",
        "sabadell",
        "bankinter",
        "ing ",
        "revolut",
        "cofidis",
        "evobanco",
        "openbank",
    ];
    for kw in &keywords {
        if org_lower.contains(kw) {
            return Some(DecisionTrace::conserved(
                "C4-Financiera",
                &format!("ORG contiene '{}'", kw),
            ));
        }
    }

    // También buscar en dominios de email
    let financial_domains = ["@bbva", "@caixa", "@santander", "@revolut", "@cofidis"];
    for email in &contact.emails {
        let lower = email.value.to_lowercase();
        for domain in &financial_domains {
            if lower.contains(domain) {
                return Some(DecisionTrace::conserved(
                    "C4-Financiera",
                    &format!("EMAIL contiene '{}'", domain),
                ));
            }
        }
    }
    None
}

/// C6: Personal con TEL presente
fn check_c6(contact: &Contact) -> Option<DecisionTrace> {
    if !contact.tels.is_empty() {
        Some(DecisionTrace::conserved("C6-Personal", "TEL presente"))
    } else {
        None
    }
}

// ── Reglas E ──

/// E2: ROLE/NOTE con contenido inapropiado → NeedsReview
fn check_e2(contact: &Contact, keywords: &[String]) -> Option<DecisionTrace> {
    let haystack = [
        contact.role.as_deref().unwrap_or(""),
        contact.note.as_deref().unwrap_or(""),
    ]
    .join(" ")
    .to_lowercase();

    for kw in keywords {
        if haystack.contains(&kw.to_lowercase()) {
            return Some(DecisionTrace::needs_review(
                ReviewReason::E2InappropriateMetadata,
                "E2-Metadata",
                &format!("contenido inapropiado: '{}'", kw),
            ));
        }
    }
    None
}

/// E1: FN es email sin ORG ni TEL
fn is_email_only(contact: &Contact) -> bool {
    contact.fn_value.contains('@') && contact.org.is_none() && contact.tels.is_empty()
}

/// E3: Sin EMAIL ni TEL
fn is_huerfano(contact: &Contact) -> bool {
    contact.emails.is_empty() && contact.tels.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contact::{CategorySet, SourceDetail, Tel, TelType, TypedValue};

    fn make_contact(
        fn_val: &str,
        org: Option<&str>,
        emails: Vec<&str>,
        tels: Vec<&str>,
    ) -> Contact {
        Contact {
            uid: "test-uid".into(),
            fn_value: fn_val.into(),
            org: org.map(|s| s.to_string()),
            emails: emails
                .into_iter()
                .map(|e| TypedValue {
                    value: e.into(),
                    types: vec![],
                    pref: 1,
                })
                .collect(),
            tels: tels
                .into_iter()
                .map(|t| Tel {
                    value: t.into(),
                    tel_type: TelType::Cell,
                    normalized: true,
                })
                .collect(),
            structured_name: None,
            org_fullname: None,
            org_legal_form: None,
            title: None,
            role: None,
            note: None,
            categories: CategorySet::default(),
            source_detail: SourceDetail::Unknown("test".into()),
            decision: ScreeningDecision::Conserved,
            screening_rule: String::new(),
            merged_uids: vec![],
        }
    }

    #[test]
    fn test_c2_juzgado_overrides_e1() {
        let c = make_contact(
            "info@unknown.com",
            Some("Juzgado Instrucción 9"),
            vec![],
            vec![],
        );
        let trace = decide(&c, &ScreeningConfig::default());
        assert_eq!(trace.outcome, ScreeningDecision::Conserved);
        assert!(trace.triggered_rule.contains("C2"));
    }

    #[test]
    fn test_c2_fiscalia() {
        let c = make_contact("Fiscalía TSJ", Some("Fiscalía de Valencia"), vec![], vec![]);
        let trace = decide(&c, &ScreeningConfig::default());
        assert_eq!(trace.outcome, ScreeningDecision::Conserved);
        assert!(trace.triggered_rule.contains("C2"));
    }

    #[test]
    fn test_c3_icav() {
        let c = make_contact("ICAV Abogados", Some("ICAV Turno Oficio"), vec![], vec![]);
        let trace = decide(&c, &ScreeningConfig::default());
        assert_eq!(trace.outcome, ScreeningDecision::Conserved);
        assert!(trace.triggered_rule.contains("C3"));
    }

    #[test]
    fn test_c6_personal_with_tel() {
        let c = make_contact("Juan Pérez", None, vec![], vec!["+34600000001"]);
        let trace = decide(&c, &ScreeningConfig::default());
        assert_eq!(trace.outcome, ScreeningDecision::Conserved);
        assert!(trace.triggered_rule.contains("C6"));
    }

    #[test]
    fn test_e1_email_only_no_rescue() {
        let c = make_contact("info@unknown.com", None, vec![], vec![]);
        let trace = decide(&c, &ScreeningConfig::default());
        assert_eq!(trace.outcome, ScreeningDecision::Eliminated(ElimCode::E1));
    }

    #[test]
    fn test_e3_huerfano() {
        let c = make_contact("Sin Datos", None, vec![], vec![]);
        let trace = decide(&c, &ScreeningConfig::default());
        assert_eq!(trace.outcome, ScreeningDecision::Eliminated(ElimCode::E3));
    }

    #[test]
    fn test_e2_sanitize_but_preserve() {
        let mut c = make_contact("Contacto X", None, vec!["x@test.com"], vec![]);
        c.role = Some("perverted dealer".into());
        let trace = decide(&c, &ScreeningConfig::default());
        assert!(matches!(
            trace.outcome,
            ScreeningDecision::NeedsReview(ReviewReason::E2InappropriateMetadata)
        ));
    }

    #[test]
    fn test_default_conserved() {
        let c = make_contact("Oficina", None, vec!["office@example.com"], vec![]);
        let trace = decide(&c, &ScreeningConfig::default());
        assert_eq!(trace.outcome, ScreeningDecision::Conserved);
        assert!(trace.triggered_rule.contains("Default"));
    }

    #[test]
    fn test_precedence_c2_before_e1() {
        // ORG contiene "Juzgado" → C2 prevalece aunque FN sea email
        let c = make_contact("info@test.com", Some("Juzgado 1"), vec![], vec![]);
        let trace = decide(&c, &ScreeningConfig::default());
        assert_eq!(trace.outcome, ScreeningDecision::Conserved);
        assert!(trace.triggered_rule.contains("C2"));
    }

    #[test]
    fn test_e2_with_juzgado_still_c2() {
        // ORG juzgado prevalece sobre role inapropiado
        let mut c = make_contact("Test", Some("Juzgado 1"), vec!["x@test.com"], vec![]);
        c.role = Some("perverted".into());
        let trace = decide(&c, &ScreeningConfig::default());
        assert_eq!(trace.outcome, ScreeningDecision::Conserved);
        assert!(trace.triggered_rule.contains("C2"));
    }
}
