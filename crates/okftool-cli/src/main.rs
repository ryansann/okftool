//! `okftool` — the native CLI surface (the CI use case). All logic lives in
//! okftool-core; this binary only does filesystem IO and formatting.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use okftool_core::{
    build_bundle, check, rule_descriptors, rule_meta, rule_metas, Bundle, Diagnostic,
    ResolvedConfig, Severity,
};

#[derive(Parser)]
#[command(
    name = "okftool",
    version,
    about = "Validate, lint, and package OKF bundles"
)]
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
        /// Config file (defaults to `<path>/.okftool.yaml`, else okf-recommended).
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = Format::Pretty)]
        format: Format,
    },
    /// Explain a lint rule: its rationale, category, default severity.
    Explain {
        /// Rule id, e.g. `topology/no-orphan-concepts` (old flat aliases also work).
        rule: String,
    },
    /// List all available lint rules.
    Rules,
    /// Scaffold a starter `.okftool.yaml`.
    Init {
        /// Directory to write `.okftool.yaml` into (default: current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Package a bundle directory into a `.tar.gz` for distribution.
    Build {
        /// Path to the bundle directory.
        path: PathBuf,
        /// Output archive path (default: `<bundle>.tar.gz`).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Top-level directory inside the archive (default: the output file stem).
        #[arg(long)]
        prefix: Option<String>,
        /// Package even if the bundle is not spec-conformant.
        #[arg(long)]
        no_validate: bool,
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
        Command::Build {
            path,
            output,
            prefix,
            no_validate,
        } => build(&path, output, prefix, no_validate),
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
                "name": m.name(),
                "shortDescription": { "text": m.summary },
                "fullDescription": { "text": m.rationale },
                "properties": {
                    "category": m.category.as_str(),
                    "categoryName": m.category.name(),
                    "aliases": m.aliases(),
                    "docsPath": m.docs_path()
                }
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
                "name": "okftool",
                "informationUri": "https://github.com/ryansann/okftool",
                "version": env!("CARGO_PKG_VERSION"),
                "rules": rules
            }},
            "results": results
        }]
    });
    serde_json::to_string_pretty(&doc).unwrap_or_default()
}

fn explain(rule: &str) -> ExitCode {
    match rule_meta(rule) {
        Some(m) => {
            println!("{}  [{}]", m.id, m.category.as_str());
            let aliases = m.aliases();
            if !aliases.is_empty() {
                println!("  aliases: {}", aliases.join(", "));
            }
            println!(
                "  default: {}{}",
                m.default_severity.label(),
                if m.fixable { " · fixable" } else { "" }
            );
            println!("  {}", m.summary);
            println!("\n  {}", m.rationale);
            println!("\n  docs: {}", m.docs_path());
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("error: unknown rule `{rule}`. Run `okftool rules` to list them.");
            ExitCode::FAILURE
        }
    }
}

fn list_rules() -> ExitCode {
    let mut rules = rule_descriptors();
    rules.sort_by(|a, b| a.category.id.cmp(b.category.id).then(a.id.cmp(b.id)));
    let mut current = "";
    for m in &rules {
        if m.category.id != current {
            current = m.category.id;
            println!("\n{}:", m.category.name);
        }
        let fixable = if m.fixable { " (fixable)" } else { "" };
        println!(
            "  {:<24} {:<6} {}{}",
            m.id, m.default_severity, m.summary, fixable
        );
    }
    ExitCode::SUCCESS
}

fn init(dir: &Path) -> ExitCode {
    let target = dir.join(".okftool.yaml");
    if target.exists() {
        eprintln!("error: {} already exists", target.display());
        return ExitCode::FAILURE;
    }
    let template = "# okftool configuration — https://github.com/ryansann/okftool\nextends: okf-recommended\n\nrules: {}\n\n# overrides:\n#   - files: \"drafts/**\"\n#     rules: { \"*\": \"off\" }\n\nci:\n  fail-on: error\n";
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

/// Package a bundle directory into a gzipped tarball.
fn build(
    root: &Path,
    output: Option<PathBuf>,
    prefix: Option<String>,
    no_validate: bool,
) -> ExitCode {
    if !root.is_dir() {
        eprintln!("error: {} is not a directory", root.display());
        return ExitCode::FAILURE;
    }

    // Conformance gate (only the `.md` files matter for §9).
    if !no_validate {
        let md = match read_bundle(root) {
            Ok(md) => md,
            Err(err) => {
                eprintln!("error: {err}");
                return ExitCode::FAILURE;
            }
        };
        let bundle = build_bundle(md);
        if !bundle.conformant {
            eprintln!("error: bundle is not OKF-conformant; refusing to package (pass --no-validate to override)");
            for d in bundle.diagnostics.iter().filter(|d| d.spec) {
                eprintln!("  {}: {} — {}", d.file, d.code, d.message);
            }
            return ExitCode::FAILURE;
        }
    }

    let output = output.unwrap_or_else(|| {
        let stem = root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("bundle");
        PathBuf::from(format!("{stem}.tar.gz"))
    });
    let prefix = prefix.unwrap_or_else(|| archive_stem(&output));

    // Resolve the output absolutely so we never package the archive into itself.
    let out_abs = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(
            || std::env::current_dir().ok(),
            |p| std::fs::canonicalize(p).ok(),
        )
        .map(|dir| dir.join(output.file_name().unwrap_or_default()));

    let files = match collect_all_files(root) {
        Ok(files) => files,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    match write_targz(&output, &prefix, &files, out_abs.as_deref()) {
        Ok(count) => {
            println!("wrote {} ({count} files)", output.display());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Strip a `.tar.gz`/`.tgz` suffix to get the archive's top-level dir name.
fn archive_stem(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("bundle");
    name.strip_suffix(".tar.gz")
        .or_else(|| name.strip_suffix(".tgz"))
        .unwrap_or(name)
        .to_string()
}

/// Every file under `root` (not just `.md`), skipping `.git`, sorted for
/// reproducibility.
fn collect_all_files(root: &Path) -> std::io::Result<Vec<(String, PathBuf)>> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                if path.file_name().and_then(|s| s.to_str()) == Some(".git") {
                    continue;
                }
                walk(root, &path, out)?;
            } else {
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((rel, path));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(root, root, &mut out)?;
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

fn write_targz(
    output: &Path,
    prefix: &str,
    files: &[(String, PathBuf)],
    skip: Option<&Path>,
) -> std::io::Result<usize> {
    let gz = flate2::write::GzEncoder::new(
        std::fs::File::create(output)?,
        flate2::Compression::default(),
    );
    let mut builder = tar::Builder::new(gz);
    let mut count = 0;
    for (rel, abs) in files {
        if let (Some(skip), Ok(canon)) = (skip, std::fs::canonicalize(abs)) {
            if canon == skip {
                continue; // don't archive the output into itself
            }
        }
        let data = std::fs::read(abs)?;
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(0); // fixed mtime → reproducible archives
        let name = if prefix.is_empty() {
            rel.clone()
        } else {
            format!("{prefix}/{rel}")
        };
        builder.append_data(&mut header, name, &data[..])?;
        count += 1;
    }
    builder.into_inner()?.finish()?;
    Ok(count)
}

fn load_config(root: &Path, explicit: Option<&Path>) -> Result<ResolvedConfig, String> {
    let path = explicit.map(PathBuf::from).or_else(|| {
        // Accept either extension at the bundle root.
        [".okftool.yaml", ".okftool.yml"]
            .iter()
            .map(|name| root.join(name))
            .find(|p| p.exists())
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
