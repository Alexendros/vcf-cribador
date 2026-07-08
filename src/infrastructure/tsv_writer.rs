//! Auditoría TSV — trazabilidad completa de cada contacto procesado.

use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;

use crate::domain::screening::ScreeningDecision;
use crate::error::CribaError;
use crate::infrastructure::parser::ParsedVCard;

/// Escribe el archivo de auditoría TSV con una fila por cada contacto de entrada.
pub fn write_audit_tsv(
    vcards: &[ParsedVCard],
    contacts: &[crate::domain::contact::Contact],
    _vcard_map: &HashMap<String, &ParsedVCard>,
    path: &Path,
) -> Result<(), CribaError> {
    // Construir índice de contactos finales por UID
    let contact_by_uid: HashMap<&str, &crate::domain::contact::Contact> =
        contacts.iter().map(|c| (c.uid.as_str(), c)).collect();

    // Determinar qué UIDs fueron fusionados (absorbidos por otro)
    let mut fusionados: HashMap<String, String> = HashMap::new();
    for c in contacts {
        for merged_uid in &c.merged_uids {
            fusionados.insert(merged_uid.clone(), c.uid.clone());
        }
    }

    let mut buf = String::with_capacity(vcards.len() * 256);

    // Header
    writeln!(
        buf,
        "TIMESTAMP\tUID\tFN_ORIGINAL\tFN_FINAL\tACCION\tMOTIVO\tREGLA\tCATEGORIAS\tSOURCE\tTELS\tEMAILS"
    )?;

    let now = jiff::Timestamp::now()
        .strftime("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    for vcard in vcards {
        let uid = vcard.compute_uid();

        let (accion, motivo, regla, categorias, fn_final, tels, emails) =
            if let Some(contact) = contact_by_uid.get(uid.as_str()) {
                let (a, m) = accion_motivo(contact);
                (
                    a,
                    m,
                    contact.screening_rule.clone(),
                    format_categories_tsv(&contact.categories),
                    contact.fn_value.clone(),
                    format_tels_tsv(&contact.tels),
                    format_emails_tsv(&contact.emails),
                )
            } else if let Some(absorber_uid) = fusionados.get(&uid) {
                if let Some(contact) = contact_by_uid.get(absorber_uid.as_str()) {
                    (
                        "FUSIONADO",
                        format!("Fusionado en {}", absorber_uid),
                        String::new(),
                        String::new(),
                        contact.fn_value.clone(),
                        String::new(),
                        String::new(),
                    )
                } else {
                    (
                        "DESCONOCIDO",
                        "No encontrado en contactos finales ni fusionados".to_string(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                    )
                }
            } else {
                (
                    "DESCONOCIDO",
                    "No encontrado en contactos finales".to_string(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                )
            };

        let fn_original = vcard
            .fn_raw
            .as_deref()
            .unwrap_or("Sin nombre")
            .replace(['\t', '\n'], " ");

        let source = vcard_source(vcard);

        writeln!(
            buf,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            now,
            escape_tsv(&uid),
            escape_tsv(&fn_original),
            escape_tsv(&fn_final),
            escape_tsv(accion),
            escape_tsv(&motivo),
            escape_tsv(&regla),
            escape_tsv(&categorias),
            escape_tsv(source),
            escape_tsv(&tels),
            escape_tsv(&emails),
        )?;
    }

    fs::write(path, buf.as_bytes())?;
    tracing::info!(
        "Auditoría TSV escrita: {} ({} filas)",
        path.display(),
        vcards.len()
    );

    Ok(())
}

fn accion_motivo(contact: &crate::domain::contact::Contact) -> (&str, String) {
    match &contact.decision {
        ScreeningDecision::Conserved => ("CONSERVADO", "Conservado por regla de cribado".into()),
        ScreeningDecision::Eliminated(code) => {
            let motivo = match code {
                crate::domain::screening::ElimCode::E1 => "FN es email sin identidad",
                crate::domain::screening::ElimCode::E3 => "Sin EMAIL ni TEL",
                crate::domain::screening::ElimCode::E4 => "Servicio descontinuado",
                crate::domain::screening::ElimCode::E6 => "Inactivo > 5 años",
            };
            ("ELIMINADO", motivo.into())
        }
        ScreeningDecision::Quarantine(code) => {
            let motivo = match code {
                crate::domain::screening::ElimCode::E1 => "Cuarentena: FN es email",
                crate::domain::screening::ElimCode::E3 => "Cuarentena: huérfano",
                crate::domain::screening::ElimCode::E4 => "Cuarentena: servicio descontinuado",
                crate::domain::screening::ElimCode::E6 => "Cuarentena: inactivo > 5 años",
            };
            ("CUARENTENA", motivo.into())
        }
        ScreeningDecision::NeedsReview(reason) => {
            let motivo = match reason {
                crate::domain::screening::ReviewReason::E2InappropriateMetadata => {
                    "Metadatos inapropiados"
                }
                crate::domain::screening::ReviewReason::D3DuplicateCandidate => {
                    "Candidato a duplicado"
                }
                crate::domain::screening::ReviewReason::D6FuzzyDuplicate => "Duplicado difuso",
            };
            ("REVISION", motivo.into())
        }
    }
}

fn format_categories_tsv(cats: &crate::domain::contact::CategorySet) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for c in &cats.n1 {
        parts.push(c.as_str());
    }
    for c in &cats.n2 {
        parts.push(c.as_str());
    }
    parts.join(",")
}

fn format_tels_tsv(tels: &[crate::domain::contact::Tel]) -> String {
    tels.iter()
        .map(|t| t.value.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

fn format_emails_tsv(emails: &[crate::domain::contact::TypedValue]) -> String {
    emails
        .iter()
        .map(|e| e.value.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

fn vcard_source(vcard: &ParsedVCard) -> &str {
    // La fuente se determina a nivel de pipeline; aquí devolvemos
    // un fallback basado en el prodid.
    vcard
        .prodid
        .as_deref()
        .map(|p| {
            if p.contains("ProtonMail") {
                "proton"
            } else {
                "unknown"
            }
        })
        .unwrap_or("unknown")
}

fn escape_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

// ── tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contact::{CategorySet, Contact, SourceDetail};
    use crate::domain::screening::{ElimCode, ScreeningDecision};

    fn make_parsed_vcard(uid: &str, fn_raw: &str) -> ParsedVCard {
        ParsedVCard {
            raw_properties: vec![],
            uid: Some(uid.into()),
            fn_raw: Some(fn_raw.into()),
            n_raw: None,
            org_raw: None,
            emails_raw: vec![],
            tels_raw: vec![],
            title_raw: None,
            role_raw: None,
            note_raw: None,
            photo_lines: vec![],
            logo_lines: vec![],
            sound_lines: vec![],
            key_lines: vec![],
            version: Some("4.0".into()),
            prodid: None,
        }
    }

    fn make_contact(uid: &str, fn_val: &str, decision: ScreeningDecision) -> Contact {
        Contact {
            uid: uid.into(),
            fn_value: fn_val.into(),
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
            decision,
            screening_rule: String::new(),
            merged_uids: vec![],
        }
    }

    #[test]
    fn test_write_tsv_basic() {
        let vcards = vec![make_parsed_vcard("u1", "Juan Pérez")];
        let contacts = vec![make_contact(
            "u1",
            "Juan Pérez",
            ScreeningDecision::Conserved,
        )];
        let mut map = HashMap::new();
        map.insert("u1".into(), &vcards[0]);

        let dir = std::env::temp_dir();
        let path = dir.join("test_tsv_basic.tsv");
        write_audit_tsv(&vcards, &contacts, &map, &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("TIMESTAMP\tUID\tFN_ORIGINAL"));
        assert!(content.contains("CONSERVADO"));
        assert!(content.contains("Juan Pérez"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_write_tsv_eliminated() {
        let vcards = vec![make_parsed_vcard("e1", "spam@test.com")];
        let mut contact = make_contact(
            "e1",
            "spam@test.com",
            ScreeningDecision::Eliminated(ElimCode::E1),
        );
        contact.screening_rule = "E1".into();
        let contacts = vec![contact];
        let mut map = HashMap::new();
        map.insert("e1".into(), &vcards[0]);

        let dir = std::env::temp_dir();
        let path = dir.join("test_tsv_eliminated.tsv");
        write_audit_tsv(&vcards, &contacts, &map, &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("ELIMINADO"));
        assert!(content.contains("E1"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_write_tsv_columns_count() {
        let vcards = vec![make_parsed_vcard("c1", "Test")];
        let contacts = vec![make_contact("c1", "Test", ScreeningDecision::Conserved)];
        let mut map = HashMap::new();
        map.insert("c1".into(), &vcards[0]);

        let dir = std::env::temp_dir();
        let path = dir.join("test_tsv_columns.tsv");
        write_audit_tsv(&vcards, &contacts, &map, &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2); // header + 1 data row
        assert_eq!(lines[0].split('\t').count(), 11);
        assert_eq!(lines[1].split('\t').count(), 11);

        let _ = fs::remove_file(&path);
    }
}
