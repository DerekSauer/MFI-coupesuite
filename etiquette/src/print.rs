use crate::label_data;
use coupesuite_shared::{
    ghostscript::print_to_printer,
    htmltopdf::{PaperOrientation, PaperSize},
    print::PrintSettings,
    settings::Settings,
    templates,
};

/// Print a customer service label.
pub async fn print_label(
    lot_number: &str,
    quantity: Option<u32>,
    app_settings: &Settings,
    print_settings: &PrintSettings<'_>,
) -> anyhow::Result<()> {
    // Grab the label's data from the database
    let label_data = label_data::LabelData::from_lot(lot_number, print_settings.db_pool).await?;

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
            &pdf_path,
        )
        .await?;

    // If the user asked for a specific quantity, print that
    // Otherwise print what the label's data demands plus a few extra
    let quantity: u32 = match quantity {
        Some(amount) => amount,
        None => u32::try_from(label_data.print_quantity)? + app_settings.etiquette.extra,
    };

    // Print the PDF
    print_to_printer(
        &pdf_path.to_string_lossy(),
        quantity,
        &app_settings.etiquette.nom_imprimante,
        &PaperSize::CSLabel,
        &PaperOrientation::Portrait,
        print_settings.ghostscript_path,
    )?;

    Ok(())
}
