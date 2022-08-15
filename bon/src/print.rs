use crate::bon_data::BonData;
use coupesuite_shared::{
    database::SkuInfo,
    ghostscript::print_to_printer,
    htmltopdf::{PaperOrientation, PaperSize},
    print::PrintSettings,
    settings::Settings,
    templates,
};
use std::path::PathBuf;

/// Data used to fill in the HTML template.
#[derive(serde::Serialize, sqlx::FromRow, Debug)]
struct TemplateData<'a> {
    /// General data about the furniture model.
    sku_data: &'a SkuInfo,

    /// Data about each part that must be transfered.
    part_data: &'a Vec<BonData>,

    /// Lot number of this production job.
    lot_number: &'a str,

    /// Working directory of the app.
    working_dir: &'a PathBuf,
}

/// Print a transfer form.
#[allow(dead_code)] // TODO: Remove when main app is complete.
pub async fn print_bon(
    lot_number: &str,
    quantity: Option<u32>,
    app_settings: &Settings,
    print_settings: &PrintSettings<'_>,
) -> anyhow::Result<()> {
    // Grab the bon de transfert's data from the database
    let (mut bon_data, sku_data) = BonData::from_lot(lot_number, print_settings.db_pool).await?;

    // Remove kanban parts if the lot is not a kanban production
    if !sku_data.kanban {
        bon_data.retain(|part| part.machining_time.is_empty());
    }

    // Build HTML template data
    let template_data = TemplateData {
        lot_number,
        part_data: &bon_data,
        sku_data: &sku_data,
        working_dir: &std::env::current_dir()?,
    };

    // Load template data into the template engine
    let context = tera::Context::from_serialize(&template_data)?;

    // Cache the rendered HTML to disk
    let html_path = print_settings
        .temp_path
        .join(format!("Bon de transfert {}.html", lot_number));
    templates::render_to_file("bon/bonm.html", print_settings.tera, &context, &html_path)?;

    // Convert the rendered HTML to PDF
    let pdf_path = &print_settings
        .temp_path
        .join(format!("Bon de transfert {}.pdf", lot_number));
    print_settings
        .pdf_renderer
        .save_pdf(
            &html_path,
            &PaperSize::Letter,
            &PaperOrientation::Landscape,
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
        &PaperSize::Letter,
        print_settings.ghostscript_path,
    )?;

    Ok(())
}
