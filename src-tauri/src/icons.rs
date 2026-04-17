use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use image::imageops::FilterType;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use url::Url;

use crate::paths::{display_path, ManagedPaths};
use crate::utils::normalize_url;

pub async fn ensure_icon(
    paths: &ManagedPaths,
    webapp_id: &str,
    page_url: &str,
    uploaded_icon_data_url: Option<&str>,
    icon_source_url: Option<&str>,
    timeout_secs: u64,
    max_bytes: u64,
) -> Result<Option<String>> {
    if let Some(data_url) = uploaded_icon_data_url {
        return save_uploaded_icon(paths, webapp_id, data_url).map(Some);
    }

    if let Some(source_url) = icon_source_url {
        match download_icon_from_url(paths, webapp_id, source_url, timeout_secs, max_bytes).await {
            Ok(path) => return Ok(Some(path)),
            Err(e) => {
                log::warn!("Failed to download icon from explicit source URL {source_url}: {e}");
            }
        }
    }

    match fetch_icon_from_site(paths, webapp_id, page_url, timeout_secs, max_bytes).await {
        Ok(path) => Ok(Some(path)),
        Err(e) => {
            log::warn!("Failed to fetch icon from site {page_url}: {e}");
            Ok(None)
        }
    }
}

pub fn remove_icon(paths: &ManagedPaths, webapp_id: &str) -> Result<()> {
    let path = icon_file_path(paths, webapp_id);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn icon_data_url_from_path(path: Option<&str>) -> Result<Option<String>> {
    let Some(path) = path else {
        return Ok(None);
    };

    let bytes = fs::read(path)?;
    Ok(Some(format!(
        "data:image/png;base64,{}",
        BASE64.encode(bytes)
    )))
}

pub fn save_uploaded_icon_data_url(
    paths: &ManagedPaths,
    webapp_id: &str,
    data_url: &str,
) -> Result<String> {
    save_uploaded_icon(paths, webapp_id, data_url)
}

fn save_uploaded_icon(paths: &ManagedPaths, webapp_id: &str, data_url: &str) -> Result<String> {
    let payload = data_url
        .split_once(',')
        .map(|(_, data)| data)
        .context("Invalid icon data URL.")?;
    let bytes = BASE64.decode(payload)?;
    save_png_icon(paths, webapp_id, &bytes)
}

fn build_client(timeout_secs: u64) -> Result<Client> {
    Client::builder()
        .user_agent("webapp-manager/0.1")
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .context("Failed to build HTTP client.")
}

async fn fetch_icon_from_site(
    paths: &ManagedPaths,
    webapp_id: &str,
    page_url: &str,
    timeout_secs: u64,
    max_bytes: u64,
) -> Result<String> {
    let normalized = normalize_url(page_url)?;
    let client = build_client(timeout_secs)?;

    if let Some(path) = try_manifest_icon(&client, paths, webapp_id, &normalized, max_bytes).await?
    {
        return Ok(path);
    }

    if let Some(path) =
        try_html_icon_links(&client, paths, webapp_id, &normalized, max_bytes).await?
    {
        return Ok(path);
    }

    let favicon = normalized.join("/favicon.ico")?;
    if let Ok(path) = download_and_store_icon(&client, paths, webapp_id, favicon, max_bytes).await {
        return Ok(path);
    }

    let fallback = normalized.join("/favicon.png")?;
    download_and_store_icon(&client, paths, webapp_id, fallback, max_bytes).await
}

async fn download_icon_from_url(
    paths: &ManagedPaths,
    webapp_id: &str,
    icon_url: &str,
    timeout_secs: u64,
    max_bytes: u64,
) -> Result<String> {
    let client = build_client(timeout_secs)?;
    let normalized = Url::parse(icon_url)?;
    download_and_store_icon(&client, paths, webapp_id, normalized, max_bytes).await
}

async fn try_manifest_icon(
    client: &Client,
    paths: &ManagedPaths,
    webapp_id: &str,
    page_url: &Url,
    max_bytes: u64,
) -> Result<Option<String>> {
    let manifest_candidates = ["/manifest.json", "/site.webmanifest"];
    for candidate in manifest_candidates {
        let manifest_url = page_url.join(candidate)?;
        let response = client.get(manifest_url).send().await?;
        if !response.status().is_success() {
            continue;
        }

        // Guard against oversized manifest responses.
        if let Some(len) = response.content_length() {
            if len > max_bytes {
                continue;
            }
        }
        let body = response.bytes().await?;
        if body.len() as u64 > max_bytes {
            continue;
        }
        let manifest: Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let Some(icons) = manifest.get("icons").and_then(Value::as_array) else {
            continue;
        };

        let mut largest_icon_url: Option<Url> = None;
        let mut largest_size = 0;

        for item in icons {
            let Some(src) = item.get("src").and_then(Value::as_str) else {
                continue;
            };
            let size = item
                .get("sizes")
                .and_then(Value::as_str)
                .and_then(parse_largest_size)
                .unwrap_or_default();
            let candidate_url = page_url.join(src)?;
            if size >= largest_size {
                largest_size = size;
                largest_icon_url = Some(candidate_url);
            }
        }

        if let Some(icon_url) = largest_icon_url {
            return download_and_store_icon(client, paths, webapp_id, icon_url, max_bytes)
                .await
                .map(Some);
        }
    }

    Ok(None)
}

async fn try_html_icon_links(
    client: &Client,
    paths: &ManagedPaths,
    webapp_id: &str,
    page_url: &Url,
    max_bytes: u64,
) -> Result<Option<String>> {
    let response = client.get(page_url.clone()).send().await?;
    // Guard against oversized HTML responses.
    if let Some(len) = response.content_length() {
        if len > max_bytes {
            return Ok(None);
        }
    }
    let body_bytes = response.bytes().await?;
    if body_bytes.len() as u64 > max_bytes {
        return Ok(None);
    }
    let body = String::from_utf8_lossy(&body_bytes);
    let icon_urls = {
        let document = Html::parse_document(&body);
        let selector = Selector::parse("link").expect("valid selector");
        let mut icon_urls = Vec::new();

        for link in document.select(&selector) {
            let rel = link
                .value()
                .attr("rel")
                .unwrap_or_default()
                .to_ascii_lowercase();
            if !(rel.contains("icon") || rel.contains("apple-touch-icon")) {
                continue;
            }

            let Some(href) = link.value().attr("href") else {
                continue;
            };
            icon_urls.push(page_url.join(href)?);
        }

        icon_urls
    };

    for icon_url in icon_urls {
        if let Ok(path) =
            download_and_store_icon(client, paths, webapp_id, icon_url, max_bytes).await
        {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

async fn download_and_store_icon(
    client: &Client,
    paths: &ManagedPaths,
    webapp_id: &str,
    icon_url: Url,
    max_bytes: u64,
) -> Result<String> {
    let response = client.get(icon_url.clone()).send().await?;

    // Reject if Content-Length header declares the file is too large.
    if let Some(len) = response.content_length() {
        if len > max_bytes {
            return Err(anyhow!(
                "Icon at {icon_url} is too large ({len} bytes, max {max_bytes})."
            ));
        }
    }

    let bytes = response.bytes().await?;

    // Guard against servers that omit Content-Length but return huge bodies.
    if bytes.len() as u64 > max_bytes {
        return Err(anyhow!(
            "Icon at {icon_url} exceeded size limit after download ({} bytes, max {max_bytes}).",
            bytes.len()
        ));
    }

    save_png_icon(paths, webapp_id, bytes.as_ref())
}

fn save_png_icon(paths: &ManagedPaths, webapp_id: &str, bytes: &[u8]) -> Result<String> {
    let image =
        image::load_from_memory(bytes).context("The fetched icon format could not be decoded.")?;
    let resized = image.resize_to_fill(128, 128, FilterType::Lanczos3);
    let path = icon_file_path(paths, webapp_id);
    resized
        .save(&path)
        .with_context(|| format!("Failed to save icon to {}.", path.display()))?;
    Ok(display_path(&path))
}

fn icon_file_path(paths: &ManagedPaths, webapp_id: &str) -> PathBuf {
    paths.icons_dir.join(format!("{webapp_id}.png"))
}

fn parse_largest_size(value: &str) -> Option<u32> {
    value
        .split_whitespace()
        .filter_map(|entry| entry.split_once('x'))
        .filter_map(|(width, height)| {
            let width = width.parse::<u32>().ok()?;
            let height = height.parse::<u32>().ok()?;
            Some(width.max(height))
        })
        .max()
}
