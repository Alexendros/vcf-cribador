//! Adaptación vCard 3.0 → 4.0.
//!
//! Google Contacts y Apple iCloud exportan en vCard 3.0.
//! Este módulo normaliza propiedades de 3.0 a su equivalente en 4.0
//! y filtra propiedades obsoletas (AGENT, LABEL, MAILER).

use crate::infrastructure::parser::ParsedVCard;

/// Propiedades obsoletas que no deben propagarse a vCard 4.0.
const OBSOLETE_PROPERTIES: &[&str] = &["AGENT", "LABEL", "MAILER"];

/// Adapta un ParsedVCard de vCard 3.0 a semántica vCard 4.0.
///
/// Transformaciones:
/// 1. Normaliza TYPE a lowercase
/// 2. Filtra propiedades obsoletas (AGENT, LABEL, MAILER)
/// 3. Separa PREF de TYPE en vCard 3.0
pub fn adapt_v3(vcard: &mut ParsedVCard) {
    // 1. Filtrar propiedades obsoletas
    vcard.raw_properties.retain(|p| {
        let upper = p.name.to_uppercase();
        !OBSOLETE_PROPERTIES.contains(&upper.as_str())
    });

    // 2. Normalizar tipos a lowercase en emails_raw
    for email in &mut vcard.emails_raw {
        normalize_types_v3(&mut email.types, &mut email.pref);
    }

    // 3. Normalizar tipos a lowercase en tels_raw
    for tel in &mut vcard.tels_raw {
        normalize_types_v3(&mut tel.types, &mut tel.pref);
    }

    // 4. Normalizar TYPE en raw_properties params
    for prop in &mut vcard.raw_properties {
        for param in &mut prop.params {
            if param.name.eq_ignore_ascii_case("TYPE") {
                param.values = param.values.iter().map(|v| v.to_lowercase()).collect();
            }
        }
    }

    // 5. Versión se actualiza a 4.0 para la salida
    vcard.version = Some("4.0".to_string());
}

/// En vCard 3.0, PREF es un valor de TYPE en lugar de un parámetro separado.
/// Esta función separa "PREF" de los tipos y lo asigna al campo pref.
fn normalize_types_v3(types: &mut Vec<String>, pref: &mut u8) {
    // Convertir todos a lowercase y separar PREF
    let mut new_types = Vec::new();
    for t in types.iter() {
        let lower = t.to_lowercase();
        if lower == "pref" {
            *pref = 1;
        } else {
            new_types.push(lower);
        }
    }
    *types = new_types;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::parser::RawProperty;

    #[test]
    fn test_v3_type_lowercase() {
        let mut vcard = ParsedVCard {
            version: Some("3.0".to_string()),
            emails_raw: vec![crate::infrastructure::parser::RawTypedValue {
                value: "x@y.com".to_string(),
                types: vec!["INTERNET".to_string(), "PREF".to_string()],
                pref: 0,
            }],
            ..Default::default()
        };

        adapt_v3(&mut vcard);

        let email = &vcard.emails_raw[0];
        assert_eq!(email.types, vec!["internet"]);
        assert_eq!(email.pref, 1);
    }

    #[test]
    fn test_v3_agent_ignored() {
        let mut vcard = ParsedVCard {
            version: Some("3.0".to_string()),
            raw_properties: vec![
                RawProperty {
                    group: None,
                    name: "FN".to_string(),
                    params: vec![],
                    value: "Test".to_string(),
                },
                RawProperty {
                    group: None,
                    name: "AGENT".to_string(),
                    params: vec![],
                    value: "BEGIN:VCARD...".to_string(),
                },
                RawProperty {
                    group: None,
                    name: "LABEL".to_string(),
                    params: vec![],
                    value: "Calle Test".to_string(),
                },
                RawProperty {
                    group: None,
                    name: "MAILER".to_string(),
                    params: vec![],
                    value: "Apple Mail".to_string(),
                },
            ],
            ..Default::default()
        };

        adapt_v3(&mut vcard);

        // Solo FN debe sobrevivir
        assert_eq!(vcard.raw_properties.len(), 1);
        assert_eq!(vcard.raw_properties[0].name, "FN");
    }

    #[test]
    fn test_v3_version_updated() {
        let mut vcard = ParsedVCard {
            version: Some("3.0".to_string()),
            ..Default::default()
        };

        adapt_v3(&mut vcard);

        assert_eq!(vcard.version.as_deref(), Some("4.0"));
    }

    #[test]
    fn test_v3_type_multi_value_lowercase() {
        let mut vcard = ParsedVCard {
            version: Some("3.0".to_string()),
            tels_raw: vec![crate::infrastructure::parser::RawTypedValue {
                value: "+34600000000".to_string(),
                types: vec!["CELL".to_string(), "VOICE".to_string()],
                pref: 0,
            }],
            ..Default::default()
        };

        adapt_v3(&mut vcard);

        let tel = &vcard.tels_raw[0];
        assert_eq!(tel.types, vec!["cell", "voice"]);
        assert_eq!(tel.pref, 0);
    }
}
