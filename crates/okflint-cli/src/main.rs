//! `okflint` — the native CLI surface. Reads a bundle from the filesystem and
//! runs okflint-core. (Lint subcommands + SARIF land in Phase 3.)

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use okflint_core::{validate_files, Bundle, Severity};

#[derive(Parser)]
#[command(name = "okflint", version, about = "Validate and lint OKF bundles")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check a bundle against the OKF spec (§9 conformance).
    Validate {
        /// Path to the bundle directory.
        path: PathBuf,
        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Pretty)]
        format: Format,
    },
}

#[derive(Copy, Clone, ValueEnum)]
enum Format {
    Pretty,
    Json,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Validate { path, format } => {
            let files = match read_bundle(&path) {
                Ok(files) => files,
                Err(err) => {
                    eprintln!("error: {err}");
                    return ExitCode::FAILURE;
                }
            };
            let bundle = validate_files(files);
            match format {
                Format::Json => println!(
                    "{}",
                    serde_json::to_string_pretty(&bundle).unwrap_or_default()
                ),
                Format::Pretty => print_pretty(&bundle),
            }
            if bundle.conformant {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}

fn print_pretty(bundle: &Bundle) {
    for d in &bundle.diagnostics {
        let label = match d.severity {
            Severity::Error => "error",
            Severity::Warn => "warn",
            Severity::Info => "info",
            Severity::Off => "off",
        };
        println!("{label}: {}: {} — {}", d.file, d.code, d.message);
        if let Some(fix) = &d.fix {
            println!("       ↳ {fix}");
        }
    }
    let verdict = if bundle.conformant {
        "CONFORMANT"
    } else {
        "NOT CONFORMANT"
    };
    println!(
        "\n{} concepts · {} diagnostics · {verdict}",
        bundle.concepts.len(),
        bundle.diagnostics.len()
    );
}

/// Recursively collect `(bundle-relative-path, content)` for every `.md` file.
fn read_bundle(root: &Path) -> std::io::Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    collect(root, root, &mut out)?;
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(root, &path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let content = std::fs::read_to_string(&path)?;
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push((rel, content));
        }
    }
    Ok(())
}
