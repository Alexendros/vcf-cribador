//! Configuración externa desde TOML.

use std::path::Path;

use crate::domain::screening::ScreeningConfig;
use crate::error::CribaError;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
struct ConfigToml {
    cribado: Option<CribadoToml>,
}

#[derive(Debug, Deserialize)]
struct CribadoToml {
    #[serde(default)]
    replace: bool,
    #[serde(default)]
    prefijo_pais: Option<String>,
    #[serde(default)]
    conservar_dominios: Vec<String>,
    #[serde(default)]
    servicios_descontinuados: Vec<String>,
    #[serde(default)]
    e2_keywords: Vec<String>,
}

/// Carga `ScreeningConfig` desde un archivo TOML.
/// Si `path` es None, se usa la configuración por defecto.
/// Las palabras clave se añaden a las del default a menos que `replace = true`.
pub fn load_config(path: Option<&Path>) -> Result<ScreeningConfig, CribaError> {
    let Some(path) = path else {
        return Ok(ScreeningConfig::default());
    };

    let content = std::fs::read_to_string(path)?;
    let config: ConfigToml = toml::from_str(&content).map_err(|e| CribaError::Config {
        path: path.display().to_string(),
        line: 0,
        reason: e.to_string(),
    })?;

    let mut screening = ScreeningConfig::default();

    if let Some(cribado) = config.cribado {
        if let Some(prefijo) = cribado.prefijo_pais {
            screening.prefijo_pais = prefijo;
        }

        if cribado.replace {
            screening.conservar_dominios = cribado.conservar_dominios;
            screening.servicios_descontinuados = cribado.servicios_descontinuados;
            screening.e2_keywords = cribado.e2_keywords;
        } else {
            screening
                .conservar_dominios
                .extend(cribado.conservar_dominios);
            screening
                .servicios_descontinuados
                .extend(cribado.servicios_descontinuados);
            screening.e2_keywords.extend(cribado.e2_keywords);
        }
    }

    Ok(screening)
}

// ── tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_toml(suffix: &str, content: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("test_config_{}.toml", suffix));
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_load_default() {
        let config = load_config(None).unwrap();
        assert_eq!(config.prefijo_pais, "+34");
        assert!(!config.e2_keywords.is_empty());
    }

    #[test]
    fn test_load_toml_append() {
        let toml_content = r#"[cribado]
replace = false
conservar_dominios = ["@gva.es"]
e2_keywords = ["spam", "basura"]
"#;
        let path = write_toml("append", toml_content);

        let config = load_config(Some(&path)).unwrap();
        assert!(config.conservar_dominios.contains(&"@gva.es".to_string()));
        assert!(config.e2_keywords.contains(&"spam".to_string()));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_load_toml_replace() {
        let path = write_toml(
            "replace",
            r#"[cribado]
replace = true
prefijo_pais = "+44"
conservar_dominios = ["@example.com"]
e2_keywords = ["bad"]
"#,
        );

        let config = load_config(Some(&path)).unwrap();
        assert_eq!(config.prefijo_pais, "+44");
        assert_eq!(config.conservar_dominios, vec!["@example.com".to_string()]);
        assert_eq!(config.e2_keywords, vec!["bad".to_string()]);

        let _ = std::fs::remove_file(&path);
    }
}
