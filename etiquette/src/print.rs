use crate::label_data;
use coupesuite_shared::{
    ghostscript::print_to_printer,
    htmltopdf::{PaperOrientation, PaperSize},
    print::PrintSettings,
    settings::Settings,
    templates,
};
use std::path::PathBuf;

/// Print a customer service label.
pub async fn print_label(
    lot_number: &str,
    print_quantity: Option<u32>,
    save_local_pdf: bool,
    app_settings: &Settings,
    print_settings: &PrintSettings<'_>,
) -> anyhow::Result<()> {
    // Grab the label's data from the database
    let (label_data, sku_info) =
        label_data::LabelData::from_lot(lot_number, print_settings.db_pool).await?;

    // If the user asked for a specific quantity, print that
    // Otherwise print what the label's data demands plus a few extra
    let print_quantity: u32 = match print_quantity {
        Some(amount) => amount,
        None => u32::try_from(label_data.print_quantity)? + app_settings.etiquette.extra,
    };

    // Bail if not printing or saving a PDF
    if print_quantity > 0 || save_local_pdf {
        // Kanban models don't have labels, just bail
        if sku_info.kanban {
            println!(
            "INFO: Le numéro de lot, {}, est un Kanban, l'impression de l'étiquette a été ignorée.",
            lot_number,
        );
            return Ok(());
        }

        // Load label data into the templating engine
        let mut context = tera::Context::from_serialize(&label_data)?;
        context.insert("working_dir", &std::env::current_dir()?);
        context.insert(
            "date_stamp",
            &chrono::Local::now().date().naive_local().to_string(),
        );

        // Cache the rendered HTML to disk
        let html_path = print_settings
            .temp_path
            .join(format!("CSLabel {}.html", label_data.project_number));
        templates::render_to_file(
            "labels/label.html",
            print_settings.tera,
            &context,
            &html_path,
        )?;

        // Temporary location to cache rendered PDFs
        let pdf_temp_path = &print_settings
            .temp_path
            .join(format!("CSLabel {}.pdf", lot_number));

        // If the user wants a printed form, save the rendered PDF to a temp
        // directory prior to printing with Ghostscript
        let mut path_list: Vec<PathBuf> = Vec::new();
        if print_quantity > 0 {
            path_list.push(pdf_temp_path.clone());
        }

        // If the user wants a PDF copy for themselves save one to the current directory
        if save_local_pdf {
            path_list.push(PathBuf::from(format!(
                "./Etiquette de service - {} ({}).pdf",
                &sku_info.sku, lot_number
            )));
        }

        // Convert the rendered HTML to PDF
        let pdf_path = &print_settings
            .temp_path
            .join(format!("CSLabel {}.pdf", label_data.project_number));
        print_settings
            .pdf_renderer
            .save_pdf(
                &html_path,
                &PaperSize::CSLabel,
                &PaperOrientation::Portrait,
                &path_list,
            )
            .await?;

        // Print the PDF
        if print_quantity > 0 {
            print_to_printer(
                &pdf_path.to_string_lossy(),
                print_quantity,
                &app_settings.etiquette.nom_imprimante,
                &PaperSize::CSLabel,
                &PaperOrientation::Portrait,
                print_settings.ghostscript_path,
            )?;
        }
    }

    Ok(())
}
