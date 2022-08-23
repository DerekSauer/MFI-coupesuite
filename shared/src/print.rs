use crate::htmltopdf::HtmlToPdf;
use sqlx::PgPool;
use tera::Tera;

/// Common settings used for printing labels.
#[derive(Debug)]
pub struct PrintSettings<'a> {
    /// Path to the `gswin64c.exe` Ghostscript binary.
    pub ghostscript_path: &'a str,

    /// HTML template engine containing the label's template.
    pub tera: &'a Tera,

    /// HTML to PDF renderer.
    pub pdf_renderer: &'a HtmlToPdf,

    /// Temporary path to store intermediary files.
    pub temp_path: &'a std::path::Path,

    /// Connection pool for the database.
    pub db_pool: &'a PgPool,
}
