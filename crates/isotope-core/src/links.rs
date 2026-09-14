use url::{Host, Url};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkTarget {
    /// Let the app webview handle it.
    InApp,
    /// Open in the user's default browser.
    External,
    /// Hand to the OS (mailto:, tel:, custom protocols, unparseable).
    OsOpener,
}

pub fn classify(app_url: &str, allowed_domains: &[String], target: &str) -> LinkTarget {
    let Ok(url) = Url::parse(target) else {
        return LinkTarget::OsOpener;
    };
    match url.scheme() {
        "javascript" | "data" | "blob" => return LinkTarget::InApp,
        "about" if url.path() == "blank" => return LinkTarget::InApp,
        "http" | "https" => {}
        _ => return LinkTarget::OsOpener,
    }

    let target_site = site(&url);
    let app_site = Url::parse(app_url).ok().as_ref().and_then(site);
    if target_site.is_some() && target_site == app_site {
        return LinkTarget::InApp;
    }

    if let Some(host) = url.host_str()
        && allowed_domains.iter().any(|domain| host_matches(host, domain))
    {
        return LinkTarget::InApp;
    }
    LinkTarget::External
}

/// Registrable domain (eTLD+1) for domain hosts; the literal address for IPs.
fn site(url: &Url) -> Option<String> {
    match url.host()? {
        Host::Domain(domain) => Some(psl::domain_str(domain).unwrap_or(domain).to_string()),
        Host::Ipv4(ip) => Some(ip.to_string()),
        Host::Ipv6(ip) => Some(ip.to_string()),
    }
}

fn host_matches(host: &str, allowed: &str) -> bool {
    let allowed = allowed.trim().trim_start_matches("*.").trim_end_matches('.').to_ascii_lowercase();
    !allowed.is_empty() && (host == allowed || host.ends_with(&format!(".{allowed}")))
}
