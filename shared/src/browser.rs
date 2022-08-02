use crate::settings::Settings;
use chromiumoxide::{Browser, BrowserConfig, Handler};

/// Uses Chrome Dev Protocols to open a new headless browser instance.
///
/// ## Remarks
/// Any code using this function must setup an event pump for the browser's web
/// socket handler or the headless browser will not function.
///
/// ```ignore
/// let (browser, mut handler) = open_browser(&settings).await?;
///
/// // Web socket handler
/// let _ = tokio::task::spawn(async move {
///     loop {
///         let _ = handler.next().await.unwrap();
///     }
/// });
/// ```
pub async fn open_browser(settings: &Settings) -> anyhow::Result<(Browser, Handler)> {
    let browser_config = BrowserConfig::with_executable(&settings.chromium.location);
    Ok(Browser::launch(browser_config).await?)
}
