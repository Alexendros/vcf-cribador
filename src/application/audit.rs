//! Auditoría standalone — screening + TSV sin normalizar ni dedup.

use std::fs;
use std::path::Path;

use crate::domain::screening::decide;
use crate::error::CribaError;
use crate::infrastructure::config::load_config;
use crate::infrastructure::encoding::ensure_utf8;
use crate::infrastructure::parser::{parse_vcards, unfold};
use crate::infrastructure::tsv_writer::write_audit_tsv;

/// Ejecuta screening + auditoría TSV.
pub fn execute(
    input: &Path,
    output: Option<&Path>,
    _config: Option<&Path>,
    source_override: &str,
) -> Result<(), CribaError> {
    let bytes = fs::read(input)?;
    if bytes.is_empty() {
        return Err(CribaError::EmptyVcf);
    }

    let utf8_text = ensure_utf8(&bytes)?;
    let unfolded = unfold(&utf8_text);
    let vcards = parse_vcards(&unfolded)?;
    let total = vcards.len();
    tracing::info!("Auditoría: {} contactos parseados", total);

    // Construir vcard_map
    let mut vcard_map: std::collections::HashMap<
        String,
        &crate::infrastructure::parser::ParsedVCard,
    > = std::collections::HashMap::with_capacity(vcards.len());
    for vcard in &vcards {
        vcard_map.insert(vcard.compute_uid(), vcard);
    }

    // Determinar fuente
    let source_detail = if source_override != "auto" {
        match source_override.to_lowercase().as_str() {
            "proton" => crate::domain::contact::SourceDetail::ProtonAutosave,
            "google" => crate::domain::contact::SourceDetail::Google,
            "apple" => crate::domain::contact::SourceDetail::Apple,
            _ => crate::domain::contact::SourceDetail::Unknown(source_override.into()),
        }
    } else {
        let prodids: Vec<String> = vcards.iter().filter_map(|v| v.prodid.clone()).collect();
        let uids: Vec<String> = vcards.iter().filter_map(|v| v.uid.clone()).collect();
        crate::infrastructure::source::detect_source(&prodids, &uids)
    };

    let config = load_config(_config)?;

    let mut contacts = Vec::with_capacity(vcards.len());
    for vcard in &vcards {
        let mut adapted = vcard.clone();
        if vcard.version.as_deref() == Some("3.0") {
            crate::infrastructure::v3_compat::adapt_v3(&mut adapted);
        }
        let mut contact = adapted.to_contact()?;
        contact.source_detail = source_detail.clone();
        let trace = decide(&contact, &config);
        contact.screening_rule = trace.triggered_rule.clone();
        contact.decision = trace.outcome;
        contacts.push(contact);
    }

    let audit_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new("audit.tsv").to_path_buf());

    write_audit_tsv(&vcards, &contacts, &vcard_map, &audit_path)?;

    println!("Auditoría escrita: {}", audit_path.display());
    println!("  Contactos: {}", total);

    Ok(())
}
