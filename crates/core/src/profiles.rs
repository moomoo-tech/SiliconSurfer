//! Site-specific profiles — loaded from profiles/ directory.
//!
//! Each site gets its own .toml file. No recompilation needed.
//! Future: load from cloud database / API.

use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Deserialize, Clone)]
pub struct SiteProfile {
    pub name: String,
    pub domains: Vec<String>,
    #[serde(default)]
    pub extra_noise: Vec<String>,
    /// Force T1 (Chrome) for this domain — required for SPAs with async data loading.
    #[serde(default)]
    pub force_t1: bool,
    /// Override DOM quiescence wait (ms). Default is 500. Set higher for WebSocket/push sites.
    #[serde(default)]
    pub wait_ms: Option<u64>,
    /// Wait for a specific CSS selector to appear before extraction.
    #[serde(default)]
    pub wait_for_selector: Option<String>,
}

/// Legacy format: profiles.toml with [[profile]] array.
#[derive(Debug, Deserialize)]
struct ProfilesFile {
    profile: Vec<SiteProfile>,
}

static PROFILES: OnceLock<Vec<SiteProfile>> = OnceLock::new();

/// Load profiles from profiles/ directory (one .toml per site),
/// with fallback to legacy profiles.toml.
fn load_profiles() -> Vec<SiteProfile> {
    let mut all = Vec::new();

    // Try profiles/ directory first (new format: one file per site)
    let dirs = ["profiles", "../profiles", "../../profiles"];
    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "toml")
                    && let Ok(content) = std::fs::read_to_string(&path)
                    && let Ok(profile) = toml::from_str::<SiteProfile>(&content)
                {
                    all.push(profile);
                }
            }
            if !all.is_empty() {
                return all;
            }
        }
    }

    // Fallback: legacy profiles.toml (single file with [[profile]] array)
    let paths = ["profiles.toml", "../profiles.toml", "../../profiles.toml"];
    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path)
            && let Ok(parsed) = toml::from_str::<ProfilesFile>(&content)
        {
            return parsed.profile;
        }
    }

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

/// Check if this URL requires T1 (Chrome) rendering.
pub fn requires_t1(url: &str) -> bool {
    match_profile(url).is_some_and(|p| p.force_t1)
}

/// Get custom wait time for this URL (ms), if configured.
pub fn custom_wait_ms(url: &str) -> Option<u64> {
    match_profile(url).and_then(|p| p.wait_ms)
}

/// Get wait-for-selector for this URL, if configured.
pub fn wait_for_selector(url: &str) -> Option<&str> {
    match_profile(url).and_then(|p| p.wait_for_selector.as_deref())
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
