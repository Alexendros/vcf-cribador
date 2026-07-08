#[derive(Debug, thiserror::Error)]
pub enum CribaError {
    #[error("error de CSV: {0}")]
    Csv(#[from] csv::Error),

    #[error("error de serialización: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("error de formato: {0}")]
    Format(#[from] std::fmt::Error),

    #[error("error de E/S: {0}")]
    Io(#[from] std::io::Error),

    #[error("error de parseo en línea {line}: {reason} (contexto: {context})")]
    Parse {
        line: usize,
        reason: String,
        context: String,
    },

    #[error("codificación no soportada: {detected}")]
    Encoding { detected: String },

    #[error("error de verificación: invariante {invariant} en contacto {uid}: {detail}")]
    VerificationFailed {
        invariant: String,
        uid: String,
        detail: String,
    },

    #[error("error de configuración en {path} línea {line}: {reason}")]
    Config {
        path: String,
        line: usize,
        reason: String,
    },

    #[error("archivo VCF vacío")]
    EmptyVcf,
}
