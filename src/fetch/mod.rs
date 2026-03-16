pub(crate) mod dynamic_fetch;
pub(crate) mod static_fetch;

use url::Url;
use crate::error::Result;

/// Defines the strategy used to fetch a page.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FetchPath {
    /// Standard HTTP request (Fast, no JS).
    #[default]
    Static,
    /// Headless browser execution (Slow, renders JS).
    Dynamic,
    /// Detect if a site is an SPA and switch to Dynamic if needed.
    Auto,
}

/// The main entry point for the fetch layer. 
/// It decides which implementation to call based on the `FetchPath`.
#[inline(always)]
pub(crate) async fn get_html(
    url: &Url,
    client: &reqwest::Client,
    path: FetchPath,
) -> Result<(String, bool)> {
    match path {
        FetchPath::Static => {
            let html = static_fetch::get_html(url, client).await?;
            Ok((html, false))
        }
        FetchPath::Dynamic => {
            let html = dynamic_fetch::get_html(url).await?;
            Ok((html, true))
        }
        FetchPath::Auto => {
            // 1. Try the fast path first
            let html = static_fetch::get_html(url, client).await?;
            
            // 2. Run the SPA Detector (The Skeleton Check)
            if is_spa(&html) {
                // 3. If it's a shell, upgrade to the heavy Dynamic path
                let dynamic_html = dynamic_fetch::get_html(url).await?;
                Ok((dynamic_html, true))
            } else {
                Ok((html, false))
            }
        }
    }
}

/// Detects if the given HTML shell belongs to a Single Page Application (SPA).
///
/// This uses a multi-signal heuristic to identify JS-driven sites that require
/// a headless browser for full rendering.
pub fn is_spa(html: &str) -> bool {
    let low = html.to_lowercase();
    let len = html.len();

    // 1. The "Loud" Signals (Unconditional)
    // If a site explicitly mentions disabling JS, or has framework version markers, 
    // it's an SPA shell or requires JS-heavy hydration.
    if (low.contains("<noscript") && (low.contains("javascript") || low.contains("enable js"))) ||
       low.contains("ng-version=") || 
       low.contains("data-reactroot") ||
       low.contains("data-server-rendered") {
        return true;
    }

    // 2. SSR & Metadata Signals
    // AJAX crawlability and SSR state markers indicate a JS-driven experience.
    if low.contains("name=\"fragment\" content=\"!\"") || 
       low.contains("window.__initial_state__") ||
       low.contains("window.__next_data__") {
        return true;
    }

    // 3. The "Root Container" Check
    // Checks for common framework mounting points. The size threshold is increased 
    // to 10KB to handle shells with heavy inline styles/meta-data.
    let has_root_container = low.contains("id=\"app\"") || 
                             low.contains("id=\"root\"") || 
                             low.contains("id=\"__next\"") ||
                             low.contains("id=\"__nuxt\"") ||
                             low.contains("id=\"___gatsby\"") ||
                             low.contains("id=\"app-root\"") ||
                             low.contains("<app-root") || // Angular's custom element
                             low.contains("id=\"ember-application\"");

    if has_root_container && len < 10240 {
        return true;
    }

    // 4. The "Bundle" Heuristic
    // Even if we miss the root ID, a relatively small file containing complex 
    // script bundle patterns is almost certainly a modern SPA shell.
    if len < 15360 && (
        low.contains(".chunk.js") || 
        low.contains("bundle.js") || 
        low.contains("vendor.js") ||
        low.contains("_next/static")
    ) {
        return true;
    }

    false
}
