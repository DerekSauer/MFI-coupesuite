use chrono::Local;
use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::ghostscript::print_to_printer;
use coupesuite_shared::htmltopdf::{HtmlToPdf, PaperOrientation, PaperSize};
use coupesuite_shared::{database, settings::Settings, templates};
use std::io::Write;
use tera::Context;

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

    // Load HTML templates
    let tera = templates::load_templates().await?;

    // Headless browers used to convert our HTML file to PDF.
    let pdf_renderer = HtmlToPdf::new(Some(&settings.chromium.location)).await?;

    for lot in cmd_line_args.lots.split(',') {
        // SIGM's lot numbers are numeric
        let lot: i32 = match lot.parse() {
            Ok(good_lot) => good_lot,
            Err(_) => anyhow::bail!(
                "Numéro de lot invalide. Les numéros de lot doivent être des chiffres."
            ),
        };

        // Grab the label's data from the database
        let label_data = label_data::LabelData::from_lot(lot, &db_pool).await?;

        // Load label data into the templating engine
        let mut context = Context::from_serialize(&label_data)?;
        context.insert("working_dir", &working_dir);
        context.insert("date_stamp", &Local::now().date().naive_local().to_string());

        // Render the label with HTML place holders filled in with real data
        let html = templates::render_template("labels/label.html", &tera, &context).await?;

        // Cache the rendered HTML to disk
        let html_cache = temp_dir.join(format!("CSLabel {}.html", label_data.project_number));
        {
            let mut file = std::fs::File::create(&html_cache)?;
            file.write_all(html.as_bytes())?;
        }

        // Convert the rendered HTML to PDF
        let temp_path = &temp_dir.join(format!("CSLabel {}.pdf", label_data.project_number));
        pdf_renderer
            .save_pdf(
                &html_cache,
                &PaperSize::CSLabel,
                &PaperOrientation::Portrait,
                &temp_path,
            )
            .await?;

        // Label quantities are rounded up to the nearest multiple of pages
        let quantity =
            u32::try_from((label_data.print_quantity / settings.etiquette.multiple + 1) * 6)?;

        // Print the PDF
        print_to_printer(
            &temp_path.to_string_lossy(),
            quantity,
            &settings.etiquette.nom_imprimante,
            &PaperSize::CSLabel,
            &settings.ghostscript.location,
        )?;
    }

    Ok(())
}
