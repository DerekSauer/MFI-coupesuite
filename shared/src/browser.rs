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

/// Size of the page to print.
pub enum PaperSize {
    /// Customer service label (4 x 2 inches).
    CSLabel,

    /// Standard letter paper size (8.5 x 11 inches)
    Letter,

    /// Standard legal paper size (8.5 x 14 inches)
    Legal,

    /// Customer paper size in inches.
    Custom { width: f64, height: f64 },
}

impl PaperSize {
    /// Retrieve the value of each enum type.
    pub fn value(&self) -> (f64, f64) {
        match *self {
            PaperSize::CSLabel => (4.0, 2.0),
            PaperSize::Letter => (8.5, 11.0),
            PaperSize::Legal => (8.5, 14.0),
            PaperSize::Custom { width, height } => (width, height),
        }
    }
}

/// Indicates whether the printed page is in portrait or landscape
/// orientation.
pub enum PaperOrientation {
    /// Portrait orientation.
    Portrait,

    /// Landscape orientation.
    Landscape,
}

pub async fn save_pdf() -> anyhow::Result<()> {
    Ok(())
}
