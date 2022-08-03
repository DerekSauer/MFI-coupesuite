use crate::label_data;
use coupesuite_shared::{
    ghostscript::print_to_printer,
    htmltopdf::{HtmlToPdf, PaperOrientation, PaperSize},
    templates,
};
use sqlx::PgPool;
use tera::Tera;

/// Common settings used for printing labels.
#[derive(Debug)]
pub struct PrintSettings<'a> {
    /// Windows printer name of the label printer to use.
    pub printer_name: &'a str,

    /// Path to the `gswin64c.exe` Ghostscript binary.
    pub ghostscript_path: &'a str,

    /// Print copies rounded up to this number.
    pub print_multiple: i32,

    /// HTML template engine containing the label's template.
    pub tera: &'a Tera,

    /// HTML to PDF renderer.
    pub pdf_renderer: &'a HtmlToPdf,

    /// Temporary path to store intermediary files.
    pub temp_path: &'a std::path::Path,

    /// Connection pool for the database.
    pub db_pool: &'a PgPool,
}

/// Print a customer service label.
pub async fn print_label(
    lot_number: &str,
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

    // Label quantities are rounded up to the nearest multiple of pages
    let quantity =
        u32::try_from((label_data.print_quantity / print_settings.print_multiple + 1) * 6)?;

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
