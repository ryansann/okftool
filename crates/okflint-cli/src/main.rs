//! `okflint` — the native CLI surface (the CI use case). All logic lives in
//! okflint-core; this binary only does filesystem IO and formatting.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use okflint_core::{build_bundle, check, rule_metas, Bundle, Diagnostic, ResolvedConfig, Severity};

#[derive(Parser)]
#[command(name = "okflint", version, about = "Validate and lint OKF bundles")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check a bundle against the OKF spec (§9 conformance only).
    Validate {
        /// Path to the bundle directory.
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = Format::Pretty)]
        format: Format,
    },
    /// Validate and lint a bundle (spec + configurable rules).
    Lint {
        /// Path to the bundle directory.
        path: PathBuf,
        /// Config file (defaults to `<path>/.okflint.yaml`, else okf-recommended).
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Pretty)]
        format: Format,
    },
    /// Explain a lint rule: its rationale, category, default severity.
    Explain {
        /// Rule id, e.g. `no-orphan-concepts`.
        rule: String,
    },
    /// List all available lint rules.
    Rules,
    /// Scaffold a starter `.okflint.yaml`.
    Init {
        /// Directory to write `.okflint.yaml` into (default: current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Copy, Clone, ValueEnum)]
enum Format {
    Pretty,
    Json,
    Sarif,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Validate { path, format } => run_check(&path, None, format, false),
        Command::Lint {
            path,
            config,
            format,
        } => run_check(&path, config, format, true),
        Command::Explain { rule } => explain(&rule),
        Command::Rules => list_rules(),
        Command::Init { path } => init(&path),
    }
}

/// Shared path for `validate` (spec only) and `lint` (spec + rules).
fn run_check(root: &Path, config: Option<PathBuf>, format: Format, with_lint: bool) -> ExitCode {
    let files = match read_bundle(root) {
        Ok(files) => files,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let (bundle, fail_on) = if with_lint {
        let cfg = match load_config(root, config.as_deref()) {
            Ok(cfg) => cfg,
            Err(err) => {
                eprintln!("error: invalid config: {err}");
                return ExitCode::FAILURE;
            }
        };
        let fail_on = cfg.fail_on;
        (check(files, &cfg), fail_on)
    } else {
        (build_bundle(files), Severity::Error)
    };

    match format {
        Format::Pretty => print_pretty(&bundle, with_lint),
        Format::Json => println!(
            "{}",
            serde_json::to_string_pretty(&bundle).unwrap_or_default()
        ),
        Format::Sarif => println!("{}", sarif(&bundle)),
    }

    // Fail on non-conformance, or any diagnostic at/above the fail-on threshold.
    let tripped = bundle
        .diagnostics
        .iter()
        .any(|d| d.severity >= fail_on && d.severity != Severity::Off);
    if !bundle.conformant || tripped {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn print_pretty(bundle: &Bundle, with_lint: bool) {
    let mut diags: Vec<&Diagnostic> = bundle.diagnostics.iter().collect();
    diags.sort_by(|a, b| a.file.cmp(&b.file).then(b.severity.cmp(&a.severity)));
    for d in diags {
        let kind = if d.spec { "spec" } else { "lint" };
        println!(
            "{}: {} [{}/{}] {}",
            d.severity.label(),
            d.file,
            kind,
            d.code,
            d.message
        );
        if let Some(fix) = &d.fix {
            println!("       ↳ {fix}");
        }
    }
    let verdict = if bundle.conformant {
        "CONFORMANT"
    } else {
        "NOT CONFORMANT"
    };
    let counts = severity_counts(bundle);
    let lint_note = if with_lint {
        format!(
            " · {} errors, {} warns, {} info",
            counts.0, counts.1, counts.2
        )
    } else {
        String::new()
    };
    println!(
        "\n{} concepts · {} diagnostics · {verdict}{lint_note}",
        bundle.concepts.len(),
        bundle.diagnostics.len()
    );
}

fn severity_counts(bundle: &Bundle) -> (usize, usize, usize) {
    let mut e = 0;
    let mut w = 0;
    let mut i = 0;
    for d in &bundle.diagnostics {
        match d.severity {
            Severity::Error => e += 1,
            Severity::Warn => w += 1,
            Severity::Info => i += 1,
            Severity::Off => {}
        }
    }
    (e, w, i)
}

/// Minimal SARIF 2.1.0 for GitHub inline annotations.
fn sarif(bundle: &Bundle) -> String {
    let rules: Vec<serde_json::Value> = rule_metas()
        .iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "name": m.id,
                "shortDescription": { "text": m.summary },
                "fullDescription": { "text": m.rationale },
                "properties": { "category": m.category.as_str() }
            })
        })
        .collect();
    let results: Vec<serde_json::Value> = bundle
        .diagnostics
        .iter()
        .filter(|d| d.severity != Severity::Off)
        .map(|d| {
            serde_json::json!({
                "ruleId": d.code,
                "level": d.severity.sarif_level(),
                "message": { "text": d.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": d.file }
                    }
                }]
            })
        })
        .collect();
    let doc = serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": { "driver": {
                "name": "okflint",
                "informationUri": "https://github.com/ryansann/okflint",
                "version": env!("CARGO_PKG_VERSION"),
                "rules": rules
            }},
            "results": results
        }]
    });
    serde_json::to_string_pretty(&doc).unwrap_or_default()
}

fn explain(rule: &str) -> ExitCode {
    match rule_metas().into_iter().find(|m| m.id == rule) {
        Some(m) => {
            println!("{}  [{}]", m.id, m.category.as_str());
            println!(
                "  default: {}{}",
                m.default_severity.label(),
                if m.fixable { " · fixable" } else { "" }
            );
            println!("  {}", m.summary);
            println!("\n  {}", m.rationale);
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("error: unknown rule `{rule}`. Run `okflint rules` to list them.");
            ExitCode::FAILURE
        }
    }
}

fn list_rules() -> ExitCode {
    let mut metas = rule_metas();
    metas.sort_by(|a, b| {
        a.category
            .as_str()
            .cmp(b.category.as_str())
            .then(a.id.cmp(b.id))
    });
    let mut current = "";
    for m in &metas {
        if m.category.as_str() != current {
            current = m.category.as_str();
            println!("\n{current}:");
        }
        let fixable = if m.fixable { " (fixable)" } else { "" };
        println!(
            "  {:<24} {:<6} {}{}",
            m.id,
            m.default_severity.label(),
            m.summary,
            fixable
        );
    }
    ExitCode::SUCCESS
}

fn init(dir: &Path) -> ExitCode {
    let target = dir.join(".okflint.yaml");
    if target.exists() {
        eprintln!("error: {} already exists", target.display());
        return ExitCode::FAILURE;
    }
    let template = "# okflint configuration — https://github.com/ryansann/okflint\nextends: okf-recommended\n\nrules: {}\n\n# overrides:\n#   - files: \"drafts/**\"\n#     rules: { \"*\": \"off\" }\n\nci:\n  fail-on: error\n";
    match std::fs::write(&target, template) {
        Ok(()) => {
            println!("wrote {}", target.display());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn load_config(root: &Path, explicit: Option<&Path>) -> Result<ResolvedConfig, String> {
    let path = explicit.map(PathBuf::from).or_else(|| {
        let candidate = root.join(".okflint.yaml");
        candidate.exists().then_some(candidate)
    });
    match path {
        Some(file) => {
            let text = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
            ResolvedConfig::from_yaml(&text)
        }
        None => Ok(ResolvedConfig::recommended()),
    }
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
