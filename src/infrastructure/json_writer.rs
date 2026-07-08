//! Export JSON.

use std::fs;
use std::path::Path;

use crate::domain::contact::Contact;
use crate::domain::screening::ScreeningDecision;
use crate::error::CribaError;
use serde::Serialize;

/// DTO para exportación JSON de un contacto.
#[derive(Serialize)]
struct ContactExport<'a> {
    uid: &'a str,
    fn_value: &'a str,
    n: String,
    org: Option<&'a str>,
    tels: Vec<String>,
    emails: Vec<String>,
    title: Option<&'a str>,
    role: Option<&'a str>,
    categories: Vec<String>,
    source: String,
    cribado_result: &'a str,
    screening_rule: &'a str,
}

/// Exporta contactos conservados y needs_review a JSON.
pub fn export_json(contacts: &[Contact], path: &Path) -> Result<(), CribaError> {
    let exports: Vec<ContactExport> = contacts
        .iter()
        .filter(|c| {
            matches!(
                &c.decision,
                ScreeningDecision::Conserved | ScreeningDecision::NeedsReview(_)
            )
        })
        .map(|c| {
            let n = c
                .structured_name
                .as_ref()
                .map(|sn| {
                    format!(
                        "{};{};{};{};{}",
                        sn.family.join(" "),
                        sn.given.join(" "),
                        sn.additional.join(" "),
                        sn.prefix.join(" "),
                        sn.suffix.join(" ")
                    )
                })
                .unwrap_or_default();

            let source = match &c.source_detail {
                crate::domain::contact::SourceDetail::ProtonAutosave => "proton-autosave",
                crate::domain::contact::SourceDetail::ProtonImport => "proton-import",
                crate::domain::contact::SourceDetail::ProtonWeb => "proton-web",
                crate::domain::contact::SourceDetail::Google => "google",
                crate::domain::contact::SourceDetail::Apple => "apple",
                crate::domain::contact::SourceDetail::Unknown(s) => s.as_str(),
            };

            let result = match &c.decision {
                ScreeningDecision::Conserved => "conserved",
                ScreeningDecision::NeedsReview(_) => "review",
                _ => unreachable!(),
            };

            ContactExport {
                uid: &c.uid,
                fn_value: &c.fn_value,
                n,
                org: c.org.as_deref(),
                tels: c.tels.iter().map(|t| t.value.clone()).collect(),
                emails: c.emails.iter().map(|e| e.value.clone()).collect(),
                title: c.title.as_deref(),
                role: c.role.as_deref(),
                categories: {
                    let mut cats: Vec<String> = Vec::new();
                    for cat in &c.categories.n1 {
                        cats.push(cat.clone());
                    }
                    for cat in &c.categories.n2 {
                        cats.push(cat.clone());
                    }
                    cats.sort();
                    cats
                },
                source: source.into(),
                cribado_result: result,
                screening_rule: &c.screening_rule,
            }
        })
        .collect();

    let json = serde_json::to_string_pretty(&exports)?;
    fs::write(path, json)?;
    Ok(())
}

// ── tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contact::{CategorySet, SourceDetail, Tel, TelType};
    use crate::domain::screening::ScreeningDecision;

    #[test]
    fn test_export_json_basic() {
        let contact = Contact {
            uid: "u1".into(),
            fn_value: "Juan".into(),
            structured_name: None,
            org: None,
            org_fullname: None,
            org_legal_form: None,
            emails: vec![],
            tels: vec![Tel {
                value: "+34600000000".into(),
                tel_type: TelType::Cell,
                normalized: true,
            }],
            title: None,
            role: None,
            note: None,
            categories: CategorySet::default(),
            source_detail: SourceDetail::Unknown(String::new()),
            decision: ScreeningDecision::Conserved,
            screening_rule: String::new(),
            merged_uids: vec![],
        };

        let dir = std::env::temp_dir();
        let path = dir.join("test_export.json");
        export_json(&[contact], &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"uid\": \"u1\""));
        assert!(content.contains("\"fn_value\": \"Juan\""));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_export_json_skips_eliminated() {
        let c = Contact {
            uid: "e1".into(),
            fn_value: "Elim".into(),
            structured_name: None,
            org: None,
            org_fullname: None,
            org_legal_form: None,
            emails: vec![],
            tels: vec![],
            title: None,
            role: None,
            note: None,
            categories: CategorySet::default(),
            source_detail: SourceDetail::Unknown(String::new()),
            decision: ScreeningDecision::Eliminated(crate::domain::screening::ElimCode::E1),
            screening_rule: String::new(),
            merged_uids: vec![],
        };

        let dir = std::env::temp_dir();
        let path = dir.join("test_skip.json");
        export_json(&[c], &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content.trim(), "[]");

        let _ = fs::remove_file(&path);
    }
}
