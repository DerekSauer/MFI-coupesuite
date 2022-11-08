use chromiumoxide::{cdp::browser_protocol::page::PrintToPdfParams, Browser, BrowserConfig};
use futures::StreamExt;
use std::path::Path;
use tokio::task::JoinHandle;

/// Tool to convert HTML documents into PDF files.
#[derive(Debug)]
pub struct HtmlToPdf {
    /// A headless Chromium web brower that performs the conversion.
    browser: Browser,

    /// Websocket use to communicate with the browser.
    #[allow(dead_code)] // Actually read but hidden in a thread pool
    websocket_thread: JoinHandle<()>,
}

impl HtmlToPdf {
    /// Create a new headless browser instance.
    ///
    /// ## Parameters
    /// - `chromium_path`: File path to the Chromium browser you wish to use. If no path is
    /// passed in, we'll attempt to find a suitable browser on the system and fail if none
    /// can be found.
    pub async fn new(chromium_path: Option<&impl AsRef<Path>>) -> anyhow::Result<Self> {
        let (browser, mut handler) = match chromium_path {
            Some(path) => {
                let browser_config = BrowserConfig::with_executable(path);
                Browser::launch(browser_config).await?
            }
            None => {
                let browser_config = match BrowserConfig::builder().build() {
                    Ok(config) => config,
                    Err(err) => anyhow::bail!(err),
                };
                Browser::launch(browser_config).await?
            }
        };

        let websocket_thread = tokio::task::spawn(async move {
            loop {
                let _ = handler.next().await.unwrap();
            }
        });

        Ok(Self {
            browser,
            websocket_thread,
        })
    }

    /// Convert an HTML document to PDF using the headless browser.
    pub async fn save_pdf(
        &self,
        input_html_path: &impl AsRef<Path>,
        paper_size: &PaperSize,
        paper_orientation: &PaperOrientation,
        output_pdf_path: &[impl AsRef<Path>],
    ) -> anyhow::Result<()> {
        // Open the HTML file in the browser
        let page = self
            .browser
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
        for path in output_pdf_path {
            let pdf_params = pdf_params.clone();
            page.save_pdf(pdf_params, path).await?;
        }
        page.close().await?;

        Ok(())
    }
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
