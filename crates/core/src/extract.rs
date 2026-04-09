//! Unified extraction strategy trait + configurable profiles.
//!
//! Both scraper (AST) and lol_html (stream) implement the same trait.
//! Profiles define noise rules, modes, and behavior — loaded from config, not hardcoded.

use serde::{Deserialize, Serialize};

use crate::distiller_fast::DistillMode;

/// Unified extraction trait — both engines implement this.
pub trait Extractor: Send + Sync {
    fn extract(&self, html: &str, profile: &Profile) -> ExtractionResult;
    fn name(&self) -> &str;
}

/// Extraction result with metadata for eval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub content: String,
    pub title: Option<String>,
    pub content_length: usize,
    pub links_count: usize,
    pub headings_count: usize,
    pub engine: String,
    pub mode: DistillMode,
}

/// Configurable extraction profile — replaces hardcoded noise selectors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub mode: DistillMode,
    pub base_url: Option<String>,
    /// CSS selectors for elements to remove entirely
    pub noise_selectors: Vec<String>,
    /// CSS selectors to find main content area (AST only, priority order)
    pub content_selectors: Vec<String>,
    /// Additional noise text patterns to filter in post-processing
    pub noise_text_patterns: Vec<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self::reader(None)
    }
}

impl Profile {
    /// Reader mode: aggressive noise removal, clean markdown
    pub fn reader(base_url: Option<&str>) -> Self {
        Self {
            name: "reader".to_string(),
            mode: DistillMode::Reader,
            base_url: base_url.map(|s| s.to_string()),
            noise_selectors: vec![
                "script",
                "style",
                "nav",
                "footer",
                "header",
                "iframe",
                "noscript",
                "svg",
                "form",
                "button",
                "input",
                "select",
                "textarea",
                "head",
                "img",
                "picture",
                "video",
                "audio",
                "canvas",
                "[class*='ad-']",
                "[class*='ads-']",
                "[class*='cookie-']",
                "[class*='cookie_']",
                ".popup",
                ".modal",
                "[class*='-popup']",
                "[class*='-modal']",
                ".social-share",
                ".share-buttons",
                ".sharing",
                ".newsletter",
                ".subscribe",
                "[class*='-banner'][class*='ad']",
                "[role='navigation']",
                "[role='complementary']",
                "[role='search']",
                "[aria-hidden='true']",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
            content_selectors: vec![
                "article",
                "main",
                "[role='main']",
                ".post-content",
                ".article-content",
                ".entry-content",
                ".post-body",
                ".article-body",
                "#content",
                ".content",
                "#main-content",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
            noise_text_patterns: vec![],
        }
    }

    /// Operator mode: minimal noise removal, preserve UI
    pub fn operator(base_url: Option<&str>) -> Self {
        Self {
            name: "operator".to_string(),
            mode: DistillMode::Operator,
            base_url: base_url.map(|s| s.to_string()),
            noise_selectors: vec![
                "script", "style", "head", "noscript", "svg", "img", "picture", "video", "audio",
                "canvas",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
            content_selectors: vec![],
            noise_text_patterns: vec![],
        }
    }

    /// Site-specific profile (e.g. Hacker News)
    pub fn hacker_news() -> Self {
        let mut p = Self::reader(Some("https://news.ycombinator.com/"));
        p.name = "hacker_news".to_string();
        // HN-specific noise
        p.noise_selectors.extend([
            ".pagetop".to_string(),              // Top navigation bar
            ".yclinks".to_string(),              // Bottom links
            "td[bgcolor='#ff6600']".to_string(), // Orange nav bar
        ]);
        p.noise_text_patterns = vec!["| hide |".to_string(), "| past |".to_string()];
        p
    }

    /// Custom profile from selectors
    pub fn custom(
        name: &str,
        mode: DistillMode,
        base_url: Option<&str>,
        noise: Vec<&str>,
        content: Vec<&str>,
    ) -> Self {
        Self {
            name: name.to_string(),
            mode,
            base_url: base_url.map(|s| s.to_string()),
            noise_selectors: noise.into_iter().map(|s| s.to_string()).collect(),
            content_selectors: content.into_iter().map(|s| s.to_string()).collect(),
            noise_text_patterns: vec![],
        }
    }
}
