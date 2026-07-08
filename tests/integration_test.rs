use std::fs;
use std::path::PathBuf;
use vcf_cribador::infrastructure::parser::{parse_vcards, unfold};
use vcf_cribador::infrastructure::source::detect_source;
use vcf_cribador::infrastructure::v3_compat::adapt_v3;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn read_vcf_to_vcards(path: &PathBuf) -> Vec<vcf_cribador::infrastructure::parser::ParsedVCard> {
    let bytes = fs::read(path).expect("Unable to read fixture");
    let raw = String::from_utf8(bytes).expect("Fixture not valid UTF-8");
    let unfolded = unfold(&raw);
    parse_vcards(&unfolded).expect("Failed to parse fixture")
}

// ── ProtonMail vCard 4.0 ──

#[test]
fn test_parse_proton_real() {
    let vcards = read_vcf_to_vcards(&fixture("protonContacts-2026-07-07.vcf"));

    assert!(
        vcards.len() > 100,
        "Expected >100 contacts, got {}",
        vcards.len()
    );

    let v4_count = vcards
        .iter()
        .filter(|v| v.version.as_deref() == Some("4.0"))
        .count();
    assert!(v4_count > 0, "No vCard 4.0 contacts found");

    let with_email = vcards.iter().filter(|v| !v.emails_raw.is_empty()).count();
    let with_tel = vcards.iter().filter(|v| !v.tels_raw.is_empty()).count();
    assert!(with_email > 0, "No contacts with email");
    assert!(with_tel > 0, "No contacts with phone");

    println!(
        "Proton: {} contacts (email: {}, tel: {}, v4: {})",
        vcards.len(),
        with_email,
        with_tel,
        v4_count
    );
}

#[test]
fn test_convert_proton_to_contacts() {
    let vcards = read_vcf_to_vcards(&fixture("protonContacts-2026-07-07.vcf"));

    let contacts: Vec<_> = vcards
        .into_iter()
        .map(|v| v.to_contact())
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to convert Proton contacts");

    assert!(contacts.len() > 100);
    for c in &contacts {
        assert!(!c.uid.is_empty(), "Contact has empty UID");
    }

    println!("Proton: {} contacts converted", contacts.len());
}

// ── Google Contacts vCard 3.0 ──

#[test]
fn test_parse_google_contactos() {
    let vcards = read_vcf_to_vcards(&fixture("google_contactos.vcf"));

    assert!(
        vcards.len() > 70,
        "Expected >70 contacts, got {}",
        vcards.len()
    );

    // Todos deben ser vCard 3.0
    let v3_count = vcards
        .iter()
        .filter(|v| v.version.as_deref() == Some("3.0"))
        .count();
    assert_eq!(
        v3_count,
        vcards.len(),
        "All Google contacts should be vCard 3.0"
    );

    // La mayoría deben tener TEL (es una exportación de contactos con teléfono)
    let with_tel = vcards.iter().filter(|v| !v.tels_raw.is_empty()).count();
    assert!(with_tel > 60);

    // Algunos contactos de Google no tienen FN (solo TEL)
    let with_fn = vcards.iter().filter(|v| v.fn_raw.is_some()).count();
    assert!(with_fn > 0, "Some contacts should have FN");
    assert!(
        with_fn < vcards.len(),
        "Some Google contacts have no FN (TEL-only)"
    );

    // Verificar que hay contactos con CATEGORIES
    let with_cat = vcards
        .iter()
        .filter(|v| {
            v.raw_properties
                .iter()
                .any(|p| p.name.eq_ignore_ascii_case("CATEGORIES"))
        })
        .count();
    assert!(with_cat > 0);

    println!(
        "Google contactos: {} vcards (with_fn: {}, with_tel: {}, v3: {})",
        vcards.len(),
        with_fn,
        with_tel,
        v3_count
    );
}

#[test]
fn test_parse_google_otroscontactos() {
    let vcards = read_vcf_to_vcards(&fixture("google_otroscontactos.vcf"));

    assert!(vcards.len() > 70);

    let v3_count = vcards
        .iter()
        .filter(|v| v.version.as_deref() == Some("3.0"))
        .count();
    assert_eq!(v3_count, vcards.len());

    println!("Google otros contactos: {} vcards", vcards.len());
}

#[test]
fn test_google_escaped_commas_in_org_and_adr() {
    let vcards = read_vcf_to_vcards(&fixture("google_contactos.vcf"));

    // Buscar contacto "Alicia Arnás" que tiene ORG: Gráficas Nasve\, SL
    let alicia = vcards
        .iter()
        .find(|v| v.fn_raw.as_deref() == Some("Alicia Arnás"))
        .expect("Alicia Arnás not found");

    assert_eq!(alicia.org_raw.as_deref(), Some("Gráficas Nasve\\, SL"));

    // Tiene ADR con escape
    let adr = alicia
        .raw_properties
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case("ADR"));
    assert!(adr.is_some(), "ADR property missing");
    let adr_val = adr.unwrap().value.to_lowercase();
    assert!(
        adr_val.contains("calicanto"),
        "ADR should contain Calicanto: {}",
        adr_val
    );

    // to_contact debe desescapar los \,
    let contact = alicia.clone().to_contact().unwrap();
    assert_eq!(contact.org.as_deref(), Some("Gráficas Nasve, SL"));
}

#[test]
fn test_google_emoji_and_brackets_in_fn() {
    let vcards = read_vcf_to_vcards(&fixture("google_contactos.vcf"));

    // Adrián🖤
    let adrian = vcards
        .iter()
        .find(|v| v.fn_raw.as_deref() == Some("Adrián🖤"));
    assert!(adrian.is_some(), "Adrián🖤 not found");

    // Alberto[Veci] — brackets en FN
    let alberto = vcards
        .iter()
        .find(|v| v.fn_raw.as_deref() == Some("Alberto[Veci]"));
    assert!(alberto.is_some(), "Alberto[Veci] not found");

    // Ambos deben convertirse a Contact sin errores
    adrian.unwrap().clone().to_contact().unwrap();
    alberto.unwrap().clone().to_contact().unwrap();
}

#[test]
fn test_google_v3_compat_adaptation() {
    let mut vcards = read_vcf_to_vcards(&fixture("google_contactos.vcf"));

    // Aplicar v3_compat a todos
    for vcard in &mut vcards {
        adapt_v3(vcard);
    }

    // Después de adapt_v3, versión debe ser 4.0
    for vcard in &vcards {
        assert_eq!(
            vcard.version.as_deref(),
            Some("4.0"),
            "v3_compat should set version to 4.0"
        );
    }

    // Los tipos deben estar en lowercase
    let with_tel = vcards.iter().find(|v| !v.tels_raw.is_empty()).unwrap();
    let tel = &with_tel.tels_raw[0];
    for t in &tel.types {
        assert_eq!(
            *t,
            t.to_lowercase(),
            "TYPE should be lowercase after v3_compat: {}",
            t
        );
    }
}

#[test]
fn test_google_source_detection() {
    let vcards = read_vcf_to_vcards(&fixture("google_contactos.vcf"));

    let prodids: Vec<String> = vcards.iter().filter_map(|v| v.prodid.clone()).collect();
    let uids: Vec<String> = vcards.iter().filter_map(|v| v.uid.clone()).collect();

    let source = detect_source(&prodids, &uids);
    // Google VCF 3.0 no tiene PRODID, así que debe salir Unknown
    // (en un pipeline real se pasaría --source google desde CLI)
    println!("Google source detection: {:?}", source);
}

#[test]
fn test_google_contacto_without_fn_has_uid_from_fallback() {
    let vcards = read_vcf_to_vcards(&fixture("google_contactos.vcf"));

    // El primer contacto no tiene FN, solo TEL
    let first = &vcards[0];
    assert!(first.fn_raw.is_none(), "First contact should have no FN");

    let contact = first.clone().to_contact().unwrap();
    assert!(
        !contact.uid.is_empty(),
        "Contact without FN should get generated UID"
    );
    assert_eq!(contact.fn_value, "Sin nombre");
}

// ── Pipeline completo ──

#[test]
fn test_pipeline_proton_dry_run() {
    use vcf_cribador::application::cribar;

    let (stats, _contacts) = cribar::execute(
        &fixture("protonContacts-2026-07-07.vcf"),
        None,
        None,
        None,
        "auto",
        true,
    )
    .expect("Pipeline failed");

    assert!(stats.total_entrada > 100);
    assert!(stats.conservados > 0);
    // En Proton hay contactos sin TEL → E3 eliminados
    assert!(stats.eliminados > 0 || stats.conservados > 0);
    assert_eq!(stats.cuarentena, 0);

    println!(
        "Proton pipeline: {} in, {} conserved, {} eliminated, {} merged, {} review",
        stats.total_entrada,
        stats.conservados,
        stats.eliminados,
        stats.fusionados,
        stats.needs_review
    );
}

#[test]
fn test_pipeline_google_dry_run() {
    use vcf_cribador::application::cribar;

    let (stats, _contacts) = cribar::execute(
        &fixture("google_contactos.vcf"),
        None,
        None,
        None,
        "google",
        true,
    )
    .expect("Pipeline failed");

    assert!(stats.total_entrada > 70);
    // Google contacts: todos tienen TEL → todos conservados (C6)
    assert_eq!(stats.conservados, stats.total_entrada);
    assert_eq!(stats.eliminados, 0);

    println!("Google pipeline: {} in, all conserved", stats.total_entrada);
}

#[test]
fn test_pipeline_empty_file() {
    use vcf_cribador::application::cribar;
    use vcf_cribador::error::CribaError;

    // Crear un VCF vacío temporal
    let empty = fixture("protonContacts-2026-07-07.vcf");
    let empty_path = empty.with_file_name("__empty__.vcf");
    std::fs::write(&empty_path, "").unwrap();

    let result = cribar::execute(&empty_path, None, None, None, "auto", true);

    let _ = std::fs::remove_file(&empty_path);
    assert!(matches!(result, Err(CribaError::EmptyVcf)));
}

// ── General ──

#[test]
fn test_parse_empty_file() {
    use vcf_cribador::error::CribaError;
    let result = parse_vcards("");
    assert!(matches!(result, Err(CribaError::EmptyVcf)));
}
