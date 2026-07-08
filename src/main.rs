use clap::Parser;
use tracing_subscriber::EnvFilter;
use vcf_cribador::application::audit;
use vcf_cribador::application::cribar;
use vcf_cribador::application::stats;
use vcf_cribador::interfaces::cli::{Cli, Command};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("vcf_cribador=info".parse()?))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Cribar {
            input,
            output,
            audit: audit_path,
            config,
            source,
            dry_run,
        } => {
            let (stats, _contacts) = cribar::execute(
                &input,
                output.as_deref(),
                audit_path.as_deref(),
                config.as_deref(),
                &source,
                dry_run,
            )?;

            println!("{}", stats);
        }
        Command::Audit {
            input,
            output,
            config,
        } => {
            audit::execute(&input, output.as_deref(), config.as_deref(), "auto")?;
        }
        Command::Stats { input, format } => {
            stats::execute(&input, &format)?;
        }
        Command::Export {
            input,
            output,
            format,
        } => {
            let (_stats, contacts) = cribar::execute(&input, None, None, None, "auto", true)?;
            match format.as_str() {
                "json" => {
                    vcf_cribador::infrastructure::json_writer::export_json(&contacts, &output)?
                }
                _ => vcf_cribador::infrastructure::csv_writer::export_csv(&contacts, &output)?,
            }
        }
    }

    Ok(())
}
