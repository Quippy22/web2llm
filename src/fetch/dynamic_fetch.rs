use crate::error::{Result, Web2llmError};
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use tempfile::tempdir;
use tokio::sync::OnceCell;
use url::Url;

/// Fetches the rendered HTML of a page using a shared headless Chromium browser.
///
/// This is necessary for Single Page Applications (SPAs) and sites that
/// rely heavily on JavaScript to render content.
#[inline(always)]
pub(crate) async fn get_html(url: &Url, browser_cell: &OnceCell<Browser>) -> Result<String> {
    // 1. Get or initialize the shared browser instance
    let browser = browser_cell
        .get_or_try_init(|| async {
            // Create a persistent temp dir for the shared browser instance
            let tmp_dir = tempdir()
                .map_err(|e| Web2llmError::Http(format!("Failed to create temp dir: {}", e)))?;

            // Launch the browser once. We keep the tmp_dir to ensure it stays alive for the session.
            let (browser, mut handler) = Browser::launch(
                BrowserConfig::builder()
                    .no_sandbox()
                    .user_data_dir(tmp_dir.keep())
                    .build()
                    .map_err(|e| {
                        Web2llmError::Http(format!("Failed to build browser config: {}", e))
                    })?,
            )
            .await
            .map_err(|e| Web2llmError::Http(format!("Failed to launch browser: {}", e)))?;

            // Start the CDP handler in the background
            tokio::spawn(async move {
                while let Some(h) = handler.next().await {
                    if h.is_err() {
                        break;
                    }
                }
            });

            Ok::<Browser, Web2llmError>(browser)
        })
        .await?;

    // 2. Open a new tab (page) for this specific request
    let page = browser
        .new_page(url.as_str())
        .await
        .map_err(|e| Web2llmError::Http(format!("Failed to create page: {}", e)))?;

    // 3. Wait for the browser's navigation to settle.
    page.wait_for_navigation()
        .await
        .map_err(|e| Web2llmError::Http(format!("Navigation failed: {}", e)))?;

    // 4. Extract the rendered HTML from the browser's DOM
    let html = page
        .content()
        .await
        .map_err(|e| Web2llmError::Http(format!("Failed to get content: {}", e)))?;

    // 5. Close ONLY the tab. The shared browser stays open for the next request.
    page.close().await.ok();

    Ok(html)
}
