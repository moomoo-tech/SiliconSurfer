pub mod browser;
pub mod cdp;
pub mod distiller;
pub mod distiller_fast;
pub mod extract;
pub mod fetcher;
pub mod probe;
pub mod profiles;
pub mod router;
pub mod session;
pub mod strategy;
pub mod web;

pub use browser::{find_chromium, BrowserError, BrowserPool};
pub use distiller::Distiller;
pub use distiller_fast::DistillMode;
pub use fetcher::{FetchError, FetchOptions, FetchResult, Fetcher};
pub use router::{Engine, FetchMode};

// Public web API — the two primitives downstream consumers (tars/concer) link.
pub use web::{
    fetch, search, BackendKind, BraveApiDialect, BraveConfig, DdgScrapeDialect, FetchOpts,
    GoogleCseConfig, GoogleCseDialect, Page, SearchBackend, SearchConfig, SearchDialect,
    SearchOpts, SearchRequest, SearchResult, Tier, WebError,
};
