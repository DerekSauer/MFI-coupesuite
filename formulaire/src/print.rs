use crate::form_data::FormData;
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

    /// Data about each part that must be fabricated.
    part_data: &'a Vec<FormData>,

    /// Lot number of this production job.
    lot_number: &'a str,

    /// Working directory of the app.
    working_dir: &'a PathBuf,

    /// Path to the image displayed on the form
    image_path: &'a PathBuf,
}

/// Print a production tracking form.
#[allow(dead_code)] // TODO: Remove when main app is complete.
pub async fn print_form(
    lot_number: &str,
    quantity: Option<u32>,
    app_settings: &Settings,
    print_settings: &PrintSettings<'_>,
) -> anyhow::Result<()> {
    // Grab the label's data from the database
    let (mut form_data, sku_data) = FormData::from_lot(lot_number, print_settings.db_pool).await?;

    // Remove kanban parts if the lot is not a kanban production
    if !sku_data.kanban {
        form_data.retain(|x| x.machining_time.is_empty());
    }

    // Find the form's furniture image prefering SVG files over PNG over JPG
    // If no image is found use the placeholder and bail if that can't be found
    let mut image_path =
        std::path::Path::new(&app_settings.formulaire.fichier_images).join(&sku_data.sku);
    image_path.set_extension("svg");

    if !image_path.exists() {
        image_path.set_extension("png");
    }
    if !image_path.exists() {
        image_path.set_extension("jpg");
    }
    if !image_path.exists() {
        image_path.set_file_name("Placeholder");
        image_path.set_extension("png");
    }
    if !image_path.exists() {
        anyhow::bail!("Image du meuble introuvable: {}", &sku_data.sku);
    }

    // Build template data
    let template_data = TemplateData {
        sku_data: &sku_data,
        part_data: &form_data,
        lot_number,
        working_dir: &std::env::current_dir()?,
        image_path: &image_path,
    };

    // Load label data into the templating engine
    let context = tera::Context::from_serialize(&template_data)?;

    // Cache the rendered HTML to disk
    let html_path = print_settings
        .temp_path
        .join(format!("Prod Form {}.html", lot_number));
    templates::render_to_file("form/form.html", print_settings.tera, &context, &html_path)?;

    // Convert the rendered HTML to PDF
    let pdf_path = &print_settings
        .temp_path
        .join(format!("Prod Form {}.pdf", lot_number));
    print_settings
        .pdf_renderer
        .save_pdf(
            &html_path,
            &PaperSize::Letter,
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
        &app_settings.formulaire.nom_imprimante,
        &PaperSize::Letter,
        &PaperOrientation::Portrait,
        print_settings.ghostscript_path,
    )?;

    Ok(())
}
