use anyhow::{anyhow, Result};
use url::Url;

pub fn normalize_url(value: &str) -> Result<Url> {
    let candidate = if value.contains("://") {
        value.trim().to_owned()
    } else {
        format!("https://{}", value.trim())
    };
    Url::parse(&candidate).map_err(|_| anyhow!("Enter a valid URL."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepends_https_when_scheme_is_missing() {
        let url = normalize_url("example.com").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("example.com"));
    }

    #[test]
    fn leaves_existing_scheme_intact() {
        let url = normalize_url("http://example.com").unwrap();
        assert_eq!(url.scheme(), "http");
    }

    #[test]
    fn trims_whitespace() {
        let url = normalize_url("  example.com  ").unwrap();
        assert_eq!(url.host_str(), Some("example.com"));
    }

    #[test]
    fn returns_error_for_invalid_url() {
        assert!(normalize_url("not a url!!").is_err());
    }
}
