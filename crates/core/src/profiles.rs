//! Site-specific noise profiles — loaded from profiles.toml.
//!
//! Add new sites to profiles.toml without recompiling.

use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Deserialize, Clone)]
pub struct SiteProfile {
    pub name: String,
    pub domains: Vec<String>,
    pub extra_noise: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ProfilesFile {
    profile: Vec<SiteProfile>,
}

static PROFILES: OnceLock<Vec<SiteProfile>> = OnceLock::new();

/// Load profiles from profiles.toml (or use empty if not found).
fn load_profiles() -> Vec<SiteProfile> {
    // Try multiple paths
    let paths = ["profiles.toml", "../profiles.toml", "../../profiles.toml"];

    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path)
            && let Ok(parsed) = toml::from_str::<ProfilesFile>(&content)
        {
            return parsed.profile;
        }
    }

    // Fallback: empty
    Vec::new()
}

/// Get all loaded profiles.
pub fn profiles() -> &'static [SiteProfile] {
    PROFILES.get_or_init(load_profiles)
}

/// Find matching profile for a URL.
pub fn match_profile(url: &str) -> Option<&'static SiteProfile> {
    let domain = extract_domain(url)?;
    profiles()
        .iter()
        .find(|p| p.domains.iter().any(|d| domain.contains(d.as_str())))
}

/// Get extra noise selectors for a URL (empty if no profile matches).
pub fn extra_noise_for_url(url: &str) -> Vec<&str> {
    match match_profile(url) {
        Some(profile) => profile.extra_noise.iter().map(|s| s.as_str()).collect(),
        None => Vec::new(),
    }
}

fn extract_domain(url: &str) -> Option<&str> {
    let after_scheme = url.find("://").map(|i| &url[i + 3..])?;
    let end = after_scheme.find('/').unwrap_or(after_scheme.len());
    Some(&after_scheme[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_profiles() {
        let p = profiles();
        // May be empty if profiles.toml not in test cwd, that's OK
        println!("Loaded {} profiles", p.len());
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://cloud.tencent.com/dev/article/123"),
            Some("cloud.tencent.com")
        );
        assert_eq!(
            extract_domain("https://github.com/foo/bar"),
            Some("github.com")
        );
    }
}
