//! CLI con clap derive.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "vcf-cribador",
    version,
    about = "Criba, normaliza y clasifica contactos VCF vCard 4.0/3.0 (Proton, Google, Apple)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Protocolo completo: cribar → normalizar → clasificar → deduplicar
    Cribar {
        /// Archivo VCF de entrada
        input: PathBuf,

        /// Archivo VCF de salida (defecto: <input>_cribado.vcf)
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Archivo de auditoría TSV (defecto: audit.tsv)
        #[arg(short = 'a', long)]
        audit: Option<PathBuf>,

        /// Configuración TOML con reglas personalizadas
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,

        /// Forzar origen (auto|proton|google|apple)
        #[arg(short = 's', long, default_value = "auto")]
        source: String,

        /// Modo dry-run: analiza sin escribir salida
        #[arg(long)]
        dry_run: bool,
    },

    /// Solo audita: analiza el VCF sin modificarlo
    Audit {
        input: PathBuf,

        /// Archivo de auditoría TSV (defecto: audit.tsv)
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Configuración TOML
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,
    },

    /// Muestra estadísticas de un VCF (cribado o no)
    Stats {
        input: PathBuf,

        /// Formato: text, json, markdown
        #[arg(short = 'f', long, default_value = "text")]
        format: String,
    },

    /// Exporta un VCF cribado a CSV o JSON
    Export {
        input: PathBuf,

        #[arg(short = 'o', long)]
        output: PathBuf,

        /// Formato: csv, json
        #[arg(short = 'f', long, default_value = "csv")]
        format: String,
    },

    /// Genera script de autocompletado para shell
    Completions {
        /// Shell: bash, zsh, fish
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}
