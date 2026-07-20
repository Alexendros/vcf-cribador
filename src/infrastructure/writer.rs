//! Serialización de Contact → VCF 4.0 (RFC 6350).
//!
//! Incluye folding a 75 octetos respetando límites UTF-8 multibyte,
//! escapado RFC 6350 §3.4, y preservación de datos binarios (PHOTO, LOGO, SOUND, KEY).

use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;

use crate::domain::contact::{CategorySet, Contact, SourceDetail, StructuredName, Tel, TelType};
use crate::domain::screening::ScreeningDecision;
use crate::error::CribaError;
use crate::infrastructure::parser::ParsedVCard;

/// Escribe los contactos conservados y needs_review a un archivo VCF 4.0.
/// Si `include_quarantine` es true, los contactos en cuarentena van a
/// `<path>_cuarentena.vcf`.
pub fn write_vcf(
    contacts: &[Contact],
    vcard_map: &HashMap<String, &ParsedVCard>,
    path: &Path,
    include_quarantine: bool,
) -> Result<(), CribaError> {
    let mut conserved_buf = String::new();
    let mut quarantine_buf = String::new();

    for contact in contacts {
        let vcard = vcard_map.get(&contact.uid);

        match &contact.decision {
            ScreeningDecision::Conserved | ScreeningDecision::NeedsReview(_) => {
                write_one_contact(&mut conserved_buf, contact, vcard)?;
            }
            ScreeningDecision::Quarantine(_) => {
                if include_quarantine {
                    write_one_contact(&mut quarantine_buf, contact, vcard)?;
                }
            }
            ScreeningDecision::Eliminated(_) => {
                // no se escriben
            }
        }
    }

    fs::write(path, conserved_buf.as_bytes())?;
    tracing::info!(
        "VCF escrito: {} ({:.1} KB)",
        path.display(),
        conserved_buf.len() as f64 / 1024.0
    );

    if include_quarantine && !quarantine_buf.is_empty() {
        let quarantine_path = quarantine_path(path);
        fs::write(&quarantine_path, quarantine_buf.as_bytes())?;
        tracing::info!("VCF cuarentena escrito: {}", quarantine_path.display());
    }

    Ok(())
}

fn quarantine_path(path: &Path) -> std::path::PathBuf {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    path.with_file_name(format!("{}_cuarentena{}", stem, ext))
}

fn write_one_contact(
    buf: &mut String,
    contact: &Contact,
    vcard: Option<&&ParsedVCard>,
) -> Result<(), CribaError> {
    write_prop(buf, "BEGIN:VCARD")?;
    write_prop(buf, "VERSION:4.0")?;

    // PRODID del original
    if let Some(pv) = vcard.and_then(|v| v.prodid.as_ref()) {
        write_prop(buf, &format!("PRODID:{}", escape_vcard(pv)))?;
    }

    // UID
    write_prop(buf, &format!("UID:{}", escape_vcard(&contact.uid)))?;

    // FN (obligatorio)
    write_prop(buf, &format!("FN:{}", escape_vcard(&contact.fn_value)))?;

    // N (obligatorio)
    let n = format_n(&contact.structured_name);
    write_prop(buf, &format!("N:{}", n))?;

    // ORG
    if let Some(ref org) = contact.org {
        write_prop(buf, &format!("ORG:{}", org))?;
    }

    // TEL
    for tel in &contact.tels {
        write_prop(buf, &format_tel(tel))?;
    }

    // EMAIL
    for email in &contact.emails {
        write_prop(buf, &format_email(email))?;
    }

    // ADR
    for addr in &contact.addresses {
        write_prop(buf, &format_address(addr))?;
    }

    // TITLE
    if let Some(ref title) = contact.title {
        write_prop(buf, &format!("TITLE:{}", escape_vcard(title)))?;
    }

    // ROLE
    if let Some(ref role) = contact.role {
        write_prop(buf, &format!("ROLE:{}", escape_vcard(role)))?;
    }

    // NOTE
    if let Some(ref note) = contact.note {
        write_prop(buf, &format!("NOTE:{}", escape_vcard(note)))?;
    } else if let Some(pv) = vcard.and_then(|v| v.note_raw.as_ref()) {
        write_prop(buf, &format!("NOTE:{}", escape_vcard(pv)))?;
    }

    // CATEGORIES
    if !contact.categories.n1.is_empty() || !contact.categories.n2.is_empty() {
        write_prop(buf, &format_categories(&contact.categories))?;
    }

    // Datos binarios preservados del original (PHOTO, LOGO, SOUND, KEY)
    if let Some(pv) = vcard {
        for line in &pv.photo_lines {
            write_prop_raw(buf, line)?;
        }
        for line in &pv.logo_lines {
            write_prop_raw(buf, line)?;
        }
        for line in &pv.sound_lines {
            write_prop_raw(buf, line)?;
        }
        for line in &pv.key_lines {
            write_prop_raw(buf, line)?;
        }
    }

    // X-CRIBADO-*
    write_prop(
        buf,
        &format!("X-CRIBADO-RESULT:{}", cribado_result(contact)),
    )?;
    write_prop(
        buf,
        &format!("X-CRIBADO-VERSION:{}", env!("CARGO_PKG_VERSION")),
    )?;
    write_prop(
        buf,
        &format!(
            "X-CRIBADO-DATE:{}",
            jiff::Timestamp::now().strftime("%Y-%m-%dT%H:%M:%SZ")
        ),
    )?;

    // X-SOURCE
    write_prop(
        buf,
        &format!("X-SOURCE:{}", source_to_string(&contact.source_detail)),
    )?;

    // X-MERGED-UID
    if !contact.merged_uids.is_empty() {
        write_prop(
            buf,
            &format!("X-MERGED-UID:{}", contact.merged_uids.join(";")),
        )?;
    }

    write_prop(buf, "END:VCARD")?;
    writeln!(buf)?;
    Ok(())
}

// ── helpers de formato ──

fn write_prop(buf: &mut String, content: &str) -> Result<(), CribaError> {
    let folded = fold_line(content);
    buf.push_str(&folded);
    buf.push_str("\r\n");
    Ok(())
}

fn write_prop_raw(buf: &mut String, line: &str) -> Result<(), CribaError> {
    buf.push_str(line);
    if !line.ends_with('\n') {
        buf.push_str("\r\n");
    } else if !line.ends_with("\r\n") {
        // termina en \n pero no \r\n → normalizar
        let trimmed = line.trim_end_matches('\n');
        buf.truncate(buf.len() - line.len());
        buf.push_str(trimmed);
        buf.push_str("\r\n");
    }
    Ok(())
}

fn format_n(name: &Option<StructuredName>) -> String {
    match name {
        Some(sn) => {
            let family = sn.family.join(" ");
            let given = sn.given.join(" ");
            let additional = sn.additional.join(" ");
            let prefix = sn.prefix.join(" ");
            let suffix = sn.suffix.join(" ");
            format!("{};{};{};{};{}", family, given, additional, prefix, suffix)
        }
        None => ";;;;".to_string(),
    }
}

fn format_tel(tel: &Tel) -> String {
    let type_str = tel_type_to_str(tel.tel_type);
    format!("TEL;TYPE={}:{}", type_str, escape_vcard(&tel.value))
}

fn format_email(email: &crate::domain::contact::TypedValue) -> String {
    let mut out = String::from("EMAIL");
    if !email.types.is_empty() {
        out.push_str(";TYPE=");
        out.push_str(&email.types.join(","));
    }
    if email.pref > 0 {
        write!(out, ";PREF={}", email.pref).unwrap();
    }
    write!(out, ":{}", escape_vcard(&email.value)).unwrap();
    out
}

fn format_categories(cats: &CategorySet) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut n1_sorted: Vec<&String> = cats.n1.iter().collect();
    n1_sorted.sort();
    parts.extend(n1_sorted.into_iter().cloned());
    let mut n2_sorted: Vec<&String> = cats.n2.iter().collect();
    n2_sorted.sort();
    parts.extend(n2_sorted.into_iter().cloned());
    format!("CATEGORIES:{}", parts.join(","))
}

fn tel_type_to_str(t: TelType) -> &'static str {
    match t {
        TelType::Cell => "cell",
        TelType::Home => "home",
        TelType::Work => "work",
        TelType::Main => "main",
        TelType::Fax => "fax",
        TelType::Pager => "pager",
        TelType::Text => "text",
        TelType::Video => "video",
        TelType::Other => "other",
    }
}

fn format_address(addr: &crate::domain::contact::Address) -> String {
    let parts = [
        addr.po_box.as_str(),
        addr.extended.as_str(),
        addr.street.as_str(),
        addr.locality.as_str(),
        addr.region.as_str(),
        addr.postal_code.as_str(),
        addr.country.as_str(),
    ];
    let value = parts.join(";");
    if addr.types.is_empty() {
        format!("ADR:{}", escape_vcard(&value))
    } else {
        format!("ADR;TYPE={}:{}", addr.types.join(","), escape_vcard(&value))
    }
}

fn source_to_string(sd: &SourceDetail) -> &str {
    match sd {
        SourceDetail::ProtonAutosave => "proton-autosave",
        SourceDetail::ProtonImport => "proton-import",
        SourceDetail::ProtonWeb => "proton-web",
        SourceDetail::Google => "google",
        SourceDetail::Apple => "apple",
        SourceDetail::Unknown(s) if s.is_empty() => "unknown",
        SourceDetail::Unknown(s) => s.as_str(),
    }
}

fn cribado_result(contact: &Contact) -> &str {
    match &contact.decision {
        ScreeningDecision::Conserved => "conserved",
        ScreeningDecision::Eliminated(_) => "eliminated",
        ScreeningDecision::Quarantine(_) => "quarantine",
        ScreeningDecision::NeedsReview(_) => "review",
    }
}

// ── escape RFC 6350 §3.4 ──

fn escape_vcard(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            ',' => out.push_str("\\,"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str(""),
            c => out.push(c),
        }
    }
    out
}

// ── folding 75 octetos ──

fn fold_line(line: &str) -> String {
    let max_len = 75;
    if line.len() <= max_len {
        return line.to_string();
    }

    let mut result = String::with_capacity(line.len() + line.len() / max_len * 3);
    let mut remaining = line;

    while remaining.len() > max_len {
        let mut pos = max_len;
        // Retroceder si estamos en medio de un carácter UTF-8 multibyte
        while pos > 0 && (remaining.as_bytes()[pos] & 0xC0) == 0x80 {
            pos -= 1;
        }
        if pos == 0 {
            // no se pudo encontrar un corte seguro en este rango; truncar en max_len
            pos = max_len;
        }

        result.push_str(&remaining[..pos]);
        result.push_str("\r\n ");
        remaining = &remaining[pos..];
    }

    result.push_str(remaining);
    result
}

// ── tests ──

#[cfg(test)]
mod tests {
    use super::*;

    fn make_contact(fn_val: &str) -> Contact {
        Contact {
            uid: "test-uid".into(),
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
            addresses: vec![],
            categories: CategorySet::default(),
            source_detail: SourceDetail::Unknown(String::new()),
            decision: ScreeningDecision::Conserved,
            screening_rule: String::new(),
            merged_uids: vec![],
        }
    }

    // ── escape ──

    #[test]
    fn test_escape_passthrough() {
        assert_eq!(escape_vcard("Hello World"), "Hello World");
    }

    #[test]
    fn test_escape_semicolon() {
        assert_eq!(escape_vcard("a;b"), "a\\;b");
    }

    #[test]
    fn test_escape_comma() {
        assert_eq!(escape_vcard("a,b"), "a\\,b");
    }

    #[test]
    fn test_escape_backslash() {
        assert_eq!(escape_vcard("a\\b"), "a\\\\b");
    }

    #[test]
    fn test_escape_newline() {
        assert_eq!(escape_vcard("a\nb"), "a\\nb");
    }

    #[test]
    fn test_escape_combined() {
        assert_eq!(escape_vcard("a;b,c\\d\ne"), "a\\;b\\,c\\\\d\\ne");
    }

    // ── folding ──

    #[test]
    fn test_fold_short_line() {
        assert_eq!(fold_line("short"), "short");
    }

    #[test]
    fn test_fold_long_line() {
        let long = "a".repeat(80);
        let folded = fold_line(&long);
        // Max line length (without continuation marker) should be ≤ 75
        for segment in folded.split("\r\n ") {
            assert!(
                segment.len() <= 75,
                "segment '{}' has len {}",
                segment,
                segment.len()
            );
        }
    }

    #[test]
    fn test_fold_at_75_exact() {
        let line = "X".repeat(75);
        assert_eq!(fold_line(&line), line);
    }

    #[test]
    fn test_fold_at_76() {
        let line = "X".repeat(76);
        let folded = fold_line(&line);
        assert!(folded.contains("\r\n "));
        assert_eq!(folded.len(), 76 + 3); // 76 chars + \r\n + space
    }

    #[test]
    fn test_fold_preserves_multibyte() {
        // 'ñ' is 2 bytes. Place it at position 74-75 (byte indices).
        // Line: 74 X's + 'ñ' + 'Y' = 78 bytes, 77 chars
        let mut line = "X".repeat(74);
        line.push('ñ');
        line.push('Y');
        let folded = fold_line(&line);
        // First segment should end before the 'ñ', not split it
        let first_segment = folded.split("\r\n ").next().unwrap();
        // Verify ñ is intact (not split into continuation bytes)
        assert!(first_segment.ends_with('X') || first_segment.ends_with('ñ'));
    }

    // ── format_n ──

    #[test]
    fn test_format_n_none() {
        assert_eq!(format_n(&None), ";;;;");
    }

    #[test]
    fn test_format_n_family_given() {
        let sn = StructuredName {
            family: vec!["García".into()],
            given: vec!["María".into()],
            ..Default::default()
        };
        assert_eq!(format_n(&Some(sn)), "García;María;;;");
    }

    // ── format_tel ──

    #[test]
    fn test_format_tel_cell() {
        let tel = Tel {
            value: "+34600000000".into(),
            tel_type: TelType::Cell,
            normalized: true,
        };
        assert_eq!(format_tel(&tel), "TEL;TYPE=cell:+34600000000");
    }

    // ── format_email ──

    #[test]
    fn test_format_email_with_pref() {
        let email = crate::domain::contact::TypedValue {
            value: "test@example.com".into(),
            types: vec!["internet".into()],
            pref: 1,
        };
        let formatted = format_email(&email);
        assert!(formatted.contains("PREF=1"));
        assert!(formatted.contains("test@example.com"));
    }

    // ── format_categories ──

    #[test]
    fn test_format_categories_deterministic() {
        let mut cats = CategorySet::default();
        cats.n1.insert("PERS".into());
        cats.n1.insert("ORG".into());
        let f1 = format_categories(&cats);
        let f2 = format_categories(&cats);
        assert_eq!(f1, f2);
    }

    // ── write_vcf integration ──

    #[test]
    fn test_write_vcf_basic() {
        use crate::infrastructure::parser::ParsedVCard;

        let contact = make_contact("Juan Pérez");
        let vcard = ParsedVCard {
            raw_properties: vec![],
            uid: Some("test-uid".into()),
            fn_raw: Some("Juan Pérez".into()),
            n_raw: None,
            org_raw: None,
            emails_raw: vec![],
            tels_raw: vec![],
            addresses_raw: vec![],
            title_raw: None,
            role_raw: None,
            note_raw: None,
            rev_raw: None,
            photo_lines: vec![],
            logo_lines: vec![],
            sound_lines: vec![],
            key_lines: vec![],
            version: Some("4.0".into()),
            prodid: None,
        };

        let mut map = HashMap::new();
        map.insert("test-uid".into(), &vcard);

        let dir = std::env::temp_dir();
        let path = dir.join("test_write_vcf_basic.vcf");
        write_vcf(&[contact.clone()], &map, &path, false).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("BEGIN:VCARD"));
        assert!(content.contains("VERSION:4.0"));
        assert!(content.contains("FN:Juan Pérez"));
        assert!(content.contains("N:;;;;"));
        assert!(content.contains("X-CRIBADO-RESULT:conserved"));
        assert!(content.contains("END:VCARD"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_write_vcf_skips_eliminated() {
        let mut eliminated = make_contact("Eliminado");
        eliminated.decision = ScreeningDecision::Eliminated(crate::domain::screening::ElimCode::E1);

        let dir = std::env::temp_dir();
        let path = dir.join("test_skip_eliminated.vcf");
        let map = HashMap::new();
        write_vcf(&[eliminated], &map, &path, false).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(!content.contains("BEGIN:VCARD"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_write_vcf_quarantine_file() {
        let mut qu = make_contact("Cuarentena");
        qu.decision = ScreeningDecision::Quarantine(crate::domain::screening::ElimCode::E4);

        let dir = std::env::temp_dir();
        let path = dir.join("test_quarantine.vcf");
        let map = HashMap::new();
        write_vcf(&[qu.clone()], &map, &path, true).unwrap();

        // El principal no debe tener el contacto en cuarentena
        let main = fs::read_to_string(&path).unwrap();
        assert!(!main.contains("BEGIN:VCARD"));

        // El de cuarentena sí
        let qpath = dir.join("test_quarantine_cuarentena.vcf");
        let qcontent = fs::read_to_string(&qpath).unwrap();
        assert!(qcontent.contains("BEGIN:VCARD"));
        assert!(qcontent.contains("X-CRIBADO-RESULT:quarantine"));

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&qpath);
    }
}
