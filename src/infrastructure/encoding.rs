//! Detección y transcodificación de codificación (ISO-8859-1 → UTF-8).

use chardetng::EncodingDetector;

use crate::error::CribaError;

/// Asegura que la entrada esté en UTF-8, transcodificando si es necesario.
pub fn ensure_utf8(input: &[u8]) -> Result<String, CribaError> {
    match String::from_utf8(input.to_vec()) {
        Ok(s) => Ok(s),
        Err(_) => {
            // Detectar encoding sin permitir UTF-8 (si fuera UTF-8 válido ya habría pasado)
            let mut detector = EncodingDetector::new();
            detector.feed(input, true);
            let encoding = detector.guess(None, false);

            if encoding.name() == "UTF-8" {
                // Bytes sueltos inválidos en entrada mayoritariamente UTF-8
                return Ok(String::from_utf8_lossy(input).into_owned());
            }

            let (decoded, _enc, had_errors) = encoding.decode(input);
            if had_errors {
                tracing::warn!("errores durante transcodificación de {}", encoding.name());
            }
            Ok(decoded.into_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_passthrough() {
        let input = b"BEGIN:VCARD\nFN:Juan P\xc3\xa9rez\nEND:VCARD\n";
        let result = ensure_utf8(input).unwrap();
        assert!(result.contains("Juan Pérez"));
    }

    #[test]
    fn test_iso_8859_1_to_utf8() {
        // \xe9 = 'é' en ISO-8859-1
        let input = b"BEGIN:VCARD\nFN:Juan P\xe9rez\nEND:VCARD\n";
        let result = ensure_utf8(input).unwrap();
        assert!(result.contains("Juan Pérez"));
    }

    #[test]
    fn test_invalid_utf8_replacement() {
        // \xff es un byte válido en ISO-8859-1 ('ÿ'), chardetng lo detecta y transcodifica
        let input: &[u8] = b"Hello \xff World";
        let result = ensure_utf8(input).unwrap();
        assert!(result.contains('\u{FFFD}') || result.contains('ÿ'));
    }

    #[test]
    fn test_iso_spanish_chars() {
        // ñ = \xf1 en ISO-8859-1
        let input = b"FN:Pe\xf1a\n";
        let result = ensure_utf8(input).unwrap();
        assert!(result.contains("Peña"));
    }
}
