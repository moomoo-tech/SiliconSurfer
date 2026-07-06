//! T1: Headless Chrome browser pool via CDP.
//!
//! - Global daemon process (single Chromium instance)
//! - Millisecond-level context creation/destruction
//! - Block CSS/images/fonts/media at network level
//! - Extract content via JS injection or raw HTML + Rust distiller

use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::distiller::Distiller;

#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    /// No Chromium-family browser could be found or launched. Permanent: the
    /// caller should fall back to the static result or tell the user to install
    /// Chrome (or set `CHROME_PATH`). Carries a human-readable hint, never a
    /// bare stringified internal error.
    #[error("no usable Chromium-family browser: {hint}")]
    NoBrowser { hint: String },
    /// The browser process died mid-session (websocket/channel dropped). Transient:
    /// the pool can be restarted and the fetch retried once.
    #[error("Browser process crashed: {0}")]
    Crashed(String),
    /// Navigation / CDP command timed out. Transient.
    #[error("Browser navigation timed out: {0}")]
    Timeout(String),
    /// The browser binary was found but failed to launch (bad build, exited early).
    /// Permanent for this pool config.
    #[error("Browser launch failed: {0}")]
    Launch(String),
    /// Any other page-level CDP failure (JS exception, decode, not-found …). Permanent.
    #[error("Page error: {0}")]
    Page(String),
    /// The pool has not been started yet. Transient: `ensure_started` will start it.
    #[error("Browser not started")]
    NotStarted,
}

impl BrowserError {
    /// Whether a retry (after a possible restart) could plausibly succeed.
    /// `NoBrowser` and generic `Page` errors are permanent; a crash / timeout /
    /// not-yet-started pool are transient.
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            BrowserError::Crashed(_) | BrowserError::Timeout(_) | BrowserError::NotStarted
        )
    }

    /// Whether this error means the browser process needs restarting before retry.
    pub fn needs_restart(&self) -> bool {
        matches!(self, BrowserError::Crashed(_))
    }
}

impl From<chromiumoxide::error::CdpError> for BrowserError {
    fn from(e: chromiumoxide::error::CdpError) -> Self {
        use chromiumoxide::error::CdpError;
        match e {
            // The transport to the browser dropped → the process is gone/unusable.
            CdpError::Ws(_)
            | CdpError::ChannelSendError(_)
            | CdpError::NoResponse
            | CdpError::UnexpectedWsMessage(_) => BrowserError::Crashed(e.to_string()),
            // Navigation / command timeouts.
            CdpError::Timeout | CdpError::LaunchTimeout(_) => BrowserError::Timeout(e.to_string()),
            // Process exited / IO during launch → treat as launch failure.
            CdpError::LaunchExit(_, _) | CdpError::LaunchIo(_, _) => {
                BrowserError::Launch(e.to_string())
            }
            // Everything else (JS exception, decode, not-found, bad message …) is permanent.
            other => BrowserError::Page(other.to_string()),
        }
    }
}

/// Resolve a Chromium-family browser executable, honoring overrides.
///
/// Search order (first hit wins):
///   1. `CHROME_PATH` env var (sisurf's documented override) — if it points at an
///      existing file.
///   2. `CHROME` env var (chromiumoxide's native override).
///   3. `which` for the usual family binaries: Chrome, Chromium, Edge, **Brave**.
///   4. Well-known per-OS install paths (incl. Brave, which chromiumoxide misses).
///
/// Returns `None` when nothing on the machine can drive CDP — the caller turns that
/// into a typed [`BrowserError::NoBrowser`].
pub fn find_chromium() -> Option<PathBuf> {
    // 1 + 2: explicit overrides.
    for var in ["CHROME_PATH", "CHROME"] {
        if let Ok(p) = std::env::var(var) {
            let path = PathBuf::from(&p);
            if path.exists() {
                return Some(path);
            }
        }
    }

    // 3: PATH lookup across the whole Chromium family (incl. Brave).
    const NAMES: &[&str] = &[
        "google-chrome",
        "google-chrome-stable",
        "chrome",
        "chromium",
        "chromium-browser",
        "microsoft-edge",
        "microsoft-edge-stable",
        "msedge",
        "brave-browser",
        "brave",
    ];
    for name in NAMES {
        if let Ok(path) = which_on_path(name) {
            return Some(path);
        }
    }

    // 4: well-known install locations.
    #[cfg(target_os = "macos")]
    const KNOWN: &[&str] = &[
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
    ];
    #[cfg(all(unix, not(target_os = "macos")))]
    const KNOWN: &[&str] = &[
        "/usr/bin/google-chrome",
        "/opt/google/chrome/chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/opt/chromium.org/chromium/chromium",
        "/usr/bin/microsoft-edge",
        "/usr/bin/brave-browser",
        "/opt/brave.com/brave/brave",
    ];
    #[cfg(windows)]
    const KNOWN: &[&str] = &[
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe",
    ];

    KNOWN.iter().map(PathBuf::from).find(|p| p.exists())
}

/// Minimal cross-platform `which`, avoiding a new crate dependency.
fn which_on_path(name: &str) -> Result<PathBuf, ()> {
    let path_var = std::env::var_os("PATH").ok_or(())?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
        #[cfg(windows)]
        {
            let exe = dir.join(format!("{name}.exe"));
            if exe.is_file() {
                return Ok(exe);
            }
        }
    }
    Err(())
}

/// Result from a T1 browser fetch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserFetchResult {
    pub url: String,
    pub content: String,
    pub title: Option<String>,
    pub content_length: usize,
}

/// Headless Chrome pool — single daemon, multiple contexts.
pub struct BrowserPool {
    browser: Arc<Mutex<Option<Browser>>>,
    distiller: Distiller,
    /// If true, strip images/fonts/canvas for max speed. If false, load everything (for vision/captcha).
    performance_mode: bool,
    /// Forced executable path. `None` → auto-detect via [`find_chromium`] at start.
    /// Used to pin a specific binary (or, in tests, an intentionally bad one).
    executable: Option<PathBuf>,
}

impl Default for BrowserPool {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserPool {
    pub fn new() -> Self {
        Self {
            browser: Arc::new(Mutex::new(None)),
            distiller: Distiller::new(),
            performance_mode: true,
            executable: None,
        }
    }

    /// Create with explicit performance mode setting.
    pub fn with_performance_mode(performance: bool) -> Self {
        Self {
            browser: Arc::new(Mutex::new(None)),
            distiller: Distiller::new(),
            performance_mode: performance,
            executable: None,
        }
    }

    /// Pin a specific Chromium executable, bypassing auto-detection.
    pub fn with_executable(path: impl Into<PathBuf>) -> Self {
        Self {
            browser: Arc::new(Mutex::new(None)),
            distiller: Distiller::new(),
            performance_mode: true,
            executable: Some(path.into()),
        }
    }

    /// Start the Chrome daemon with aggressive resource stripping.
    ///
    /// Returns [`BrowserError::NoBrowser`] when no Chromium-family executable can be
    /// found or the chosen binary fails to launch — a typed signal so callers can
    /// fall back to the static result instead of parsing a string.
    pub async fn start(&self) -> Result<(), BrowserError> {
        // Resolve the executable up-front so "no browser installed" is a clean,
        // typed error rather than an opaque launch failure.
        let executable = match &self.executable {
            Some(p) => p.clone(),
            None => find_chromium().ok_or_else(|| BrowserError::NoBrowser {
                hint: "no Chrome/Chromium/Edge/Brave found on PATH or in the usual \
                       locations; install one or set CHROME_PATH to its executable"
                    .to_string(),
            })?,
        };
        if !executable.exists() {
            return Err(BrowserError::NoBrowser {
                hint: format!(
                    "configured browser executable does not exist: {}",
                    executable.display()
                ),
            });
        }

        let mut builder = BrowserConfig::builder().chrome_executable(&executable);
        builder = builder.no_sandbox();

        // --- Always-on: basic sanity flags ---
        builder = builder
            .arg("--disable-dev-shm-usage")
            .arg("--disable-extensions")
            .arg("--disable-default-apps")
            .arg("--disable-sync")
            .arg("--disable-translate")
            .arg("--no-first-run")
            .arg("--mute-audio")
            .arg("--disable-popup-blocking")
            .arg("--disable-notifications")
            .arg("--disable-prompt-on-repost")
            .arg("--disable-hang-monitor");

        if self.performance_mode {
            // --- Performance mode: strip everything AI doesn't need ---
            builder = builder
                // Kill images
                .arg("--blink-settings=imagesEnabled=false")
                // Kill GPU/rendering
                .arg("--disable-gpu")
                .arg("--disable-software-rasterizer")
                .arg("--disable-canvas-aa")
                .arg("--disable-2d-canvas-clip-aa")
                .arg("--disable-gl-drawing-for-tests")
                // Kill fonts
                .arg("--disable-remote-fonts")
                // Kill background services
                .arg("--disable-background-networking")
                .arg("--disable-background-timer-throttling")
                .arg("--disable-backgrounding-occluded-windows")
                .arg("--metrics-recording-only")
                .arg("--disable-component-update")
                .arg("--disable-domain-reliability")
                .arg("--aggressive-cache-discard")
                .arg("--disable-ipc-flooding-protection");
        }
        // else: Vision mode — load everything (images, fonts, CSS) for captcha/screenshots

        let config = builder.build().map_err(|e| BrowserError::NoBrowser {
            hint: format!("invalid browser config for {}: {e}", executable.display()),
        })?;

        // A launch failure here means the binary exists but can't drive CDP
        // (wrong build, missing libs, sandbox). Surface it as NoBrowser so the
        // caller falls back to the static result rather than hard-failing.
        let (browser, mut handler) = Browser::launch(config).await.map_err(|e| {
            BrowserError::NoBrowser {
                hint: format!("failed to launch {}: {e}", executable.display()),
            }
        })?;

        // Spawn CDP event handler in background
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });

        let mut guard = self.browser.lock().await;
        *guard = Some(browser);
        Ok(())
    }

    /// Ensure browser is running, start if needed.
    pub async fn ensure_started(&self) -> Result<(), BrowserError> {
        let guard = self.browser.lock().await;
        if guard.is_none() {
            drop(guard);
            self.start().await?;
        }
        Ok(())
    }

    /// Get a lock on the browser instance.
    pub async fn browser_guard(
        &self,
    ) -> Result<tokio::sync::MutexGuard<'_, Option<Browser>>, BrowserError> {
        Ok(self.browser.lock().await)
    }

    /// T1 fetch: render page with JS, extract clean content.
    pub async fn fetch(&self, url: &str, output: &str) -> Result<BrowserFetchResult, BrowserError> {
        self.ensure_started().await?;

        // Create a blank page under the lock (instant CDP command, no network I/O),
        // then drop the lock BEFORE navigating. This allows true concurrency.
        let page = {
            let guard = self.browser.lock().await;
            let browser = guard.as_ref().ok_or(BrowserError::NotStarted)?;
            browser
                .new_page("about:blank")
                .await
                .map_err(BrowserError::from)?
            // guard drops here — lock released before any network I/O
        };

        // Navigate without holding the lock — other tasks can create pages concurrently
        page.goto(url)
            .await
            .map_err(BrowserError::from)?;
        page.wait_for_navigation()
            .await
            .map_err(BrowserError::from)?;

        // Extract title
        let title = page
            .evaluate("document.title")
            .await
            .ok()
            .and_then(|v| v.into_value::<String>().ok())
            .filter(|t| !t.is_empty());

        // Get rendered HTML (after JS execution)
        let raw_html = page
            .content()
            .await
            .map_err(BrowserError::from)?;

        // Close page immediately — free resources
        let _ = page.close().await;

        // Distill in Rust — same pipeline as T0
        let content = match output {
            "text" => self.distiller.to_text(&raw_html),
            _ => self.distiller.to_markdown(&raw_html),
        };
        let content_length = content.len();

        Ok(BrowserFetchResult {
            url: url.to_string(),
            content,
            title,
            content_length,
        })
    }

    /// Fetch raw rendered HTML (after JS execution). No distilling.
    /// Used by Probe for DOM checks on JS-rendered pages.
    pub async fn fetch_raw_html(&self, url: &str) -> Result<String, BrowserError> {
        self.ensure_started().await?;

        let page = {
            let guard = self.browser.lock().await;
            let browser = guard.as_ref().ok_or(BrowserError::NotStarted)?;
            browser
                .new_page("about:blank")
                .await
                .map_err(BrowserError::from)?
        };

        page.goto(url)
            .await
            .map_err(BrowserError::from)?;
        page.wait_for_navigation()
            .await
            .map_err(BrowserError::from)?;

        let html = page
            .content()
            .await
            .map_err(BrowserError::from)?;

        let _ = page.close().await;
        Ok(html)
    }

    /// Fetch with JS injection for content extraction (alternative to Rust distiller).
    /// Useful when you need the browser's own DOM API for complex pages.
    pub async fn fetch_with_js_extract(
        &self,
        url: &str,
    ) -> Result<BrowserFetchResult, BrowserError> {
        self.ensure_started().await?;

        let page = {
            let guard = self.browser.lock().await;
            let browser = guard.as_ref().ok_or(BrowserError::NotStarted)?;
            browser
                .new_page("about:blank")
                .await
                .map_err(BrowserError::from)?
        };

        page.goto(url)
            .await
            .map_err(BrowserError::from)?;
        page.wait_for_navigation()
            .await
            .map_err(BrowserError::from)?;

        let title = page
            .evaluate("document.title")
            .await
            .ok()
            .and_then(|v| v.into_value::<String>().ok())
            .filter(|t| !t.is_empty());

        // JS-side DOM cleanup + text extraction
        let content = page
            .evaluate(
                r#"
                (() => {
                    ['nav','footer','header','script','style','iframe','noscript','svg'].forEach(tag => {
                        document.querySelectorAll(tag).forEach(el => el.remove());
                    });
                    document.querySelectorAll('[class*="ad-"],[class*="ads-"],[class*="banner"],[class*="sidebar"],[class*="popup"],[class*="modal"],[class*="cookie"]').forEach(el => el.remove());
                    let main = document.querySelector('article')
                        || document.querySelector('main')
                        || document.querySelector('[role="main"]')
                        || document.body;
                    return main ? main.innerText : '';
                })()
                "#,
            )
            .await
            .map_err(BrowserError::from)?
            .into_value::<String>()
            .unwrap_or_default();

        let _ = page.close().await;

        let content_length = content.len();

        Ok(BrowserFetchResult {
            url: url.to_string(),
            content,
            title,
            content_length,
        })
    }

    /// Shutdown the browser daemon.
    pub async fn stop(&self) {
        let mut guard = self.browser.lock().await;
        *guard = None;
        // Browser drops, Chrome process killed
    }

    /// Tear down and relaunch the daemon — used to recover from a crashed process
    /// before retrying a fetch. Returns [`BrowserError::NoBrowser`] if the browser
    /// can no longer be launched.
    pub async fn restart(&self) -> Result<(), BrowserError> {
        self.stop().await;
        self.start().await
    }
}
