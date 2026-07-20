//! Caso de uso: CribarContacts — pipeline completo.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::application::stats::Stats;
use crate::domain::audit::AuditEntry;
use crate::domain::classification::classify;
use crate::domain::contact::Contact;
use crate::domain::identity::deduplicate;
use crate::domain::normalization::{normalize_fn, normalize_org, normalize_tel};
use crate::domain::screening::{decide, ScreeningDecision};
use crate::domain::verification::verify;
use crate::error::CribaError;
use crate::infrastructure::config::load_config;
use crate::infrastructure::encoding::ensure_utf8;
use crate::infrastructure::parser::{parse_vcards, unfold};
use crate::infrastructure::source::{detect_source, detect_version};
use crate::infrastructure::tsv_writer::write_audit_tsv;
use crate::infrastructure::v3_compat::adapt_v3;
use crate::infrastructure::writer::write_vcf;

/// Ejecuta el pipeline completo de cribado.
/// Retorna estadísticas y los contactos procesados tras la deduplicación.
pub fn execute(
    input: &Path,
    output: Option<&Path>,
    audit: Option<&Path>,
    config: Option<&Path>,
    source_override: &str,
    dry_run: bool,
) -> Result<(Stats, Vec<Contact>), CribaError> {
    tracing::info!("Leyendo archivo: {}", input.display());
    let bytes = fs::read(input)?;

    if bytes.is_empty() {
        return Err(CribaError::EmptyVcf);
    }

    // 1. Transcodificar a UTF-8
    let utf8_text = ensure_utf8(&bytes)?;

    // 2. Desplegar líneas
    let unfolded = unfold(&utf8_text);

    // 3. Parsear
    let vcards = parse_vcards(&unfolded)?;
    let total_entrada = vcards.len();
    tracing::info!("Parseados {} contactos", total_entrada);

    // 4. Construir mapa UID -> &ParsedVCard (acceso O(1) para writer/auditoría)
    let mut vcard_map: HashMap<String, &crate::infrastructure::parser::ParsedVCard> =
        HashMap::with_capacity(vcards.len());
    for vcard in &vcards {
        vcard_map.insert(vcard.compute_uid(), vcard);
    }

    // 5. Detectar fuente y versión
    let prodids: Vec<String> = vcards.iter().filter_map(|v| v.prodid.clone()).collect();
    let uids: Vec<String> = vcards.iter().filter_map(|v| v.uid.clone()).collect();
    let versions: Vec<String> = vcards.iter().filter_map(|v| v.version.clone()).collect();

    let detected_source = detect_source(&prodids, &uids);
    let detected_version = detect_version(&versions);
    tracing::info!(
        "Fuente detectada: {:?}, versión: {}",
        detected_source,
        detected_version
    );

    // 6. Determinar source_detail final
    let source_detail = if source_override != "auto" {
        match source_override.to_lowercase().as_str() {
            "proton" => crate::domain::contact::SourceDetail::ProtonAutosave,
            "google" => crate::domain::contact::SourceDetail::Google,
            "apple" => crate::domain::contact::SourceDetail::Apple,
            _ => detected_source,
        }
    } else {
        detected_source
    };

    // 7. Cargar configuración
    let screening_config = load_config(config)?;

    // 8. Procesar cada contacto
    let mut contacts = Vec::with_capacity(vcards.len());
    let mut audit_entries = Vec::with_capacity(vcards.len());
    let mut eliminados = 0usize;
    let mut cuarentena = 0usize;
    let mut needs_review = 0usize;

    for vcard in &vcards {
        let mut adapted = vcard.clone();
        if vcard.version.as_deref() == Some("3.0") {
            adapt_v3(&mut adapted);
        }

        let mut contact = adapted.to_contact()?;
        contact.source_detail = source_detail.clone();

        let trace = decide(&contact, &screening_config);
        contact.screening_rule = trace.triggered_rule.clone();

        match &trace.outcome {
            ScreeningDecision::Eliminated(_) => {
                eliminados += 1;
            }
            ScreeningDecision::Quarantine(_) => {
                cuarentena += 1;
            }
            ScreeningDecision::NeedsReview(_) => {
                needs_review += 1;
            }
            ScreeningDecision::Conserved => {}
        }

        contact.decision = trace.outcome.clone();

        // Normalización (solo conservados y needs_review)
        let (fn_normalized, title_extra, role_extra) = normalize_fn(&contact.fn_value);
        contact.fn_value = fn_normalized;
        if title_extra.is_some() && contact.title.is_none() {
            contact.title = title_extra;
        }
        if role_extra.is_some() && contact.role.is_none() {
            contact.role = role_extra;
        }

        for tel in &mut contact.tels {
            let tel_type = tel.tel_type;
            let normalized = normalize_tel(&tel.value, &screening_config.prefijo_pais, tel_type);
            *tel = normalized;
        }

        if let Some(ref org) = contact.org {
            let (org_clean, legal_form) = normalize_org(org);
            contact.org = Some(org_clean);
            if contact.org_legal_form.is_none() {
                contact.org_legal_form = legal_form;
            }
        }

        contact.categories = classify(&contact);

        let fn_original = vcard
            .fn_raw
            .as_deref()
            .unwrap_or("Sin nombre")
            .replace(['\t', '\n'], " ");
        let audit_entry = AuditEntry::from_contact(&contact, &fn_original, &trace);
        audit_entries.push(audit_entry);

        contacts.push(contact);
    }

    let conservados = contacts.len() - eliminados - cuarentena;

    // 9. Deduplicación
    let (contacts, fusionados) = deduplicate(contacts);

    // Actualizar entradas de auditoría para contactos fusionados
    let mut merged_map: HashMap<String, String> = HashMap::new();
    for c in &contacts {
        for merged_uid in &c.merged_uids {
            merged_map.insert(merged_uid.clone(), c.uid.clone());
        }
    }
    for entry in &mut audit_entries {
        if let Some(target) = merged_map.get(&entry.uid) {
            *entry = entry.clone().merged_into(target);
        }
    }

    // 10. Verificación de invariantes
    let warnings = verify(&contacts, Some(input), output);
    if !warnings.is_empty() {
        for warning in &warnings {
            tracing::warn!("{}", warning);
        }
    }

    tracing::info!(
        "Pipeline: {} entrada, {} conservados, {} eliminados, {} cuarentena, {} fusionados, {} needs_review",
        total_entrada, conservados, eliminados, cuarentena, fusionados, needs_review
    );

    let stats = Stats::compute(
        &contacts,
        total_entrada,
        conservados,
        eliminados,
        fusionados,
        cuarentena,
        needs_review,
    );

    // 10. Salida
    if !dry_run {
        if let Some(out_path) = output {
            write_vcf(&contacts, &vcard_map, out_path, true)?;
        }
        if let Some(audit_path) = audit {
            write_audit_tsv(&audit_entries, audit_path)?;
        }
    }

    Ok((stats, contacts))
}
