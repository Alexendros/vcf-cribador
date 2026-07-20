//! Configuración externa desde TOML.

use std::path::Path;

use crate::domain::rules::ClassificationRule;
use crate::domain::screening::ScreeningConfig;
use crate::error::CribaError;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
struct ConfigToml {
    cribado: Option<CribadoToml>,
    clasificacion: Option<ClasificacionToml>,
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

#[derive(Debug, Deserialize, Default)]
struct ClasificacionToml {
    #[serde(default)]
    replace: bool,
    #[serde(default)]
    reglas: Vec<ReglaToml>,
}

#[derive(Debug, Deserialize)]
struct ReglaToml {
    regex: String,
    n1: String,
    n2: String,
    n3: String,
}

/// Configuración agregada de la aplicación: cribado + reglas de clasificación.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub screening: ScreeningConfig,
    pub classification_rules: Vec<ClassificationRule>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            screening: ScreeningConfig::default(),
            classification_rules: crate::domain::rules::CLASSIFICATION_RULES.clone(),
        }
    }
}

/// Carga `AppConfig` desde un archivo TOML.
/// Si `path` es None, se usa la configuración por defecto.
/// Las listas se añaden a las del default a menos que `replace = true`.
pub fn load_config(path: Option<&Path>) -> Result<AppConfig, CribaError> {
    let Some(path) = path else {
        return Ok(AppConfig::default());
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

    let mut classification_rules: Vec<ClassificationRule> =
        crate::domain::rules::CLASSIFICATION_RULES.clone();

    if let Some(clasificacion) = config.clasificacion {
        let mut custom_rules: Vec<ClassificationRule> = Vec::new();
        for r in clasificacion.reglas {
            let pattern = regex::Regex::new(&r.regex).map_err(|e| CribaError::Config {
                path: path.display().to_string(),
                line: 0,
                reason: format!("regex inválida '{}': {}", r.regex, e),
            })?;
            custom_rules.push(ClassificationRule {
                pattern,
                n1: r.n1,
                n2: r.n2,
                n3: r.n3,
            });
        }

        if clasificacion.replace {
            classification_rules = custom_rules;
        } else {
            classification_rules.extend(custom_rules);
        }
    }

    Ok(AppConfig {
        screening,
        classification_rules,
    })
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
        assert_eq!(config.screening.prefijo_pais, "+34");
        assert!(!config.screening.e2_keywords.is_empty());
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
        assert!(config
            .screening
            .conservar_dominios
            .contains(&"@gva.es".to_string()));
        assert!(config.screening.e2_keywords.contains(&"spam".to_string()));

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
        assert_eq!(config.screening.prefijo_pais, "+44");
        assert_eq!(
            config.screening.conservar_dominios,
            vec!["@example.com".to_string()]
        );
        assert_eq!(config.screening.e2_keywords, vec!["bad".to_string()]);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_load_toml_classification_append() {
        let toml_content = r#"[clasificacion]
[[clasificacion.reglas]]
regex = "(?i)rob[óo]tica|maker"
n1 = "TEC"
n2 = "TEC-HW"
n3 = "HW-MAKER"
"#;
        let path = write_toml("class_append", toml_content);

        let config = load_config(Some(&path)).unwrap();
        let custom = config
            .classification_rules
            .iter()
            .find(|r| r.n2 == "TEC-HW" && r.n3 == "HW-MAKER");
        assert!(
            custom.is_some(),
            "custom classification rule should be appended"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_load_toml_classification_replace() {
        let toml_content = r#"[clasificacion]
replace = true
[[clasificacion.reglas]]
regex = "(?i)fijo"
n1 = "PERS"
n2 = "PERS-FIX"
n3 = "FIXED"
"#;
        let path = write_toml("class_replace", toml_content);

        let config = load_config(Some(&path)).unwrap();
        assert_eq!(config.classification_rules.len(), 1);
        assert_eq!(config.classification_rules[0].n2, "PERS-FIX");

        let _ = std::fs::remove_file(&path);
    }
}
