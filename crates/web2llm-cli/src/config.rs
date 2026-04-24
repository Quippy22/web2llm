//! Config-file parsing and override resolution for `web2llm-cli`.
//!
//! The CLI intentionally layers configuration in a predictable order:
//! library defaults, optional TOML file, then explicit command-line flags.

use std::fs;
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;
use web2llm::{CrawlConfig, FetchMode, Web2llmConfig};

use crate::cli::{CommonOptions, CrawlCommand, FetchModeArg};
use crate::error::{CliError, Result};

/// Optional TOML configuration file consumed by the CLI.
///
/// The file may contain a `[web2llm]` section for engine settings and a
/// `[crawl]` section for crawl defaults.
#[derive(Debug, Default, Deserialize)]
pub struct FileConfig {
    /// Optional engine-level settings.
    #[serde(default)]
    pub web2llm: Web2llmSection,
    /// Optional crawl-level settings.
    #[serde(default)]
    pub crawl: CrawlSection,
}

/// TOML-backed overrides for [`Web2llmConfig`].
#[derive(Debug, Default, Deserialize)]
pub struct Web2llmSection {
    /// Optional user-agent override.
    pub user_agent: Option<String>,
    /// Optional timeout override in whole seconds.
    pub timeout_secs: Option<u64>,
    /// Optional private-host blocking override.
    pub block_private_hosts: Option<bool>,
    /// Optional extraction sensitivity override.
    pub sensitivity: Option<f32>,
    /// Optional chunk token budget override.
    pub max_tokens: Option<usize>,
    /// Optional robots.txt enforcement override.
    pub robots_check: Option<bool>,
    /// Optional rate-limit override.
    pub rate_limit: Option<u32>,
    /// Optional concurrency override.
    pub max_concurrency: Option<usize>,
    /// Optional fetch-mode override.
    pub fetch_mode: Option<FetchModeFileValue>,
    /// Optional ordering override for batch-style operations.
    pub ordered: Option<bool>,
}

/// TOML-backed overrides for [`CrawlConfig`].
#[derive(Debug, Default, Deserialize)]
pub struct CrawlSection {
    /// Optional crawl depth override.
    pub max_depth: Option<usize>,
    /// Optional origin-preservation override.
    pub preserve_domain: Option<bool>,
}

/// Fetch mode values accepted from TOML config files.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FetchModeFileValue {
    /// Automatically choose between static and dynamic fetching.
    Auto,
    /// Use only the static HTTP fetcher.
    Static,
    /// Use the rendered browser fetcher.
    Dynamic,
}

impl From<FetchModeArg> for FetchMode {
    fn from(value: FetchModeArg) -> Self {
        match value {
            FetchModeArg::Auto => FetchMode::Auto,
            FetchModeArg::Static => FetchMode::Static,
            FetchModeArg::Dynamic => FetchMode::Dynamic,
        }
    }
}

impl From<FetchModeFileValue> for FetchMode {
    fn from(value: FetchModeFileValue) -> Self {
        match value {
            FetchModeFileValue::Auto => FetchMode::Auto,
            FetchModeFileValue::Static => FetchMode::Static,
            FetchModeFileValue::Dynamic => FetchMode::Dynamic,
        }
    }
}

/// Loads a CLI TOML config file from disk.
///
/// Returns a default empty configuration when `path` is `None`.
pub fn load_file_config(path: Option<&Path>) -> Result<FileConfig> {
    let Some(path) = path else {
        return Ok(FileConfig::default());
    };

    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

/// Resolves the final [`Web2llmConfig`] from defaults, TOML, and CLI overrides.
pub fn resolve_web2llm_config(file: &FileConfig, options: &CommonOptions) -> Web2llmConfig {
    let mut config = Web2llmConfig::default();

    if let Some(value) = &file.web2llm.user_agent {
        config.user_agent = value.clone();
    }
    if let Some(value) = file.web2llm.timeout_secs {
        config.timeout = Duration::from_secs(value);
    }
    if let Some(value) = file.web2llm.block_private_hosts {
        config.block_private_hosts = value;
    }
    if let Some(value) = file.web2llm.sensitivity {
        config.sensitivity = value;
    }
    if let Some(value) = file.web2llm.max_tokens {
        config.max_tokens = value;
    }
    if let Some(value) = file.web2llm.robots_check {
        config.robots_check = value;
    }
    if let Some(value) = file.web2llm.rate_limit {
        config.rate_limit = value;
    }
    if let Some(value) = file.web2llm.max_concurrency {
        config.max_concurrency = value;
    }
    if let Some(value) = file.web2llm.fetch_mode {
        config.fetch_mode = value.into();
    }
    if let Some(value) = file.web2llm.ordered {
        config.ordered = value;
    }

    if let Some(value) = &options.user_agent {
        config.user_agent = value.clone();
    }
    if let Some(value) = options.timeout_secs {
        config.timeout = Duration::from_secs(value);
    }
    if options.allow_private_hosts {
        config.block_private_hosts = false;
    }
    if options.no_robots {
        config.robots_check = false;
    }
    if let Some(value) = options.sensitivity {
        config.sensitivity = value;
    }
    if let Some(value) = options.max_tokens {
        config.max_tokens = value;
    }
    if let Some(value) = options.rate_limit {
        config.rate_limit = value;
    }
    if let Some(value) = options.max_concurrency {
        config.max_concurrency = value;
    }
    if let Some(value) = options.fetch_mode {
        config.fetch_mode = value.into();
    }
    if options.ordered {
        config.ordered = true;
    }

    config
}

/// Resolves the final [`CrawlConfig`] from defaults, TOML, and CLI overrides.
pub fn resolve_crawl_config(file: &FileConfig, command: &CrawlCommand) -> CrawlConfig {
    let mut config = CrawlConfig::default();

    if let Some(value) = file.crawl.max_depth {
        config.max_depth = value;
    }
    if let Some(value) = file.crawl.preserve_domain {
        config.preserve_domain = value;
    }

    if let Some(value) = command.depth {
        config.max_depth = value;
    }
    if command.cross_origin {
        config.preserve_domain = false;
    }

    config
}

/// Validates the resolved configs before the engine is constructed.
pub fn validate_configs(web_config: &Web2llmConfig) -> Result<()> {
    if web_config.timeout.is_zero() {
        return Err(CliError::Config(
            "timeout_secs must be greater than zero".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::CommonOptions;

    #[test]
    fn test_cli_overrides_win_over_file_values() {
        let file = FileConfig {
            web2llm: Web2llmSection {
                max_tokens: Some(400),
                ordered: Some(false),
                ..Default::default()
            },
            ..Default::default()
        };

        let options = CommonOptions {
            max_tokens: Some(900),
            ordered: true,
            ..Default::default()
        };

        let resolved = resolve_web2llm_config(&file, &options);
        assert_eq!(resolved.max_tokens, 900);
        assert!(resolved.ordered);
    }
}
