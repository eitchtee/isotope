use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use url::Url;

pub const MAX_ICON_BYTES: u64 = 1024 * 1024;

pub fn favicon_url(app_url: &str) -> Option<Url> {
    Url::parse(app_url).ok()?.join("/favicon.ico").ok()
}

pub fn fetch_favicon(app_url: &str) -> Result<Vec<u8>, String> {
    let url = favicon_url(app_url).ok_or("invalid app URL")?;
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .new_agent();
    let bytes = agent
        .get(url.as_str())
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .with_config()
        .limit(MAX_ICON_BYTES)
        .read_to_vec()
        .map_err(|e| e.to_string())?;
    sniff_mime(&bytes).ok_or("favicon is not a supported image")?;
    Ok(bytes)
}

/// Detects image formats by magic bytes; anything else is rejected.
pub fn sniff_mime(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [0x89, b'P', b'N', b'G', ..] => Some("image/png"),
        [0, 0, 1, 0, ..] => Some("image/x-icon"),
        [0xFF, 0xD8, 0xFF, ..] => Some("image/jpeg"),
        [b'G', b'I', b'F', b'8', ..] => Some("image/gif"),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => Some("image/webp"),
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

    #[test]
    fn favicon_is_at_the_site_root() {
        assert_eq!(
            favicon_url("https://mail.google.com/mail/u/0/").unwrap().as_str(),
            "https://mail.google.com/favicon.ico"
        );
        assert_eq!(favicon_url("not a url"), None);
    }

    #[test]
    fn sniffs_supported_formats_only() {
        assert_eq!(sniff_mime(PNG), Some("image/png"));
        assert_eq!(sniff_mime(&[0, 0, 1, 0, 1, 0]), Some("image/x-icon"));
        assert_eq!(sniff_mime(b"<html>"), None);
        assert_eq!(sniff_mime(&[]), None);
    }

    #[test]
    fn data_url_and_write_round_trip() {
        assert!(data_url(PNG).unwrap().starts_with("data:image/png;base64,"));
        assert_eq!(data_url(b"<svg"), None);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("icons").join("a.png");
        write_icon(&path, PNG).unwrap();
        assert_eq!(fs::read(&path).unwrap(), PNG);
    }
}
