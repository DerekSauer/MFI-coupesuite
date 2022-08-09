use std::path::PathBuf;

use crate::form_data::FormData;
use coupesuite_shared::{
    database::SkuInfo,
    ghostscript::print_to_printer,
    htmltopdf::{PaperOrientation, PaperSize},
    print::PrintSettings,
    settings::Settings,
    templates,
};

/// Data used to fill in the HTML template.
#[derive(serde::Serialize, sqlx::FromRow, Debug)]
struct TemplateData<'a> {
    /// General data about the furniture model.
    sku_data: &'a SkuInfo,

    /// Data about each part that must be fabricated.
    part_data: &'a Vec<FormData>,

    /// Lot number of this production job.
    lot_number: &'a str,

    /// Working directory of the app.
    working_dir: &'a PathBuf,

    /// Today's date.
    date_stamp: &'a str,
}

/// Print a production tracking form.
pub async fn print_form(
    lot_number: &str,
    quantity: Option<u32>,
    app_settings: &Settings,
    print_settings: &PrintSettings<'_>,
) -> anyhow::Result<()> {
    // Grab the label's data from the database
    let (form_data, sku_data) = FormData::from_lot(lot_number, print_settings.db_pool).await?;

    // Build template data
    let template_data = TemplateData {
        sku_data: &sku_data,
        part_data: &form_data,
        lot_number: lot_number,
        working_dir: &std::env::current_dir()?,
        date_stamp: &chrono::Local::now().date().naive_local().to_string(),
    };

    // Load label data into the templating engine
    let context = tera::Context::from_serialize(&template_data)?;

    // Cache the rendered HTML to disk
    let html_path = print_settings
        .temp_path
        .join(format!("Prod Form {}.html", lot_number));
    templates::render_to_file("form/form.html", print_settings.tera, &context, &html_path)?;

    println!("HTML: {}", &html_path.to_string_lossy());
    let mut derp = String::new();
    std::io::stdin().read_line(&mut derp)?;

    // Convert the rendered HTML to PDF
    let pdf_path = &print_settings
        .temp_path
        .join(format!("Prod Form {}.pdf", lot_number));
    print_settings
        .pdf_renderer
        .save_pdf(
            &html_path,
            &PaperSize::CSLabel,
            &PaperOrientation::Portrait,
            &pdf_path,
        )
        .await?;

    // Override default print quantity if needed
    let quantity = match quantity {
        Some(quantity) => quantity,
        None => {
            if sku_data.painted_parts {
                app_settings.formulaire.copies_mdf
            } else {
                app_settings.formulaire.copies_defaut
            }
        }
    };

    // Print the PDF
    print_to_printer(
        &pdf_path.to_string_lossy(),
        quantity,
        print_settings.printer_name,
        &PaperSize::CSLabel,
        print_settings.ghostscript_path,
    )?;

    Ok(())
}
