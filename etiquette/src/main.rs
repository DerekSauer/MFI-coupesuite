use chromiumoxide::cdp::browser_protocol::page::PrintToPdfParams;
use chromiumoxide::{Browser, BrowserConfig};
use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::database::{self, verify_lot};
use coupesuite_shared::settings::Settings;
use futures::StreamExt;
use tera::{Context, Tera};

mod cmd_line;
mod label_data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let working_dir = std::env::current_dir()?;
    let settings = Settings::load(&working_dir.join("coupesuite.toml"))?;
    let db_pool = database::get_database_pool(&settings.database).await?;

    // Temporary directory for generated files
    let temp_dir = tempfile::tempdir()?;
    let temp_dir = temp_dir.path();

    // Open a headless chromium browser
    let browser_config = BrowserConfig::with_executable(&settings.chromium.location);
    let (browser, mut handler) = Browser::launch(browser_config).await?;

    // Web socket handler
    let _ = tokio::task::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

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
        let label_data = label_data::LabelData::from_lot(lot, &db_pool).await?;

        // Path to the HTML template
        let template_path = format!("{}/www/**/*.html", working_dir.to_string_lossy());

        // Load the HTML templat&e
        let tera = Tera::new(&template_path)?;

        // Load label data into the templating engine
        let mut context = Context::from_serialize(&label_data)?;
        context.insert("working_dir", &working_dir);

        // Render the label with HTML place holders filled in with real data
        let html = tera.render("label/label.html", &context)?;

        // Load the template so that CSS is processed, then replace with rendered HTML
        let page = browser
            .new_page(
                working_dir
                    .join("www")
                    .join("label")
                    .join("label.html")
                    .to_string_lossy(),
            )
            .await?;
        page.set_content(&html).await?;

        // Rendering parameters for the PDF file
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

        // Use the web browser to render a PDF of the webpage and save it to a temp directory
        let temp_path = &temp_dir.join(format!(
            "Etiquette Service Client - {}.pdf",
            label_data.project_number
        ));
        page.save_pdf(pdf_params, &temp_path).await?;

        // Label quantities are rounded up to the nearest multiple of pages
        let quantity =
            u32::try_from((label_data.print_quantity / settings.etiquette.multiple + 1) * 6)?;

        println!("Print Quantity: {}\n", quantity);

        // Print the PDF
        coupesuite_shared::ghostscript::print_to_printer(
            &temp_path.to_string_lossy(),
            quantity,
            &settings.etiquette.nom_imprimante,
            (4.0, 2.0),
            &settings.ghostscript.location,
        )?;

        page.close().await?;
    }

    Ok(())
}
