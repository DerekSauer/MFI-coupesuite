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
    print_quantity: Option<u32>,
    save_local_pdf: bool,
    app_settings: &Settings,
    print_settings: &PrintSettings<'_>,
) -> anyhow::Result<()> {
    // Override default print quantity if needed
    let print_quantity = match print_quantity {
        Some(quantity) => quantity,
        None => app_settings.bon.copies_defaut,
    };

    // Not much point in doing any of this if the users doesn't want to save the PDF
    // and has a print quantity of zero
    if print_quantity > 0 || save_local_pdf {
        // Grab the bon de transfert's data from the database
        let (mut bon_data, sku_data) =
            BonData::from_lot(lot_number, print_settings.db_pool).await?;

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
        templates::render_to_file("bon/bon.html", print_settings.tera, &context, &html_path)?;

        // Temporary location to cache rendered PDFs
        let pdf_temp_path = print_settings
            .temp_path
            .join(format!("Bon de transfert - {}.pdf", lot_number));

        // If the user wants a printed form, save the rendered PDF to a temp
        // directory prior to printing with Ghostscript
        let mut path_list: Vec<PathBuf> = Vec::new();
        if print_quantity > 0 {
            path_list.push(pdf_temp_path.clone());
        }

        // If the user wants a PDF copy for themselves save one to the current directory
        if save_local_pdf {
            path_list.push(PathBuf::from(&app_settings.pdf.location).join(format!(
                "./Bon de transfert - {} ({}).pdf",
                &sku_data.sku, lot_number
            )));
        }

        // Convert the rendered HTML to PDF
        print_settings
            .pdf_renderer
            .save_pdf(
                &html_path,
                &PaperSize::Letter,
                &PaperOrientation::Landscape,
                &path_list,
            )
            .await?;

        // Print the PDF
        if print_quantity > 0 {
            print_to_printer(
                pdf_temp_path.to_str().unwrap(),
                print_quantity,
                &app_settings.bon.nom_imprimante,
                &PaperSize::Letter,
                &PaperOrientation::Landscape,
                print_settings.ghostscript_path,
            )?;
        }
    }

    Ok(())
}
