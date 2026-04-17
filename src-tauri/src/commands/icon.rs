use anyhow::Result;

use crate::paths::ManagedPaths;

pub async fn store_icon(
    paths: &ManagedPaths,
    webapp_id: &str,
    url: &str,
    uploaded_icon_data_url: Option<&str>,
    icon_source_url: Option<&str>,
    timeout_secs: u64,
    max_bytes: u64,
) -> Result<Option<String>> {
    crate::icons::ensure_icon(
        paths,
        webapp_id,
        url,
        uploaded_icon_data_url,
        icon_source_url,
        timeout_secs,
        max_bytes,
    )
    .await
}
