//! Detección de fuente (Proton/Google/Apple) y versión vCard.

use crate::domain::contact::SourceDetail;

/// Detecta la fuente del archivo VCF.
///
/// Busca en PRODID y UID de los contactos.
pub fn detect_source(prodids: &[String], uids: &[String]) -> SourceDetail {
    // Buscar en PRODID
    for prodid in prodids {
        let lower = prodid.to_lowercase();
        if lower.contains("protonmail") || lower.contains("proton ag") {
            let from_uid = detect_from_uids(uids);
            if matches!(from_uid, SourceDetail::Unknown(_)) {
                return SourceDetail::ProtonAutosave;
            }
            return from_uid;
        }
        if lower.contains("google") {
            return SourceDetail::Google;
        }
        if lower.contains("apple") {
            return SourceDetail::Apple;
        }
    }

    // Buscar en UIDs
    detect_from_uids(uids)
}

fn detect_from_uids(uids: &[String]) -> SourceDetail {
    for uid in uids {
        let lower = uid.to_lowercase();
        if lower.contains("proton-autosave") {
            return SourceDetail::ProtonAutosave;
        }
        if lower.contains("proton-import") {
            return SourceDetail::ProtonImport;
        }
        if lower.contains("proton-web") {
            return SourceDetail::ProtonWeb;
        }
        if lower.contains("proton") {
            // Cualquier UID de Proton sin sub-tipo específico
            return SourceDetail::ProtonAutosave;
        }
    }
    SourceDetail::Unknown(String::new())
}

/// Detecta la versión vCard predominante (3.0 o 4.0).
pub fn detect_version(versions: &[String]) -> &'static str {
    let mut count_4 = 0usize;
    let mut count_3 = 0usize;

    for v in versions {
        let trimmed = v.trim();
        if trimmed == "4.0" {
            count_4 += 1;
        } else if trimmed == "3.0" {
            count_3 += 1;
        }
    }

    if count_4 >= count_3 {
        "4.0"
    } else {
        "3.0"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_proton_prodid() {
        let result = detect_source(&["-//ProtonMail//ES".to_string()], &["abc-123".to_string()]);
        assert!(matches!(result, SourceDetail::ProtonAutosave));
    }

    #[test]
    fn test_detect_google_prodid() {
        let result = detect_source(&["-//Google Inc//Google Contacts//EN".to_string()], &[]);
        assert_eq!(result, SourceDetail::Google);
    }

    #[test]
    fn test_detect_apple_prodid() {
        let result = detect_source(&["-//Apple Inc.//Address Book//EN".to_string()], &[]);
        assert_eq!(result, SourceDetail::Apple);
    }

    #[test]
    fn test_detect_proton_autosave_uid() {
        let result = detect_source(&[], &["proton-autosave-abc-123".to_string()]);
        assert_eq!(result, SourceDetail::ProtonAutosave);
    }

    #[test]
    fn test_detect_proton_import_uid() {
        let result = detect_source(&[], &["proton-import-abc-123".to_string()]);
        assert_eq!(result, SourceDetail::ProtonImport);
    }

    #[test]
    fn test_detect_proton_web_uid() {
        let result = detect_source(&[], &["proton-web-abc-123".to_string()]);
        assert_eq!(result, SourceDetail::ProtonWeb);
    }

    #[test]
    fn test_detect_unknown() {
        let result = detect_source(&[], &[]);
        assert!(matches!(result, SourceDetail::Unknown(_)));
    }

    #[test]
    fn test_detect_version_4_0() {
        let result = detect_version(&["4.0".to_string(), "4.0".to_string(), "3.0".to_string()]);
        assert_eq!(result, "4.0");
    }

    #[test]
    fn test_detect_version_3_0() {
        let result = detect_version(&["3.0".to_string(), "3.0".to_string()]);
        assert_eq!(result, "3.0");
    }
}
