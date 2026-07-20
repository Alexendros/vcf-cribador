//! Modelo de auditoría — traza inmutable de cada contacto procesado.

use crate::domain::contact::{CategorySet, Contact};
use crate::domain::screening::{DecisionTrace, ScreeningDecision};

/// Entrada de auditoría para un contacto procesado.
///
/// Captura el estado original y final del contacto, la decisión de cribado,
/// y metadatos de trazabilidad. Es inmutable y se construye una vez
/// finalizado el procesamiento del contacto.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: String,
    pub uid: String,
    pub fn_original: String,
    pub fn_final: String,
    pub action: AuditAction,
    pub reason: String,
    pub rule: String,
    pub categories: CategorySet,
    pub source: String,
    pub tels: Vec<String>,
    pub emails: Vec<String>,
    pub merged_into: Option<String>,
}

/// Acción determinada por el pipeline para un contacto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditAction {
    Conserved,
    Eliminated,
    Quarantine,
    NeedsReview,
    Merged,
}

impl AuditEntry {
    /// Construye una entrada de auditoría a partir del contacto final procesado,
    /// el nombre original y la traza de decisión.
    pub fn from_contact(contact: &Contact, fn_original: &str, trace: &DecisionTrace) -> Self {
        Self {
            timestamp: jiff::Timestamp::now()
                .strftime("%Y-%m-%dT%H:%M:%SZ")
                .to_string(),
            uid: contact.uid.clone(),
            fn_original: fn_original.to_string(),
            fn_final: contact.fn_value.clone(),
            action: AuditAction::from(&trace.outcome),
            reason: trace.evidence.clone(),
            rule: trace.triggered_rule.clone(),
            categories: contact.categories.clone(),
            source: source_to_string(&contact.source_detail),
            tels: contact.tels.iter().map(|t| t.value.clone()).collect(),
            emails: contact.emails.iter().map(|e| e.value.clone()).collect(),
            merged_into: None,
        }
    }

    /// Marca la entrada como fusionada en otro contacto.
    pub fn merged_into(mut self, uid: &str) -> Self {
        self.merged_into = Some(uid.into());
        self.action = AuditAction::Merged;
        self.reason = format!("Fusionado en {}", uid);
        self
    }
}

impl From<&ScreeningDecision> for AuditAction {
    fn from(decision: &ScreeningDecision) -> Self {
        match decision {
            ScreeningDecision::Conserved => AuditAction::Conserved,
            ScreeningDecision::Eliminated(_) => AuditAction::Eliminated,
            ScreeningDecision::Quarantine(_) => AuditAction::Quarantine,
            ScreeningDecision::NeedsReview(_) => AuditAction::NeedsReview,
        }
    }
}

fn source_to_string(source: &crate::domain::contact::SourceDetail) -> String {
    use crate::domain::contact::SourceDetail;
    match source {
        SourceDetail::ProtonAutosave => "proton-autosave".into(),
        SourceDetail::ProtonImport => "proton-import".into(),
        SourceDetail::ProtonWeb => "proton-web".into(),
        SourceDetail::Google => "google".into(),
        SourceDetail::Apple => "apple".into(),
        SourceDetail::Unknown(s) if s.is_empty() => "unknown".into(),
        SourceDetail::Unknown(s) => s.clone(),
    }
}

/// Formatea un conjunto de categorías para el TSV.
pub fn format_categories_tsv(cats: &CategorySet) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for c in &cats.n1 {
        parts.push(c.as_str());
    }
    for c in &cats.n2 {
        parts.push(c.as_str());
    }
    parts.join(",")
}
