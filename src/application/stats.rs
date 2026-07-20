//! Caso de uso: ComputeStats — estadísticas post-cribado.

use std::collections::HashMap;
use std::fmt;

use crate::domain::contact::{Contact, SourceDetail};
use crate::domain::screening::ScreeningDecision;

/// Estadísticas del pipeline de cribado.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Stats {
    pub total_entrada: usize,
    pub conservados: usize,
    pub eliminados: usize,
    pub fusionados: usize,
    pub cuarentena: usize,
    pub needs_review: usize,
    pub por_categoria: HashMap<String, usize>,
    pub por_origen: HashMap<String, usize>,
}

impl Stats {
    /// Calcula estadísticas a partir de conteos precalculados.
    pub fn compute(
        contacts: &[Contact],
        total_entrada: usize,
        conservados: usize,
        eliminados: usize,
        fusionados: usize,
        cuarentena: usize,
        needs_review: usize,
    ) -> Self {
        let mut por_categoria = HashMap::new();
        let mut por_origen = HashMap::new();

        for c in contacts {
            if matches!(
                &c.decision,
                ScreeningDecision::Conserved | ScreeningDecision::NeedsReview(_)
            ) {
                for cat in &c.categories.n2 {
                    *por_categoria.entry(cat.clone()).or_insert(0) += 1;
                }
                let origen = source_to_str(&c.source_detail);
                *por_origen.entry(origen).or_insert(0) += 1;
            }
        }

        Self {
            total_entrada,
            conservados,
            eliminados,
            fusionados,
            cuarentena,
            needs_review,
            por_categoria,
            por_origen,
        }
    }

    /// Formato texto legible.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str("=== Estadísticas de cribado ===\n");
        out.push_str(&format!("Total entrada:   {}\n", self.total_entrada));
        out.push_str(&format!("  Conservados:   {}\n", self.conservados));
        out.push_str(&format!("  Eliminados:    {}\n", self.eliminados));
        out.push_str(&format!("  Fusionados:    {}\n", self.fusionados));
        out.push_str(&format!("  Cuarentena:    {}\n", self.cuarentena));
        out.push_str(&format!("  Needs Review:  {}\n", self.needs_review));

        if !self.por_categoria.is_empty() {
            out.push_str("\nPor categoría:\n");
            let mut sorted: Vec<_> = self.por_categoria.iter().collect();
            sorted.sort_by_key(|(k, _)| *k);
            for (cat, count) in &sorted {
                out.push_str(&format!("  {}:  {}\n", cat, count));
            }
        }

        if !self.por_origen.is_empty() {
            out.push_str("\nPor origen:\n");
            let mut sorted: Vec<_> = self.por_origen.iter().collect();
            sorted.sort_by_key(|(k, _)| *k);
            for (origen, count) in &sorted {
                out.push_str(&format!("  {}:  {}\n", origen, count));
            }
        }

        out
    }

    /// Formato JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Formato Markdown (tabla).
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# Estadísticas de cribado\n\n");
        out.push_str("| Métrica | Valor |\n");
        out.push_str("|---------|-------|\n");
        out.push_str(&format!("| Total entrada | {} |\n", self.total_entrada));
        out.push_str(&format!("| Conservados | {} |\n", self.conservados));
        out.push_str(&format!("| Eliminados | {} |\n", self.eliminados));
        out.push_str(&format!("| Fusionados | {} |\n", self.fusionados));
        out.push_str(&format!("| Cuarentena | {} |\n", self.cuarentena));
        out.push_str(&format!("| Needs Review | {} |\n", self.needs_review));

        if !self.por_categoria.is_empty() {
            out.push_str("\n## Por categoría\n\n");
            out.push_str("| Categoría | Cantidad |\n");
            out.push_str("|-----------|----------|\n");
            let mut sorted: Vec<_> = self.por_categoria.iter().collect();
            sorted.sort_by_key(|(k, _)| *k);
            for (cat, count) in &sorted {
                out.push_str(&format!("| {} | {} |\n", cat, count));
            }
        }

        out
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_text())
    }
}

fn source_to_str(sd: &SourceDetail) -> String {
    match sd {
        SourceDetail::ProtonAutosave => "proton-autosave".into(),
        SourceDetail::ProtonImport => "proton-import".into(),
        SourceDetail::ProtonWeb => "proton-web".into(),
        SourceDetail::Google => "google".into(),
        SourceDetail::Apple => "apple".into(),
        SourceDetail::Unknown(s) if s.is_empty() => "unknown".into(),
        SourceDetail::Unknown(s) => s.clone(),
    }
}

/// Ejecuta el comando stats: pipeline screening + estadísticas.
pub fn execute(input: &std::path::Path, format: &str) -> Result<(), crate::error::CribaError> {
    let bytes = std::fs::read(input)?;
    if bytes.is_empty() {
        return Err(crate::error::CribaError::EmptyVcf);
    }

    let utf8_text = crate::infrastructure::encoding::ensure_utf8(&bytes)?;
    let unfolded = crate::infrastructure::parser::unfold(&utf8_text);
    let vcards = crate::infrastructure::parser::parse_vcards(&unfolded)?;
    let total = vcards.len();

    let config = crate::domain::screening::ScreeningConfig::default();

    let mut contacts = Vec::with_capacity(vcards.len());
    for vcard in &vcards {
        let contact = vcard.to_contact()?;
        let trace = crate::domain::screening::decide(&contact, &config);
        let mut c = contact;
        c.decision = trace.outcome;
        contacts.push(c);
    }

    let stats = Stats::compute(&contacts, total, 0, 0, 0, 0, 0);

    match format {
        "json" => println!("{}", stats.to_json()?),
        "markdown" => println!("{}", stats.to_markdown()),
        _ => println!("{}", stats),
    }

    Ok(())
}

// ── tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contact::{CategorySet, Contact, SourceDetail};
    use crate::domain::screening::ScreeningDecision;

    fn make_contact(uid: &str, decision: ScreeningDecision) -> Contact {
        Contact {
            uid: uid.into(),
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
            source_detail: SourceDetail::Unknown(String::new()),
            decision,
            screening_rule: String::new(),
            merged_uids: vec![],
        }
    }

    #[test]
    fn test_stats_compute_counts() {
        let contacts = vec![
            make_contact("a", ScreeningDecision::Conserved),
            make_contact("b", ScreeningDecision::Conserved),
            make_contact(
                "c",
                ScreeningDecision::Eliminated(crate::domain::screening::ElimCode::E1),
            ),
            make_contact(
                "d",
                ScreeningDecision::NeedsReview(
                    crate::domain::screening::ReviewReason::E2InappropriateMetadata,
                ),
            ),
        ];
        let stats = Stats::compute(&contacts, 4, 2, 1, 0, 0, 1);
        assert_eq!(stats.total_entrada, 4);
        assert_eq!(stats.conservados, 2);
        assert_eq!(stats.eliminados, 1);
        assert_eq!(stats.needs_review, 1);
        assert_eq!(stats.fusionados, 0);
        assert_eq!(stats.cuarentena, 0);
    }

    #[test]
    fn test_stats_compute_categories() {
        let mut c1 = make_contact("a", ScreeningDecision::Conserved);
        c1.categories.n2.insert("PROF-JUD".into());
        let mut c2 = make_contact("b", ScreeningDecision::Conserved);
        c2.categories.n2.insert("PROF-JUD".into());
        c2.categories.n2.insert("INST-AUT".into());

        let stats = Stats::compute(&[c1, c2], 2, 0, 0, 0, 0, 0);
        assert_eq!(stats.por_categoria.get("PROF-JUD"), Some(&2));
        assert_eq!(stats.por_categoria.get("INST-AUT"), Some(&1));
    }

    #[test]
    fn test_stats_display_text() {
        let contacts = vec![make_contact("a", ScreeningDecision::Conserved)];
        let stats = Stats::compute(&contacts, 1, 1, 0, 0, 0, 0);
        let text = stats.to_text();
        assert!(text.contains("Estadísticas de cribado"));
        assert!(text.contains("Total entrada:   1"));
        assert!(text.contains("Conservados:   1"));
    }

    #[test]
    fn test_stats_display_json() {
        let contacts = vec![make_contact("a", ScreeningDecision::Conserved)];
        let stats = Stats::compute(&contacts, 1, 1, 0, 0, 0, 0);
        let json = stats.to_json().unwrap();
        assert!(json.contains("\"total_entrada\""));
        assert!(json.contains("\"conservados\""));
    }

    #[test]
    fn test_stats_display_markdown() {
        let contacts = vec![make_contact("a", ScreeningDecision::Conserved)];
        let stats = Stats::compute(&contacts, 1, 1, 0, 0, 0, 0);
        let md = stats.to_markdown();
        assert!(md.contains("# Estadísticas de cribado"));
        assert!(md.contains("| Total entrada | 1 |"));
    }
}
