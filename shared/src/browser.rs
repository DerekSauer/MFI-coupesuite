use crate::settings::Settings;
use chromiumoxide::{
    cdp::browser_protocol::page::PrintToPdfParams, Browser, BrowserConfig, Handler,
};
use std::path::Path;

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

    /// Custom paper size in inches.
    Custom { width: f64, height: f64 },
}

impl PaperSize {
    /// Retrieve the width and height of the chosen enum value.
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

impl PaperOrientation {
    /// Chrome Dev Tools specifies paper orientation as a bool,
    /// where landscape is true and portrait is false
    pub fn chromium_value(&self) -> bool {
        match *self {
            PaperOrientation::Portrait => false,
            PaperOrientation::Landscape => true,
        }
    }
}

/// Convert an HTML document to PDF using the headless browser.
pub async fn save_pdf(
    input_html_path: &impl AsRef<Path>,
    paper_size: &PaperSize,
    paper_orientation: &PaperOrientation,
    output_pdf_path: &impl AsRef<Path>,
    browser: &Browser,
) -> anyhow::Result<()> {
    // Open the HTML file in the browser
    let page = browser
        .new_page(input_html_path.as_ref().to_string_lossy())
        .await?;

    // Setup PDF rendering params
    let (width, height) = paper_size.value();
    let pdf_params = PrintToPdfParams {
        display_header_footer: false.into(),
        print_background: false.into(),
        paper_width: width.into(),
        paper_height: height.into(),
        landscape: paper_orientation.chromium_value().into(),
        scale: Some(1.0),
        ..Default::default()
    };

    // Save the PDF to disk and cleanup
    page.save_pdf(pdf_params, output_pdf_path).await?;
    page.close().await?;

    Ok(())
}
