use crate::{Result, Web2llmError};
use std::net::IpAddr;
use url::Url;

const ALLOWED_SCHEMES: &[&str] = &["http", "https"];

/// Validates a raw URL string and returns a typed [`Url`] on success.
///
/// Checks are performed in order:
/// 1. The string must be parseable as a valid URL
/// 2. The scheme must be `http` or `https`
/// 3. The URL must have a host
/// 4. If `block_private_hosts` is `true`, the host must not be a private,
///    loopback, or link-local address
///
/// This function is synchronous and makes no network calls.
/// It is always the first step in the pre-flight stage.
///
/// # Errors
///
/// Returns [`Web2llmError::InvalidUrl`] if any check fails.
/// The error message includes the specific reason for rejection.
pub(crate) fn validate(raw: &str, block_private_hosts: bool) -> Result<Url> {
    let url = Url::parse(raw).map_err(|_| Web2llmError::InvalidUrl(raw.to_string()))?;
    if !ALLOWED_SCHEMES.contains(&url.scheme()) {
        return Err(Web2llmError::InvalidUrl(format!(
            "scheme '{}' is not allowed",
            url.scheme()
        )));
    }
    if url.host_str().is_none() {
        return Err(Web2llmError::InvalidUrl("URL has no host".to_string()));
    }
    if is_private_host(&url) && block_private_hosts {
        return Err(Web2llmError::InvalidUrl(
            "private or loopback addresses are not allowed".to_string(),
        ));
    }
    Ok(url)
}

/// Returns `true` if the host of `url` is a private, loopback, or link-local address.
///
/// Checks both numeric IP addresses and well-known loopback hostnames.
/// If the host is a domain name that is not `localhost` or `localhost.localdomain`,
/// this function returns `false` — DNS resolution is out of scope here.
fn is_private_host(url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return false;
    };
    if let Ok(ip) = host.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
            IpAddr::V6(v6) => v6.is_loopback(),
        };
    }
    matches!(host, "localhost" | "localhost.localdomain")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_https() {
        assert!(validate("https://example.com", true).is_ok());
    }

    #[test]
    fn test_validate_valid_http() {
        assert!(validate("http://example.com", true).is_ok());
    }

    #[test]
    fn test_validate_rejected_schemes() {
        assert!(validate("ftp://example.com", true).is_err());
        assert!(validate("file:///tmp/test.txt", true).is_err());
    }

    #[test]
    fn test_validate_garbage() {
        assert!(validate("not-a-url", true).is_err());
    }

    #[test]
    fn test_validate_loopback_ip() {
        assert!(validate("http://127.0.0.1", true).is_err());
    }

    #[test]
    fn test_validate_private_ip() {
        assert!(validate("http://192.168.1.1", true).is_err());
    }

    #[test]
    fn test_validate_localhost_blocked() {
        assert!(validate("http://localhost", true).is_err());
    }

    #[test]
    fn test_validate_localhost_allowed() {
        assert!(validate("http://localhost", false).is_ok());
    }
}
