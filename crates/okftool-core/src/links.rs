//! Pure markdown link helpers — ported from okfview's `links.ts` to keep
//! okftool's graph identical to the app's.

use std::sync::OnceLock;

use regex::Regex;

/// A raw markdown link before bundle resolution.
#[derive(Debug, Clone)]
pub struct RawLink {
    pub href: String,
    pub text: String,
}

fn image_or_link_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // (!?)[text](href "optional title")
        Regex::new(r#"(!?)\[([^\]]*)\]\(\s*([^)\s]+)(?:\s+["'][^"']*["'])?\s*\)"#).unwrap()
    })
}

fn autolink_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"<((?:https?|mailto):[^>]+)>").unwrap())
}

/// Extract markdown links from a body, skipping images.
pub fn extract_links(markdown: &str) -> Vec<RawLink> {
    let mut links = Vec::new();
    for caps in image_or_link_re().captures_iter(markdown) {
        if &caps[1] == "!" {
            continue; // image, not a link
        }
        links.push(RawLink {
            text: caps[2].to_string(),
            href: caps[3].to_string(),
        });
    }
    for caps in autolink_re().captures_iter(markdown) {
        let url = caps[1].to_string();
        links.push(RawLink {
            text: url.clone(),
            href: url,
        });
    }
    links
}

/// True for `scheme://…` and `mailto:` hrefs.
pub fn is_external(href: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(?i)^[a-z][a-z0-9+.-]*://").unwrap());
    re.is_match(href) || href.starts_with("mailto:")
}

/// Normalize a relative path against a base directory, resolving `.`/`..`.
fn normalize(base_dir: &str, rel: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    let parts = base_dir
        .split('/')
        .filter(|_| !base_dir.is_empty())
        .chain(rel.split('/'));
    for p in parts {
        match p {
            "" | "." => continue,
            ".." => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out.join("/")
}

/// Outcome of resolving an href as seen from a concept.
#[derive(Debug, Default, Clone)]
pub struct ResolvedTarget {
    pub external: Option<String>,
    /// In-bundle target concept id (`.md` stripped).
    pub target_path: Option<String>,
    /// `index.md`, `subdir/`, and bare `#anchor` links — never "broken".
    pub directory_or_anchor: bool,
}

/// Strip a trailing `.md`/`.MD` extension.
fn strip_md(path: &str) -> &str {
    if path.len() >= 3 && path[path.len() - 3..].eq_ignore_ascii_case(".md") {
        &path[..path.len() - 3]
    } else {
        path
    }
}

/// Resolve a markdown href as seen from `from_id` (a concept id, no `.md`).
pub fn resolve_target(from_id: &str, href: &str) -> ResolvedTarget {
    if is_external(href) {
        return ResolvedTarget {
            external: Some(href.to_string()),
            ..Default::default()
        };
    }

    let path_part = match href.find('#') {
        Some(i) => &href[..i],
        None => href,
    };
    if path_part.is_empty() {
        // pure `#anchor`
        return ResolvedTarget {
            directory_or_anchor: true,
            ..Default::default()
        };
    }
    if path_part.ends_with('/') {
        // directory link (progressive disclosure), not a concept
        return ResolvedTarget {
            directory_or_anchor: true,
            ..Default::default()
        };
    }

    let base_dir = match from_id.rfind('/') {
        Some(i) => &from_id[..i],
        None => "",
    };
    let abs = if let Some(rest) = path_part.strip_prefix('/') {
        rest.to_string()
    } else {
        normalize(base_dir, path_part)
    };
    let stripped = strip_md(&abs).to_string();

    // Links to an index file point at a directory, not a concept.
    if stripped == "index" || stripped.ends_with("/index") {
        return ResolvedTarget {
            directory_or_anchor: true,
            ..Default::default()
        };
    }

    ResolvedTarget {
        target_path: Some(stripped),
        ..Default::default()
    }
}
