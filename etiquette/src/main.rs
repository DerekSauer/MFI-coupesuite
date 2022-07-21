use chromiumoxide::cdp::browser_protocol::page::PrintToPdfParams;
use chromiumoxide::{Browser, BrowserConfig};
use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::database::{self, verify_lot};
use coupesuite_shared::settings::Settings;
use futures::StreamExt;

mod cmd_line;
mod label_data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let working_dir = std::env::current_dir()?;
    let settings = Settings::load(&working_dir.join("coupesuite.toml"))?;
    let db_pool = database::get_database_pool(&settings.database).await?;

    for lot in cmd_line_args.lots.split(',') {
        // SIGM's lot numbers are numeric, make sure of that
        let lot: i32 = match lot.parse() {
            Ok(good_lot) => good_lot,
            Err(_) => anyhow::bail!(
                "Numéro de lot invalide. Les numéros de lot doivent être des chiffres."
            ),
        };

        // verify_lot() will bail if the lot number doesn't exist or the DB fails
        verify_lot(lot, &db_pool).await?;

        // Grab the label's data from the database
        let _label_data = label_data::LabelData::from_lot(lot, &db_pool).await?;

        // TODO: Templatize the HTML file and insert label data into the placeholders
    }

    let browser_config = BrowserConfig::with_executable(&settings.chromium.location);
    let (browser, mut handler) = Browser::launch(browser_config).await?;

    let _ = tokio::task::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    let page = browser
        .new_page("file://C:/Users/DSauer/Source/coupesuite/etiquette/www/label.html")
        .await?;

    let pdf_params = PrintToPdfParams {
        landscape: false.into(),
        display_header_footer: false.into(),
        print_background: false.into(),
        scale: Some(1.0),
        paper_width: Some(4.0),
        paper_height: Some(2.0),
        margin_top: None,
        margin_left: None,
        margin_bottom: None,
        margin_right: None,
        page_ranges: None,
        ignore_invalid_page_ranges: None,
        header_template: None,
        footer_template: None,
        prefer_css_page_size: None,
        transfer_mode: None,
    };

    page.save_pdf(pdf_params, "./test.pdf").await?;

    Ok(())
}

// gswin64c -dSAFER -dBATCH -dNOPAUSE -dNumCopies=5 -dNoCancel -dDEVICEWIDTHPOINTS=288 -dDEVICEHEIGHTPOINTS=144 -dNEWPDF -sDEVICE=mswinpr2 -sOutputFile="%printer%Zebra (Bureau Derek)" .\test.pdf
