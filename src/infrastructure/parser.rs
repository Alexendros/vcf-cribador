//! Parser vCard 4.0/3.0 con nom.
//!
//! Incluye unfold de líneas (RFC 6350 §3.2), desescapado (RFC 6350 §3.4),
//! y preservación de propiedades binarias.

use std::collections::HashSet;
use std::sync::LazyLock;

use crate::domain::contact::{
    Address, CategorySet, Contact, SourceDetail, StructuredName, Tel, TelType, TypedValue,
};
use crate::domain::screening::ScreeningDecision;
use crate::error::CribaError;

/// Representación intermedia tras parsear el VCF.
/// Contiene propiedades raw, binarios, y semántica RFC.
/// Se mapea a Contact mediante ParsedVCard::to_contact().
#[derive(Debug, Clone, Default)]
pub struct ParsedVCard {
    pub raw_properties: Vec<RawProperty>,
    pub uid: Option<String>,
    pub fn_raw: Option<String>,
    pub n_raw: Option<String>,
    pub org_raw: Option<String>,
    pub emails_raw: Vec<RawTypedValue>,
    pub tels_raw: Vec<RawTypedValue>,
    pub addresses_raw: Vec<RawTypedValue>,
    pub title_raw: Option<String>,
    pub role_raw: Option<String>,
    pub note_raw: Option<String>,
    pub rev_raw: Option<String>,
    pub photo_lines: Vec<String>,
    pub logo_lines: Vec<String>,
    pub sound_lines: Vec<String>,
    pub key_lines: Vec<String>,
    pub version: Option<String>,
    pub prodid: Option<String>,
}

/// Propiedad raw del VCF.
#[derive(Debug, Clone)]
pub struct RawProperty {
    pub group: Option<String>,
    pub name: String,
    pub params: Vec<RawParam>,
    pub value: String,
}

/// Parámetro de una propiedad.
#[derive(Debug, Clone)]
pub struct RawParam {
    pub name: String,
    pub values: Vec<String>,
}

/// Propiedad tipada sin normalizar (EMAIL, TEL raw).
#[derive(Debug, Clone, Default)]
pub struct RawTypedValue {
    pub value: String,
    pub types: Vec<String>,
    pub pref: u8,
}

impl ParsedVCard {
    pub fn compute_uid(&self) -> String {
        match &self.uid {
            Some(u) => unescape(u),
            None => generate_uid_from(self),
        }
    }

    /// Convierte al modelo de dominio. Aplica desescapado, descarta
    /// propiedades obsoletas (AGENT, LABEL, MAILER si v3), y normaliza tipos.
    pub fn to_contact(&self) -> Result<Contact, CribaError> {
        let uid = self.compute_uid();

        let fn_value = self
            .fn_raw
            .as_deref()
            .map(unescape)
            .unwrap_or_else(|| "Sin nombre".to_string());

        let structured_name = self.n_raw.as_deref().map(parse_n_structured);

        let org = self.org_raw.as_deref().map(unescape);
        let title = self.title_raw.as_deref().map(unescape);
        let role = self.role_raw.as_deref().map(unescape);
        let mut note = self.note_raw.as_deref().map(unescape);
        if let Some(rev) = self.rev_raw.as_deref() {
            let rev_note = format!("REV:{}", rev);
            note = match note {
                Some(mut n) => {
                    n.push_str(" | ");
                    n.push_str(&rev_note);
                    Some(n)
                }
                None => Some(rev_note),
            };
        }

        let emails = self
            .emails_raw
            .iter()
            .cloned()
            .map(|raw| TypedValue {
                value: unescape(&raw.value),
                types: raw.types,
                pref: raw.pref,
            })
            .collect();

        let tels = self
            .tels_raw
            .iter()
            .map(|raw| Tel {
                value: unescape(&raw.value),
                tel_type: map_tel_type(&raw.types),
                normalized: false,
            })
            .collect();

        let addresses = self
            .addresses_raw
            .iter()
            .map(|raw| parse_address(&raw.value, raw.types.clone()))
            .collect();

        Ok(Contact {
            uid,
            fn_value,
            structured_name,
            org,
            org_fullname: None,
            org_legal_form: None,
            emails,
            tels,
            addresses,
            title,
            role,
            note,
            categories: CategorySet::default(),
            source_detail: SourceDetail::Unknown(String::new()),
            decision: ScreeningDecision::Conserved,
            screening_rule: String::new(),
            merged_uids: vec![],
        })
    }
}

fn generate_uid_from(vcard: &ParsedVCard) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    vcard.fn_raw.hash(&mut hasher);
    vcard.n_raw.hash(&mut hasher);
    for e in &vcard.emails_raw {
        e.value.hash(&mut hasher);
    }
    for t in &vcard.tels_raw {
        t.value.hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

fn parse_n_structured(raw: &str) -> StructuredName {
    let parts: Vec<&str> = raw.split(';').collect();
    StructuredName {
        family: parts.first().copied().map(split_name).unwrap_or_default(),
        given: parts.get(1).copied().map(split_name).unwrap_or_default(),
        additional: parts.get(2).copied().map(split_name).unwrap_or_default(),
        prefix: parts.get(3).copied().map(split_name).unwrap_or_default(),
        suffix: parts.get(4).copied().map(split_name).unwrap_or_default(),
    }
}

fn split_name(s: &str) -> Vec<String> {
    if s.is_empty() {
        vec![]
    } else {
        s.split_whitespace().map(|w| w.to_string()).collect()
    }
}

fn parse_address(value: &str, types: Vec<String>) -> Address {
    let parts: Vec<String> = value.split(';').map(|s| s.to_string()).collect();
    let get = |idx: usize| -> String {
        parts
            .get(idx)
            .cloned()
            .unwrap_or_default()
            .split('\n')
            .next()
            .unwrap_or("")
            .to_string()
    };

    Address {
        po_box: unescape(&get(0)),
        extended: unescape(&get(1)),
        street: unescape(&get(2)),
        locality: unescape(&get(3)),
        region: unescape(&get(4)),
        postal_code: unescape(&get(5)),
        country: unescape(&get(6)),
        types,
    }
}

fn map_tel_type(types: &[String]) -> TelType {
    for t in types {
        let lower = t.to_lowercase();
        if lower == "cell" || lower == "mobile" || lower == "iphone" || lower == "car" {
            return TelType::Cell;
        }
        if lower == "home" || lower == "dom" {
            return TelType::Home;
        }
        if lower == "work" || lower == "oficina" {
            return TelType::Work;
        }
        if lower == "main" || lower == "pref" {
            return TelType::Main;
        }
        if lower == "fax" {
            return TelType::Fax;
        }
        if lower == "pager" {
            return TelType::Pager;
        }
        if lower == "text" {
            return TelType::Text;
        }
        if lower == "video" {
            return TelType::Video;
        }
    }
    TelType::Other
}

/// Propiedades que deben preservarse byte-identical.
static BINARY_PROPERTIES: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["PHOTO", "LOGO", "SOUND", "KEY"]));

fn is_binary(name: &str, value: &str) -> bool {
    BINARY_PROPERTIES.contains(name) || value.starts_with("data:")
}

/// Despliega líneas plegadas según RFC 6350 §3.2.
///
/// vCard usa terminadores CRLF. Rust's `lines()` solo corta en LF, dejando
/// el CR como parte del contenido. Debemos limpiar el CR al final de cada línea.
pub fn unfold(input: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for raw_line in input.lines() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if let Some(first_char) = line.chars().next() {
            if (first_char == ' ' || first_char == '\t') && !lines.is_empty() {
                let last = lines.last_mut().unwrap();
                last.push_str(&line[1..]);
            } else {
                lines.push(line.to_string());
            }
        }
    }
    lines.join("\n")
}

/// Desescapa una propiedad según RFC 6350 §3.4.
/// Corrección del bug v2.1: implementación secuencial sin marcadores temporales.
pub fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some(';') => out.push(';'),
                Some(',') => out.push(','),
                Some('n') | Some('N') => out.push('\n'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Parsea un string VCF (ya desplegado y transcodificado) en Vec<ParsedVCard>.
pub fn parse_vcards(input: &str) -> Result<Vec<ParsedVCard>, CribaError> {
    let lines: Vec<&str> = input.lines().collect();
    let mut vcards = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut inside = false;
    let mut line_number = 0;

    for line in &lines {
        line_number += 1;
        let trimmed = line.trim();

        if trimmed.eq_ignore_ascii_case("BEGIN:VCARD") {
            if inside {
                return Err(CribaError::Parse {
                    line: line_number,
                    reason: "BEGIN:VCARD anidado sin END:VCARD previo".into(),
                    context: trimmed.to_string(),
                });
            }
            inside = true;
            current.clear();
        } else if trimmed.eq_ignore_ascii_case("END:VCARD") {
            if !inside {
                return Err(CribaError::Parse {
                    line: line_number,
                    reason: "END:VCARD sin BEGIN:VCARD previo".into(),
                    context: trimmed.to_string(),
                });
            }
            inside = false;
            let vcard = build_parsed_vcard(&current, line_number)?;
            vcards.push(vcard);
        } else if inside && !trimmed.is_empty() {
            current.push(line);
        }
    }

    if inside {
        return Err(CribaError::Parse {
            line: line_number,
            reason: "BEGIN:VCARD sin END:VCARD al final del archivo".into(),
            context: lines.last().unwrap_or(&"").to_string(),
        });
    }

    if vcards.is_empty() {
        return Err(CribaError::EmptyVcf);
    }

    Ok(vcards)
}

/// Construye un ParsedVCard a partir de las líneas raw de un vCard.
fn build_parsed_vcard(lines: &[&str], _line_number: usize) -> Result<ParsedVCard, CribaError> {
    let mut vcard = ParsedVCard::default();
    let mut raw_props = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let prop = parse_contentline(line, i + 1)?;
        let name_upper = prop.name.to_uppercase();
        let value_noescape = prop.value.clone();

        raw_props.push(prop.clone());

        match name_upper.as_str() {
            "UID" => vcard.uid = Some(value_noescape),
            "FN" => vcard.fn_raw = Some(value_noescape),
            "N" => vcard.n_raw = Some(value_noescape),
            "ORG" => vcard.org_raw = Some(value_noescape),
            "TITLE" => vcard.title_raw = Some(value_noescape),
            "ROLE" => vcard.role_raw = Some(value_noescape),
            "NOTE" => vcard.note_raw = Some(value_noescape),
            "REV" => vcard.rev_raw = Some(value_noescape),
            "VERSION" => vcard.version = Some(value_noescape),
            "PRODID" => vcard.prodid = Some(value_noescape),
            "EMAIL" => {
                let types = extract_types(&prop.params);
                let pref = extract_pref(&prop.params);
                vcard.emails_raw.push(RawTypedValue {
                    value: value_noescape,
                    types,
                    pref,
                });
            }
            "TEL" => {
                let types = extract_types(&prop.params);
                let pref = extract_pref(&prop.params);
                vcard.tels_raw.push(RawTypedValue {
                    value: value_noescape,
                    types,
                    pref,
                });
            }
            "ADR" => {
                let types = extract_types(&prop.params);
                vcard.addresses_raw.push(RawTypedValue {
                    value: value_noescape,
                    types,
                    pref: 0,
                });
            }
            _ => {
                if is_binary(&name_upper, &prop.value) {
                    let raw_line = line.to_string();
                    match name_upper.as_str() {
                        "PHOTO" => vcard.photo_lines.push(raw_line),
                        "LOGO" => vcard.logo_lines.push(raw_line),
                        "SOUND" => vcard.sound_lines.push(raw_line),
                        "KEY" => vcard.key_lines.push(raw_line),
                        _ => {}
                    }
                }
            }
        }
    }

    vcard.raw_properties = raw_props;
    Ok(vcard)
}

fn extract_types(params: &[RawParam]) -> Vec<String> {
    for p in params {
        if p.name.eq_ignore_ascii_case("TYPE") {
            return p.values.clone();
        }
    }
    vec![]
}

fn extract_pref(params: &[RawParam]) -> u8 {
    for p in params {
        if p.name.eq_ignore_ascii_case("PREF") {
            if let Some(v) = p.values.first() {
                return v.parse::<u8>().unwrap_or(0);
            }
        }
    }
    0
}

/// Parsea una contentline vCard: [group.]name*(";"param)":"value
fn parse_contentline(line: &str, line_num: usize) -> Result<RawProperty, CribaError> {
    let colon_pos = line.find(':').ok_or_else(|| CribaError::Parse {
        line: line_num,
        reason: "falta ':' separador en propiedad".into(),
        context: line.to_string(),
    })?;

    let header = &line[..colon_pos];
    let value = &line[colon_pos + 1..];

    let (group, name, params) = parse_header(header, line_num, line)?;

    Ok(RawProperty {
        group,
        name,
        params,
        value: value.to_string(),
    })
}

/// Parsea el header de una contentline con nom.
fn parse_header(
    header: &str,
    line_num: usize,
    full_line: &str,
) -> Result<(Option<String>, String, Vec<RawParam>), CribaError> {
    match header_parser(header) {
        Ok((_, result)) => Ok(result),
        Err(e) => {
            // Intentar diagnóstico: puede ser error de nom o entrada no parseable
            let reason = match e {
                nom::Err::Error(_) | nom::Err::Failure(_) => {
                    "sintaxis inválida en cabecera de propiedad".into()
                }
                nom::Err::Incomplete(_) => "entrada inesperadamente truncada".into(),
            };
            Err(CribaError::Parse {
                line: line_num,
                reason,
                context: full_line.to_string(),
            })
        }
    }
}

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while, take_while1},
    character::complete::alpha1,
    combinator::{map, recognize},
    multi::{many0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair},
    IResult,
};

/// "GROUP.NAME" → (Some("GROUP"), "NAME")  o  "NAME" → (None, "NAME")
fn property_name(input: &str) -> IResult<&str, (Option<String>, String)> {
    alt((
        map(
            separated_pair(
                map(name_token, |n: &str| n.to_string()),
                tag("."),
                map(name_token, |n: &str| n.to_string()),
            ),
            |(g, n)| (Some(g), n),
        ),
        map(name_token, |n: &str| (None, n.to_string())),
    ))(input)
}

fn name_token(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alpha1,
        take_while(|c: char| c.is_ascii_alphanumeric() || c == '-'),
    ))(input)
}

/// Cabecera completa: property-name *(";" param) → (group, name, params)
fn header_parser(input: &str) -> IResult<&str, (Option<String>, String, Vec<RawParam>)> {
    let (input, (group, name)) = property_name(input)?;
    let (input, params) = many0(preceded(tag(";"), param_parser))(input)?;
    Ok((input, (group, name, params)))
}

/// "PARAM-NAME=value1,value2,..."
fn param_parser(input: &str) -> IResult<&str, RawParam> {
    let (input, name) = map(name_token, |n: &str| n.to_string())(input)?;
    let (input, _) = tag("=")(input)?;
    let (input, values) = separated_list1(tag(","), param_value)(input)?;
    Ok((input, RawParam { name, values }))
}

/// Valor de parámetro: "quoted" o safe-chars
fn param_value(input: &str) -> IResult<&str, String> {
    alt((
        map(delimited(tag("\""), is_not("\""), tag("\"")), |s: &str| {
            s.to_string()
        }),
        map(
            take_while1(|c: char| c != ';' && c != ',' && c != '=' && c != ':'),
            |s: &str| s.to_string(),
        ),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contact::TelType;

    #[test]
    fn test_unfold_simple() {
        // CRLF+SPACE es el indicador de plegado → ambos se eliminan (RFC 6350 §3.2)
        assert_eq!(unfold("FN:Juan\r\n Pérez"), "FN:JuanPérez");
    }

    #[test]
    fn test_unfold_tab() {
        // CRLF+TAB también es indicador de plegado válido
        assert_eq!(unfold("FN:Juan\r\n\tPérez"), "FN:JuanPérez");
    }

    #[test]
    fn test_unfold_no_continuation() {
        assert_eq!(unfold("FN:Juan\nTEL:+34"), "FN:Juan\nTEL:+34");
    }

    #[test]
    fn test_unfold_multiline_no_folding() {
        // unfold normaliza CRLF → LF como efecto colateral de lines()
        let input = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Mr. John Q. Public\nEMAIL:john@example.com\r\nEND:VCARD";
        let expected =
            "BEGIN:VCARD\nVERSION:4.0\nFN:Mr. John Q. Public\nEMAIL:john@example.com\nEND:VCARD";
        assert_eq!(unfold(input), expected);
    }

    #[test]
    fn test_unfold_space_within_content_preserved() {
        // El espacio DENTRO de una línea se preserva; solo se elimina
        // el CRLF+WSP que marcan continuación de línea.
        assert_eq!(unfold("FN:Juan Pérez\r\n érez"), "FN:Juan Pérezérez");
    }

    #[test]
    fn test_unescape_double_backslash() {
        assert_eq!(unescape("\\\\"), "\\");
    }

    #[test]
    fn test_unescape_escaped_semicolon() {
        assert_eq!(unescape("\\;"), ";");
    }

    #[test]
    fn test_unescape_escaped_comma() {
        assert_eq!(unescape("\\,"), ",");
    }

    #[test]
    fn test_unescape_escaped_newline() {
        assert_eq!(unescape("\\n"), "\n");
    }

    #[test]
    fn test_unescape_backslash_semicolon_regression() {
        // \\; debe decodificarse como \; (backslash literal + ;)
        assert_eq!(unescape("\\\\;"), "\\;");
    }

    #[test]
    fn test_unescape_mixed() {
        assert_eq!(unescape("a\\, b\\\\ c\\; d\\n e"), "a, b\\ c; d\n e");
    }

    #[test]
    fn test_unescape_no_escapes() {
        assert_eq!(unescape("plain text"), "plain text");
    }

    #[test]
    fn test_is_binary_photo() {
        assert!(is_binary("PHOTO", "base64data"));
    }

    #[test]
    fn test_is_binary_data_uri() {
        assert!(is_binary("EMAIL", "data:text/plain,hello"));
    }

    #[test]
    fn test_is_not_binary() {
        assert!(!is_binary("FN", "Juan Pérez"));
    }

    // ── Nom parser tests ──

    #[test]
    fn test_parse_single_vcard_4_0() {
        let input = "BEGIN:VCARD\nVERSION:4.0\nFN:Mr. John Q. Public\nEND:VCARD";
        let vcards = parse_vcards(input).unwrap();
        assert_eq!(vcards.len(), 1);
        let v = &vcards[0];
        assert_eq!(v.fn_raw.as_deref(), Some("Mr. John Q. Public"));
        assert_eq!(v.version.as_deref(), Some("4.0"));
    }

    #[test]
    fn test_parse_multi_vcard_4_0() {
        let input = "BEGIN:VCARD\nFN:Alice\nEND:VCARD\nBEGIN:VCARD\nFN:Bob\nEND:VCARD\nBEGIN:VCARD\nFN:Carol\nEND:VCARD";
        let vcards = parse_vcards(input).unwrap();
        assert_eq!(vcards.len(), 3);
        assert_eq!(vcards[0].fn_raw.as_deref(), Some("Alice"));
        assert_eq!(vcards[1].fn_raw.as_deref(), Some("Bob"));
        assert_eq!(vcards[2].fn_raw.as_deref(), Some("Carol"));
    }

    #[test]
    fn test_parse_vcard_3_0() {
        let input = "BEGIN:VCARD\nVERSION:3.0\nPRODID:-//Google Inc//Google Contacts//EN\nFN:Jane Doe\nN:Doe;Jane;;;\nEMAIL;TYPE=INTERNET,PREF:jane@example.com\nTEL;TYPE=CELL:+1-555-0100\nEND:VCARD";
        let vcards = parse_vcards(input).unwrap();
        assert_eq!(vcards.len(), 1);
        let v = &vcards[0];
        assert_eq!(v.version.as_deref(), Some("3.0"));
        assert!(v.prodid.as_ref().unwrap().contains("Google"));
        assert_eq!(v.fn_raw.as_deref(), Some("Jane Doe"));
        assert_eq!(v.n_raw.as_deref(), Some("Doe;Jane;;;"));
        assert_eq!(v.emails_raw.len(), 1);
        assert_eq!(v.emails_raw[0].value, "jane@example.com");
        // vCard 3.0: PREF va dentro de TYPE como valor adicional
        // v3_compat lo separará en su propio campo
        assert_eq!(v.emails_raw[0].types, vec!["INTERNET", "PREF"]);
        assert_eq!(v.tels_raw.len(), 1);
        assert_eq!(v.tels_raw[0].types, vec!["CELL"]);
    }

    #[test]
    fn test_parse_grouped_property() {
        let input = "BEGIN:VCARD\nVERSION:4.0\nFN:Test\nITEM1.EMAIL;PREF=1:x@y.com\nITEM1.EMAIL;PREF=2:z@y.com\nEND:VCARD";
        let vcards = parse_vcards(input).unwrap();
        let v = &vcards[0];
        // Buscar la propiedad agrupada en raw_properties
        let grouped = v
            .raw_properties
            .iter()
            .find(|p| p.group.as_deref() == Some("ITEM1"));
        assert!(grouped.is_some());
        let g = grouped.unwrap();
        assert_eq!(g.name, "EMAIL");
        assert_eq!(v.emails_raw.len(), 2);
    }

    #[test]
    fn test_parse_n_structured() {
        let input = "BEGIN:VCARD\nVERSION:4.0\nFN:Test\nN:Doe;John;Middle;Dr.;Jr.\nEND:VCARD";
        let vcards = parse_vcards(input).unwrap();
        let v = &vcards[0];
        assert_eq!(v.n_raw.as_deref(), Some("Doe;John;Middle;Dr.;Jr."));
    }

    #[test]
    fn test_parse_params_multi_value() {
        let input =
            "BEGIN:VCARD\nVERSION:4.0\nFN:Test\nTEL;TYPE=CELL,VOICE:+34600000000\nEND:VCARD";
        let vcards = parse_vcards(input).unwrap();
        let v = &vcards[0];
        assert_eq!(v.tels_raw.len(), 1);
        assert_eq!(v.tels_raw[0].types, vec!["CELL", "VOICE"]);
    }

    #[test]
    fn test_parse_photo_raw() {
        let input = "BEGIN:VCARD\nVERSION:4.0\nFN:Test\nPHOTO;ENCODING=b:base64photo\nEND:VCARD";
        let vcards = parse_vcards(input).unwrap();
        let v = &vcards[0];
        assert!(!v.photo_lines.is_empty());
    }

    #[test]
    fn test_parse_empty_vcf() {
        let result = parse_vcards("");
        assert!(result.is_err());
        match result {
            Err(CribaError::EmptyVcf) => {}
            other => panic!("expected EmptyVcf, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_malformed_no_colon() {
        // Propiedad sin ':' debe dar error
        let input = "BEGIN:VCARD\nVERSION:4.0\nFN:Test\nBADPROPERTY-WITHOUT-COLON\nEND:VCARD";
        let result = parse_vcards(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unclosed_vcard() {
        let input = "BEGIN:VCARD\nVERSION:4.0\nFN:Test";
        let result = parse_vcards(input);
        assert!(result.is_err());
    }

    // ── to_contact tests ──

    #[test]
    fn test_to_contact_unescapes() {
        let vcard = ParsedVCard {
            uid: Some("test-1".into()),
            fn_raw: Some("Juan\\; Pérez".into()),
            org_raw: Some("Comma\\, Inc.".into()),
            emails_raw: vec![RawTypedValue {
                value: "\\\\backslash\\\\".into(),
                types: vec![],
                pref: 1,
            }],
            version: Some("4.0".into()),
            ..Default::default()
        };
        let contact = vcard.to_contact().unwrap();
        assert_eq!(contact.uid, "test-1");
        assert_eq!(contact.fn_value, "Juan; Pérez");
        assert_eq!(contact.org.as_deref(), Some("Comma, Inc."));
        assert_eq!(contact.emails[0].value, "\\backslash\\");
    }

    #[test]
    fn test_to_contact_uid_fallback() {
        let vcard = ParsedVCard {
            uid: None,
            fn_raw: Some("Alice".into()),
            tels_raw: vec![RawTypedValue {
                value: "+34600000001".into(),
                types: vec![],
                pref: 0,
            }],
            version: Some("4.0".into()),
            ..Default::default()
        };
        let contact = vcard.to_contact().unwrap();
        assert!(!contact.uid.is_empty());
        assert_eq!(contact.fn_value, "Alice");
    }

    #[test]
    fn test_to_contact_n_structured() {
        let vcard = ParsedVCard {
            uid: Some("test-n".into()),
            fn_raw: Some("Test".into()),
            n_raw: Some("Doe;John;Middle;Dr.;Jr.".into()),
            version: Some("4.0".into()),
            ..Default::default()
        };
        let contact = vcard.to_contact().unwrap();
        let sn = contact.structured_name.unwrap();
        assert_eq!(sn.family, vec!["Doe"]);
        assert_eq!(sn.given, vec!["John"]);
        assert_eq!(sn.additional, vec!["Middle"]);
        assert_eq!(sn.prefix, vec!["Dr."]);
        assert_eq!(sn.suffix, vec!["Jr."]);
    }

    #[test]
    fn test_to_contact_tel_type_mapping() {
        let vcard = ParsedVCard {
            uid: Some("test-tel".into()),
            fn_raw: Some("Test".into()),
            tels_raw: vec![RawTypedValue {
                value: "+34".into(),
                types: vec!["CELL".into()],
                pref: 0,
            }],
            version: Some("4.0".into()),
            ..Default::default()
        };
        let contact = vcard.to_contact().unwrap();
        assert_eq!(contact.tels[0].tel_type, TelType::Cell);
        assert!(!contact.tels[0].normalized);
    }
}
