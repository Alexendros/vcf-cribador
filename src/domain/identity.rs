//! Detección de duplicados con Union-Find (DSU).
//!
//! Implementa cierre transitivo: si A coincide con B por TEL
//! y A coincide con C por EMAIL, entonces A, B y C se fusionan.

use crate::domain::contact::Contact;
use std::collections::HashMap;

struct Dsu {
    parent: Vec<usize>,
}

impl Dsu {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra != rb {
            self.parent[rb] = ra;
        }
    }
}

/// Normaliza FN para comparación de duplicados (lowercase, sin tildes, sin espacios extra).
fn normalize_for_dedup(fn_val: &str) -> String {
    fn_val
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' => 'u',
            'ü' => 'u',
            'ñ' => 'n',
            other => other,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Deduplica contactos usando Union-Find para D1 y D2.
///
/// Retorna los contactos fusionados y el número de fusiones realizadas.
/// La materialización usa `Vec<Option<Contact>>` con índices descendentes
/// para evitar invalidaciones.
pub fn deduplicate(contacts: Vec<Contact>) -> (Vec<Contact>, usize) {
    let n = contacts.len();
    let mut dsu = Dsu::new(n);

    // ── D1: mismo UID ──
    let mut uid_to_idx: HashMap<String, usize> = HashMap::new();
    for (i, c) in contacts.iter().enumerate() {
        let uid = c.uid.clone();
        if let Some(&prev) = uid_to_idx.get(&uid) {
            dsu.union(prev, i);
        } else {
            uid_to_idx.insert(uid, i);
        }
    }

    // ── D2: mismo FN + TEL  o  mismo FN + EMAIL ──
    let mut fn_tel_to_idx: HashMap<(String, String), usize> = HashMap::new();
    let mut fn_email_to_idx: HashMap<(String, String), usize> = HashMap::new();

    for (i, c) in contacts.iter().enumerate() {
        let fn_norm = normalize_for_dedup(&c.fn_value);

        for tel in &c.tels {
            if tel.normalized {
                let key = (fn_norm.clone(), tel.value.clone());
                if let Some(&prev) = fn_tel_to_idx.get(&key) {
                    dsu.union(prev, i);
                } else {
                    fn_tel_to_idx.insert(key, i);
                }
            }
        }

        for email in &c.emails {
            let key = (fn_norm.clone(), email.value.to_lowercase());
            if let Some(&prev) = fn_email_to_idx.get(&key) {
                dsu.union(prev, i);
            } else {
                fn_email_to_idx.insert(key, i);
            }
        }
    }

    // ── Materializar fusión: agrupar por raíz ──
    let mut groups: Vec<Vec<usize>> = vec![Vec::new(); n];
    for i in 0..n {
        groups[dsu.find(i)].push(i);
    }

    let mut merged: Vec<Option<Contact>> = contacts.into_iter().map(Some).collect();
    let mut fusion_count = 0;

    for indices in &groups {
        if indices.len() <= 1 {
            continue;
        }
        // Ordenar descendentemente: fusionar del último al primero
        let mut sorted = indices.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));

        let base_idx = sorted.pop().unwrap();
        for &other_idx in &sorted {
            let other = merged[other_idx].take().expect("contacto ya absorbido");
            if let Some(ref mut base) = merged[base_idx] {
                merge_contacts(base, other);
                fusion_count += 1;
            }
        }
    }

    let mut result: Vec<Contact> = merged.into_iter().flatten().collect();

    // ── D3-D6: candidatos a duplicado (notas, no fusión automática) ──
    detect_candidates(&mut result);

    (result, fusion_count)
}

/// Detecta candidatos D3-D6 y añade propuestas en el campo NOTE.
///
/// - D3: mismo FN + mismo ORG
/// - D4: mismo TEL (sin requerir FN)
/// - D5: mismo EMAIL (sin requerir FN)
/// - D6: FN difusa (normalizada) + TEL/EMAIL compartido
fn detect_candidates(contacts: &mut [Contact]) {
    let n = contacts.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let a = &contacts[i];
            let b = &contacts[j];

            let fn_a = normalize_for_dedup(&a.fn_value);
            let fn_b = normalize_for_dedup(&b.fn_value);
            let org_a = a.org.as_deref().unwrap_or("").to_lowercase();
            let org_b = b.org.as_deref().unwrap_or("").to_lowercase();

            let same_fn = fn_a == fn_b;
            let same_org = !org_a.is_empty() && org_a == org_b;
            let shared_tel = a
                .tels
                .iter()
                .any(|ta| b.tels.iter().any(|tb| ta.value == tb.value));
            let shared_email = a.emails.iter().any(|ea| {
                b.emails
                    .iter()
                    .any(|eb| ea.value.to_lowercase() == eb.value.to_lowercase())
            });

            let mut rule: Option<&str> = None;

            if same_fn && same_org {
                rule = Some("D3");
            } else if shared_tel && !same_fn {
                rule = Some("D4");
            } else if shared_email && !same_fn {
                rule = Some("D5");
            } else if (shared_tel || shared_email) && !same_fn {
                // FN difusa: comparten TEL/EMAIL pero no exactamente FN
                rule = Some("D6");
            }

            if let Some(r) = rule {
                let proposal_a = format!("{} candidate: uid={} fn={}", r, b.uid, b.fn_value);
                let proposal_b = format!("{} candidate: uid={} fn={}", r, a.uid, a.fn_value);
                append_note(&mut contacts[i], &proposal_a);
                append_note(&mut contacts[j], &proposal_b);
            }
        }
    }
}

fn append_note(contact: &mut Contact, text: &str) {
    match contact.note {
        Some(ref mut note) => {
            note.push_str(" | ");
            note.push_str(text);
        }
        None => contact.note = Some(text.into()),
    }
}

fn format_address_key(addr: &crate::domain::contact::Address) -> String {
    format!(
        "{};{};{};{};{};{};{};{}",
        addr.po_box,
        addr.extended,
        addr.street,
        addr.locality,
        addr.region,
        addr.postal_code,
        addr.country,
        addr.types.join(",")
    )
}

fn merge_contacts(base: &mut Contact, other: Contact) {
    base.merged_uids.push(other.uid);

    // FN: el más largo en tokens
    if other.fn_value.split_whitespace().count() > base.fn_value.split_whitespace().count() {
        base.fn_value = other.fn_value;
    }
    // structured_name: el que lo tenga
    if base.structured_name.is_none() && other.structured_name.is_some() {
        base.structured_name = other.structured_name;
    }
    // ORG: el más específico (más tokens)
    if let Some(ref other_org) = other.org {
        if base
            .org
            .as_ref()
            .map(|o| other_org.split_whitespace().count() > o.split_whitespace().count())
            .unwrap_or(true)
        {
            base.org = Some(other_org.clone());
        }
    }
    // TELs: unión dedup
    let mut seen_tels: std::collections::HashSet<String> =
        base.tels.iter().map(|t| t.value.clone()).collect();
    for tel in other.tels {
        if seen_tels.insert(tel.value.clone()) {
            base.tels.push(tel);
        }
    }
    // EMAILs: unión dedup
    let mut seen_emails: std::collections::HashSet<String> =
        base.emails.iter().map(|e| e.value.to_lowercase()).collect();
    for email in other.emails {
        if seen_emails.insert(email.value.to_lowercase()) {
            base.emails.push(email);
        }
    }
    // ADRs: unión dedup por valor completo
    let mut seen_adrs: std::collections::HashSet<String> =
        base.addresses.iter().map(format_address_key).collect();
    for addr in other.addresses {
        if seen_adrs.insert(format_address_key(&addr)) {
            base.addresses.push(addr);
        }
    }
    // TITLE, ROLE: unión
    if base.title.is_none() {
        base.title = other.title;
    }
    if base.role.is_none() {
        base.role = other.role;
    }
    // NOTE: concatenar
    if let Some(ref other_note) = other.note {
        match base.note {
            Some(ref mut bnote) => {
                bnote.push_str(" | ");
                bnote.push_str(other_note);
            }
            None => base.note = Some(other_note.clone()),
        }
    }
    // CATEGORIES: unión
    for n1 in other.categories.n1 {
        base.categories.n1.insert(n1);
    }
    for n2 in other.categories.n2 {
        base.categories.n2.insert(n2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contact::{CategorySet, SourceDetail, Tel, TelType, TypedValue};
    use crate::domain::screening::ScreeningDecision;

    fn make_contact(uid: &str, fn_val: &str, email: &str, tel_val: &str) -> Contact {
        Contact {
            uid: uid.into(),
            fn_value: fn_val.into(),
            emails: vec![TypedValue {
                value: email.into(),
                types: vec![],
                pref: 1,
            }],
            tels: if tel_val.is_empty() {
                vec![]
            } else {
                vec![Tel {
                    value: tel_val.into(),
                    tel_type: TelType::Cell,
                    normalized: true,
                }]
            },
            structured_name: None,
            org: None,
            org_fullname: None,
            org_legal_form: None,
            title: None,
            role: None,
            note: None,
            addresses: vec![],
            categories: CategorySet::default(),
            source_detail: SourceDetail::Unknown("test".into()),
            decision: ScreeningDecision::Conserved,
            screening_rule: String::new(),
            merged_uids: vec![],
        }
    }

    #[test]
    fn test_d1_same_uid() {
        let a = make_contact("u1", "Alice", "a@x.com", "");
        let b = make_contact("u1", "Alice", "a2@x.com", "");
        let (result, count) = deduplicate(vec![a, b]);
        assert_eq!(result.len(), 1);
        assert_eq!(count, 1);
        assert_eq!(result[0].merged_uids.len(), 1);
    }

    #[test]
    fn test_d2_same_fn_tel() {
        let a = make_contact("u1", "Alice", "a@x.com", "+34600000001");
        let b = make_contact("u2", "Alice", "a2@x.com", "+34600000001");
        let (result, count) = deduplicate(vec![a, b]);
        assert_eq!(result.len(), 1);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_d2_same_fn_email() {
        let a = make_contact("u1", "Alice", "a@x.com", "+34600000001");
        let b = make_contact("u2", "Alice", "a@x.com", "+34600000002");
        let (result, count) = deduplicate(vec![a, b]);
        assert_eq!(result.len(), 1);
        assert_eq!(count, 1);
        // emails duplicados no se duplican
        assert_eq!(result[0].emails.len(), 1);
    }

    #[test]
    fn test_d2_transitive() {
        // A ↔ B por TEL, A ↔ C por EMAIL → los tres se fusionan
        let a = make_contact("u1", "Alice", "shared@x.com", "+34600000001");
        let b = make_contact("u2", "Alice", "b@x.com", "+34600000001");
        let c = make_contact("u3", "Alice", "shared@x.com", "+34600000003");
        let (result, count) = deduplicate(vec![a, b, c]);
        assert_eq!(result.len(), 1);
        assert_eq!(count, 2);
        assert_eq!(result[0].merged_uids.len(), 2);
    }

    #[test]
    fn test_no_duplicates() {
        let a = make_contact("u1", "Alice", "a@x.com", "+34600000001");
        let b = make_contact("u2", "Bob", "b@x.com", "+34600000002");
        let c = make_contact("u3", "Carol", "c@x.com", "+34600000003");
        let (result, count) = deduplicate(vec![a, b, c]);
        assert_eq!(result.len(), 3);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_normalize_for_dedup_tildes() {
        assert_eq!(normalize_for_dedup("María José"), "maria jose");
        assert_eq!(normalize_for_dedup("GARCÍA  López"), "garcia lopez");
    }
}
