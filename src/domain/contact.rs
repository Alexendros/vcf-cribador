//! Entidad canónica de dominio y value objects.
//!
//! `Contact` es la representación limpia de un contacto,
//! sin escapes RFC ni datos binarios.

use std::collections::HashSet;

/// Entidad canónica de dominio.
#[derive(Debug, Clone)]
pub struct Contact {
    pub uid: String,
    pub fn_value: String,
    pub structured_name: Option<StructuredName>,
    pub org: Option<String>,
    pub org_fullname: Option<String>,
    pub org_legal_form: Option<String>,
    pub emails: Vec<TypedValue>,
    pub tels: Vec<Tel>,
    pub title: Option<String>,
    pub role: Option<String>,
    pub note: Option<String>,
    pub categories: CategorySet,
    pub source_detail: SourceDetail,
    pub decision: super::screening::ScreeningDecision,
    pub screening_rule: String,
    pub merged_uids: Vec<String>,
}

/// Componentes estructurados del nombre (RFC 6350).
#[derive(Debug, Clone, Default)]
pub struct StructuredName {
    pub family: Vec<String>,
    pub given: Vec<String>,
    pub additional: Vec<String>,
    pub prefix: Vec<String>,
    pub suffix: Vec<String>,
}

/// Número de teléfono normalizado.
#[derive(Debug, Clone)]
pub struct Tel {
    pub value: String,
    pub tel_type: TelType,
    pub normalized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelType {
    Cell,
    Home,
    Work,
    Main,
    Other,
}

/// Valor con tipos y preferencia (EMAIL, TEL sin normalizar).
#[derive(Debug, Clone)]
pub struct TypedValue {
    pub value: String,
    pub types: Vec<String>,
    pub pref: u8,
}

/// Conjunto de categorías N1, N2, N3.
#[derive(Debug, Clone, Default)]
pub struct CategorySet {
    pub n1: HashSet<String>,
    pub n2: HashSet<String>,
    pub n3: Vec<String>,
}

impl CategorySet {
    pub fn has_n1(&self) -> bool {
        !self.n1.is_empty()
    }
}

/// Origen detallado del contacto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceDetail {
    ProtonAutosave,
    ProtonImport,
    ProtonWeb,
    Google,
    Apple,
    Unknown(String),
}
