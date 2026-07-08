//! Export CSV.

use std::path::Path;

use crate::domain::contact::Contact;
use crate::domain::screening::ScreeningDecision;
use crate::error::CribaError;

/// Exporta contactos conservados y needs_review a CSV.
pub fn export_csv(contacts: &[Contact], path: &Path) -> Result<(), CribaError> {
    let mut wtr = csv::Writer::from_path(path)?;

    wtr.write_record([
        "FN",
        "N",
        "ORG",
        "TEL",
        "EMAIL",
        "CATEGORIES",
        "SOURCE",
        "CRIBADO_RESULT",
    ])?;

    for c in contacts {
        if matches!(
            &c.decision,
            ScreeningDecision::Eliminated(_) | ScreeningDecision::Quarantine(_)
        ) {
            continue;
        }

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

        let tels = c
            .tels
            .iter()
            .map(|t| t.value.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        let emails = c
            .emails
            .iter()
            .map(|e| e.value.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        let categories = {
            let mut parts: Vec<&str> = Vec::new();
            for cat in &c.categories.n1 {
                parts.push(cat.as_str());
            }
            for cat in &c.categories.n2 {
                parts.push(cat.as_str());
            }
            parts.join(", ")
        };

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

        wtr.write_record([
            &c.fn_value,
            &n,
            c.org.as_deref().unwrap_or(""),
            &tels,
            &emails,
            &categories,
            source,
            result,
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

// ── tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contact::{CategorySet, Contact, SourceDetail, Tel, TelType, TypedValue};
    use crate::domain::screening::ScreeningDecision;
    use std::fs;

    #[test]
    fn test_export_csv_basic() {
        let contact = Contact {
            uid: "u1".into(),
            fn_value: "Juan Pérez".into(),
            structured_name: None,
            org: Some("Juzgado".into()),
            org_fullname: None,
            org_legal_form: None,
            emails: vec![TypedValue {
                value: "juan@example.com".into(),
                types: vec![],
                pref: 0,
            }],
            tels: vec![Tel {
                value: "+34600000000".into(),
                tel_type: TelType::Cell,
                normalized: true,
            }],
            title: None,
            role: None,
            note: None,
            categories: CategorySet::default(),
            source_detail: SourceDetail::ProtonAutosave,
            decision: ScreeningDecision::Conserved,
            screening_rule: String::new(),
            merged_uids: vec![],
        };

        let dir = std::env::temp_dir();
        let path = dir.join("test_export.csv");
        export_csv(&[contact], &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("FN,N,ORG,TEL,EMAIL"));
        assert!(content.contains("Juan Pérez"));
        assert!(content.contains("+34600000000"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_export_csv_skips_eliminated() {
        let mut c = Contact {
            uid: "e1".into(),
            fn_value: "Eliminado".into(),
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
        c.decision = ScreeningDecision::Eliminated(crate::domain::screening::ElimCode::E1);

        let dir = std::env::temp_dir();
        let path = dir.join("test_skip.csv");
        export_csv(&[c], &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        // Solo header, sin datos
        let lines: Vec<_> = content.lines().collect();
        assert_eq!(lines.len(), 1);

        let _ = fs::remove_file(&path);
    }
}
