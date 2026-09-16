use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use url::Url;

pub const MAX_ICON_BYTES: u64 = 1024 * 1024;
const MAX_PAGE_BYTES: u64 = 2 * 1024 * 1024;
/// Sidebar icons render at 28px; around 128px stays sharp on high-DPI screens.
const TARGET_ICON_SIZE: u32 = 128;
/// Some sites (and bot protection) reject non-browser user agents.
const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36";

/// An icon declared by a page's `<link>` tags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconCandidate {
    pub url: Url,
    /// Largest edge from the `sizes` attribute, if any.
    pub size: Option<u32>,
    pub svg: bool,
}

pub fn favicon_url(app_url: &str) -> Option<Url> {
    Url::parse(app_url).ok()?.join("/favicon.ico").ok()
}

/// Finds a site's icon: icons declared in the page's HTML first (best match first),
/// then `/favicon.ico`. Returns the image bytes of the first candidate that downloads.
pub fn fetch_icon(app_url: &str) -> Result<Vec<u8>, String> {
    let page = Url::parse(app_url).map_err(|_| "invalid app URL")?;
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .user_agent(USER_AGENT)
        .build()
        .new_agent();

    let mut candidates = Vec::new();
    if let Ok(mut response) = agent.get(page.as_str()).call()
        && let Ok(html) = response.body_mut().with_config().limit(MAX_PAGE_BYTES).read_to_string()
    {
        candidates = parse_icon_links(&html, &page);
    }
    if let Some(fallback) = favicon_url(app_url) {
        candidates.push(IconCandidate { url: fallback, size: None, svg: false });
    }

    let mut last_error = String::from("no icon found");
    for candidate in rank_icons(candidates) {
        match download(&agent, &candidate.url) {
            Ok(bytes) => return Ok(bytes),
            Err(e) => last_error = e,
        }
    }
    Err(last_error)
}

fn download(agent: &ureq::Agent, url: &Url) -> Result<Vec<u8>, String> {
    let bytes = agent
        .get(url.as_str())
        .call()
        .map_err(|e| format!("{url}: {e}"))?
        .body_mut()
        .with_config()
        .limit(MAX_ICON_BYTES)
        .read_to_vec()
        .map_err(|e| format!("{url}: {e}"))?;
    sniff_mime(&bytes).ok_or_else(|| format!("{url} is not a supported image"))?;
    Ok(bytes)
}

/// Extracts `rel="icon"`, `rel="shortcut icon"` and `rel="apple-touch-icon"` links, in document order.
pub fn parse_icon_links(html: &str, base: &Url) -> Vec<IconCandidate> {
    // ASCII lowercasing keeps byte offsets identical, so indexes are valid in `html`.
    let lower = html.to_ascii_lowercase();
    let mut icons = Vec::new();
    let mut from = 0;
    while let Some(offset) = lower[from..].find("<link") {
        let start = from + offset;
        let Some(len) = lower[start..].find('>') else { break };
        let attrs = parse_attributes(&html[start + "<link".len()..start + len]);
        from = start + len + 1;

        let rel = attribute(&attrs, "rel").unwrap_or_default().to_ascii_lowercase();
        let is_icon = rel
            .split_ascii_whitespace()
            .any(|r| matches!(r, "icon" | "apple-touch-icon" | "apple-touch-icon-precomposed"));
        let Some(href) = attribute(&attrs, "href").filter(|_| is_icon) else { continue };
        let Ok(url) = base.join(&href.replace("&amp;", "&")) else { continue };
        if !matches!(url.scheme(), "http" | "https") {
            continue;
        }

        let size = attribute(&attrs, "sizes").and_then(|s| largest_size(&s));
        let svg = attribute(&attrs, "type").is_some_and(|t| t.eq_ignore_ascii_case("image/svg+xml"))
            || url.path().to_ascii_lowercase().ends_with(".svg");
        icons.push(IconCandidate { url, size, svg });
    }
    icons
}

/// Best first: SVGs (scale cleanly), then sized icons closest to the target size
/// (larger wins ties), then unsized ones in their original order. Duplicates are dropped.
pub fn rank_icons(mut icons: Vec<IconCandidate>) -> Vec<IconCandidate> {
    icons.sort_by_key(|icon| {
        let group = if icon.svg {
            0
        } else if icon.size.is_some() {
            1
        } else {
            2
        };
        let distance = icon.size.map_or(u32::MAX, |s| s.abs_diff(TARGET_ICON_SIZE));
        (group, distance, std::cmp::Reverse(icon.size.unwrap_or(0)))
    });
    let mut seen = HashSet::new();
    icons.retain(|icon| seen.insert(icon.url.clone()));
    icons
}

fn attribute(attrs: &[(String, String)], name: &str) -> Option<String> {
    attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v.clone())
}

/// Minimal HTML attribute parser for the inside of a start tag.
fn parse_attributes(tag: &str) -> Vec<(String, String)> {
    let chars: Vec<char> = tag.chars().collect();
    let mut attrs = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        while i < chars.len() && (chars[i].is_whitespace() || chars[i] == '/') {
            i += 1;
        }
        let name_start = i;
        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '=' && chars[i] != '/' {
            i += 1;
        }
        if i == name_start {
            i += 1;
            continue;
        }
        let name = chars[name_start..i].iter().collect::<String>().to_ascii_lowercase();
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        let mut value = String::new();
        if i < chars.len() && chars[i] == '=' {
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
                let quote = chars[i];
                let value_start = i + 1;
                i = value_start;
                while i < chars.len() && chars[i] != quote {
                    i += 1;
                }
                value = chars[value_start..i].iter().collect();
                i += 1;
            } else {
                let value_start = i;
                while i < chars.len() && !chars[i].is_whitespace() {
                    i += 1;
                }
                value = chars[value_start..i].iter().collect();
            }
        }
        attrs.push((name, value));
    }
    attrs
}

/// `"16x16 32x32"` -> 32; `"any"` -> None.
fn largest_size(sizes: &str) -> Option<u32> {
    sizes
        .split_ascii_whitespace()
        .filter_map(|s| {
            let (w, h) = s.to_ascii_lowercase().split_once('x').map(|(w, h)| (w.to_owned(), h.to_owned()))?;
            Some(w.parse::<u32>().ok()?.max(h.parse::<u32>().ok()?))
        })
        .max()
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]).to_ascii_lowercase();
    let text = head.trim_start_matches('\u{feff}').trim_start();
    text.starts_with("<svg") || (text.starts_with("<?xml") && text.contains("<svg"))
}

/// Detects image formats by content; anything else is rejected.
pub fn sniff_mime(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [0x89, b'P', b'N', b'G', ..] => Some("image/png"),
        [0, 0, 1, 0, ..] => Some("image/x-icon"),
        [0xFF, 0xD8, 0xFF, ..] => Some("image/jpeg"),
        [b'G', b'I', b'F', b'8', ..] => Some("image/gif"),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => Some("image/webp"),
        _ if looks_like_svg(bytes) => Some("image/svg+xml"),
        _ => None,
    }
}

pub fn data_url(bytes: &[u8]) -> Option<String> {
    let mime = sniff_mime(bytes)?;
    Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
}

pub fn write_icon(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

    fn base() -> Url {
        Url::parse("https://dinheiro.example.com/monthly/9/2026/").unwrap()
    }

    #[test]
    fn favicon_is_at_the_site_root() {
        assert_eq!(
            favicon_url("https://mail.google.com/mail/u/0/").unwrap().as_str(),
            "https://mail.google.com/favicon.ico"
        );
        assert_eq!(favicon_url("not a url"), None);
    }

    #[test]
    fn parses_icon_links_and_resolves_urls() {
        let html = r#"
            <link rel="stylesheet" href="/app.css">
            <LINK REL="apple-touch-icon" sizes="180x180" href="/static/apple-180.png">
            <link rel='icon' type='image/png' sizes='16x16 32x32' href='favicon-32.png'>
            <link href="https://cdn.example.com/logo.svg" rel="shortcut icon" type="image/svg+xml" />
            <link rel=icon href=/plain.ico>
            <link rel="icon" href="data:image/png;base64,AAAA">
            <link rel="icon" href="/q.png?v=1&amp;s=2">
        "#;

        let icons = parse_icon_links(html, &base());

        let summary: Vec<(String, Option<u32>, bool)> =
            icons.iter().map(|i| (i.url.to_string(), i.size, i.svg)).collect();
        assert_eq!(
            summary,
            vec![
                ("https://dinheiro.example.com/static/apple-180.png".into(), Some(180), false),
                ("https://dinheiro.example.com/monthly/9/2026/favicon-32.png".into(), Some(32), false),
                ("https://cdn.example.com/logo.svg".into(), None, true),
                ("https://dinheiro.example.com/plain.ico".into(), None, false),
                ("https://dinheiro.example.com/q.png?v=1&s=2".into(), None, false),
            ]
        );
    }

    #[test]
    fn ranking_prefers_svg_then_sizes_near_the_target() {
        let icon = |path: &str, size: Option<u32>, svg: bool| IconCandidate { url: base().join(path).unwrap(), size, svg };
        let ranked = rank_icons(vec![
            icon("/unsized.png", None, false),
            icon("/16.png", Some(16), false),
            icon("/192.png", Some(192), false),
            icon("/120.png", Some(120), false),
            icon("/136.png", Some(136), false),
            icon("/logo.svg", None, true),
            icon("/120.png", Some(120), false),
        ]);

        let paths: Vec<&str> = ranked.iter().map(|i| i.url.path()).collect();
        assert_eq!(paths, vec!["/logo.svg", "/136.png", "/120.png", "/192.png", "/16.png", "/unsized.png"]);
    }

    #[test]
    fn size_attribute_parsing() {
        assert_eq!(largest_size("16x16 32x32"), Some(32));
        assert_eq!(largest_size("180X180"), Some(180));
        assert_eq!(largest_size("any"), None);
        assert_eq!(largest_size(""), None);
    }

    #[test]
    fn sniffs_supported_formats_only() {
        assert_eq!(sniff_mime(PNG), Some("image/png"));
        assert_eq!(sniff_mime(&[0, 0, 1, 0, 1, 0]), Some("image/x-icon"));
        assert_eq!(sniff_mime(b"  <svg xmlns=\"http://www.w3.org/2000/svg\"></svg>"), Some("image/svg+xml"));
        assert_eq!(sniff_mime(b"<?xml version=\"1.0\"?>\n<svg></svg>"), Some("image/svg+xml"));
        assert_eq!(sniff_mime(b"<!doctype html><html>"), None);
        assert_eq!(sniff_mime(&[]), None);
    }

    #[test]
    fn data_url_and_write_round_trip() {
        assert!(data_url(PNG).unwrap().starts_with("data:image/png;base64,"));
        assert!(data_url(b"<svg></svg>").unwrap().starts_with("data:image/svg+xml;base64,"));
        assert_eq!(data_url(b"<html>"), None);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("icons").join("a.png");
        write_icon(&path, PNG).unwrap();
        assert_eq!(fs::read(&path).unwrap(), PNG);
    }
}
