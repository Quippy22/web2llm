//! Recursive crawl configuration and URL filtering helpers.
//!
//! The crawler in `web2llm` is intentionally simple: it discovers links
//! breadth-first with `get_urls`, applies optional origin preservation, and
//! then performs one final `batch_fetch` over the discovered set.

use url::Url;

/// Configuration for recursive crawling.
///
/// The crawler first discovers links level-by-level using `get_urls`, then
/// performs one final `batch_fetch` over the discovered URL set.
#[derive(Debug, Clone)]
pub struct CrawlConfig {
    /// Maximum number of link-expansion steps from the seed URL.
    ///
    /// `0` means only the seed URL is fetched in the final batch.
    pub max_depth: usize,
    /// If `true`, only URLs on the same origin as the seed URL are followed.
    ///
    /// In practice this preserves the seed host and effective port, which keeps
    /// a crawl pinned to the same site by default.
    pub preserve_domain: bool,
}

impl Default for CrawlConfig {
    /// Returns the default crawl configuration.
    ///
    /// Defaults are intentionally conservative:
    /// - `max_depth = 0`
    /// - `preserve_domain = true`
    fn default() -> Self {
        Self {
            max_depth: 0,
            preserve_domain: true,
        }
    }
}

pub(crate) fn normalize_url(raw: &str) -> Option<Url> {
    let mut url = Url::parse(raw).ok()?;
    url.set_fragment(None);
    Some(url)
}

pub(crate) fn should_follow(
    url: &Url,
    seed_host: &str,
    seed_port: Option<u16>,
    config: &CrawlConfig,
) -> bool {
    matches!(url.scheme(), "http" | "https")
        && (!config.preserve_domain
            || (url.host_str() == Some(seed_host) && url.port_or_known_default() == seed_port))
}
