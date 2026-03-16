use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use tempfile::tempdir;
use url::Url;
use crate::error::{Result, Web2llmError};

/// Fetches the rendered HTML of a page using a headless Chromium browser.
///
/// This is necessary for Single Page Applications (SPAs) and sites that
/// rely heavily on JavaScript to render content.
#[inline(always)]
pub(crate) async fn get_html(url: &Url) -> Result<String> {
    // 1. Create a unique temporary directory for this browser instance's 
    // data to prevent "SingletonLock" conflicts during concurrent fetching.
    let tmp_dir = tempdir()
        .map_err(|e| Web2llmError::Http(format!("Failed to create temp dir: {}", e)))?;

    // 2. Launch a browser. We use no-sandbox which is often 
    // needed in Linux/Docker environments.
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .no_sandbox()
            .user_data_dir(tmp_dir.path())
            .build()
            .map_err(|e| Web2llmError::Http(format!("Failed to build browser config: {}", e)))?,
    )
    .await
    .map_err(|e| Web2llmError::Http(format!("Failed to launch browser: {}", e)))?;

    // 3. The handler MUST run in the background to process the CDP messages.
    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() { break; }
        }
    });

    // 4. Open a tab and navigate
    let page = browser
        .new_page(url.as_str())
        .await
        .map_err(|e| Web2llmError::Http(format!("Failed to create page: {}", e)))?;

    // 5. Wait for the browser's navigation to settle.
    page.wait_for_navigation()
        .await
        .map_err(|e| Web2llmError::Http(format!("Navigation failed: {}", e)))?;

    // 6. Extract the rendered HTML from the browser's DOM
    let html = page
        .content()
        .await
        .map_err(|e| Web2llmError::Http(format!("Failed to get content: {}", e)))?;

    // 7. Clean up: close the browser, wait for handler, and temp dir will be deleted.
    browser.close().await.ok();
    handle.abort();
    let _ = tmp_dir; // Ensure tmp_dir is not dropped until the end

    Ok(html)
}
