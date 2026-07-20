//! Auditoría TSV — trazabilidad completa de cada contacto procesado.

use std::fmt::Write;
use std::fs;
use std::path::Path;

use crate::domain::audit::{format_categories_tsv, AuditAction, AuditEntry};
use crate::error::CribaError;

/// Escribe el archivo de auditoría TSV con una fila por cada entrada.
pub fn write_audit_tsv(entries: &[AuditEntry], path: &Path) -> Result<(), CribaError> {
    let mut buf = String::with_capacity(entries.len() * 256);

    // Header (11 columnas requeridas)
    writeln!(
        buf,
        "TIMESTAMP\tUID\tFN_ORIGINAL\tFN_FINAL\tACCION\tMOTIVO\tREGLA\tCATEGORIAS\tSOURCE\tTELS\tEMAILS"
    )
    .unwrap();

    for entry in entries {
        let action_str = match entry.action {
            AuditAction::Conserved => "CONSERVADO",
            AuditAction::Eliminated => "ELIMINADO",
            AuditAction::Quarantine => "CUARENTENA",
            AuditAction::NeedsReview => "REVISION",
            AuditAction::Merged => "FUSIONADO",
        };

        let categorias = format_categories_tsv(&entry.categories);
        let tels = entry.tels.join(",");
        let emails = entry.emails.join(",");

        writeln!(
            buf,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            entry.timestamp,
            escape_tsv(&entry.uid),
            escape_tsv(&entry.fn_original),
            escape_tsv(&entry.fn_final),
            action_str,
            escape_tsv(&entry.reason),
            escape_tsv(&entry.rule),
            escape_tsv(&categorias),
            escape_tsv(&entry.source),
            escape_tsv(&tels),
            escape_tsv(&emails),
        )
        .unwrap();
    }

    fs::write(path, buf.as_bytes())?;
    tracing::info!(
        "Auditoría TSV escrita: {} ({} filas)",
        path.display(),
        entries.len()
    );

    Ok(())
}

fn escape_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

// ── tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audit::{AuditAction, AuditEntry};
    use crate::domain::contact::CategorySet;

    fn make_audit_entry(
        uid: &str,
        fn_original: &str,
        action: AuditAction,
        reason: &str,
        rule: &str,
    ) -> AuditEntry {
        AuditEntry {
            timestamp: "2023-01-01T00:00:00Z".into(),
            uid: uid.into(),
            fn_original: fn_original.into(),
            fn_final: fn_original.into(),
            action,
            reason: reason.into(),
            rule: rule.into(),
            categories: CategorySet::default(),
            source: "test".into(),
            tels: vec![],
            emails: vec![],
            merged_into: None,
        }
    }

    #[test]
    fn test_write_tsv_basic() {
        let entries = vec![make_audit_entry(
            "u1",
            "Juan Pérez",
            AuditAction::Conserved,
            "Conservado",
            "",
        )];

        let dir = std::env::temp_dir();
        let path = dir.join("test_tsv_basic.tsv");
        write_audit_tsv(&entries, &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("TIMESTAMP\tUID\tFN_ORIGINAL"));
        assert!(content.contains("CONSERVADO"));
        assert!(content.contains("Juan Pérez"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_write_tsv_eliminated() {
        let entries = vec![make_audit_entry(
            "e1",
            "spam@test.com",
            AuditAction::Eliminated,
            "FN es email sin identidad",
            "E1",
        )];

        let dir = std::env::temp_dir();
        let path = dir.join("test_tsv_eliminated.tsv");
        write_audit_tsv(&entries, &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("ELIMINADO"));
        assert!(content.contains("E1"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_write_tsv_columns_count() {
        let entries = vec![make_audit_entry(
            "c1",
            "Test",
            AuditAction::Conserved,
            "Conservado",
            "",
        )];

        let dir = std::env::temp_dir();
        let path = dir.join("test_tsv_columns.tsv");
        write_audit_tsv(&entries, &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2); // header + 1 data row
        assert_eq!(lines[0].split('\t').count(), 11);
        assert_eq!(lines[1].split('\t').count(), 11);

        let _ = fs::remove_file(&path);
    }
}
