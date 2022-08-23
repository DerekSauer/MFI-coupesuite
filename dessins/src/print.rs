use crate::dessin_data::DessinData;
use coupesuite_shared::{
    database::SkuInfo,
    ghostscript::print_to_printer,
    htmltopdf::{PaperOrientation, PaperSize},
    print::PrintSettings,
    settings::Settings,
    templates,
};
use std::path::{Path, PathBuf};

/// Data used to fill in the HTML template.
#[derive(serde::Serialize, sqlx::FromRow, Debug)]
struct TemplateData<'a> {
    /// General data about the furniture model.
    sku_data: &'a SkuInfo,

    /// Data about each part that must be transfered.
    part_data: &'a Vec<DessinData>,

    /// Lot number of this production job.
    lot_number: &'a str,

    /// Working directory of the app.
    working_dir: &'a PathBuf,
}

/// Print a transfer form.
#[allow(dead_code)] // TODO: Remove when main app is complete.
pub async fn print_dessins(
    lot_number: &str,
    quantity: Option<u32>,
    app_settings: &Settings,
    print_settings: &PrintSettings<'_>,
) -> anyhow::Result<()> {
    // Grab the drawing data from the database
    let (mut dessin_data, sku_data) =
        DessinData::from_lot(lot_number, print_settings.db_pool).await?;

    // Remove kanban parts if the lot is not a kanban production
    if !sku_data.kanban {
        dessin_data.retain(|part| part.machining_time.is_empty());
    }

    // Add file path to drawing images
    let image_path = std::path::Path::new(&app_settings.dessins.fichier_dessins);
    for part in dessin_data.iter_mut() {
        part.image_path = find_image(&part.part_number, image_path)?
            .to_string_lossy()
            .to_string();
    }

    // Build HTML template data
    let template_data = TemplateData {
        lot_number,
        part_data: &dessin_data,
        sku_data: &sku_data,
        working_dir: &std::env::current_dir()?,
    };

    // Load template data into the template engine
    let context = tera::Context::from_serialize(&template_data)?;

    // Cache the rendered HTML to disk
    let html_path = print_settings
        .temp_path
        .join(format!("Dessins {}.html", lot_number));
    templates::render_to_file(
        "dessins/dessins.html",
        print_settings.tera,
        &context,
        &html_path,
    )?;

    // Convert the rendered HTML to PDF
    let pdf_path = &print_settings
        .temp_path
        .join(format!("Dessins {}.pdf", lot_number));
    print_settings
        .pdf_renderer
        .save_pdf(
            &html_path,
            &PaperSize::Legal,
            &PaperOrientation::Landscape,
            &pdf_path,
        )
        .await?;

    println!(
        "HTML: {}\nPDF: {}\n",
        &html_path.to_str().unwrap(),
        &pdf_path.to_str().unwrap()
    );

    // Override default print quantity if needed
    let quantity = match quantity {
        Some(quantity) => quantity,
        None => app_settings.dessins.copies_defaut,
    };

    // Print the PDF
    print_to_printer(
        &pdf_path.to_string_lossy(),
        quantity,
        print_settings.printer_name,
        &PaperSize::Legal,
        &PaperOrientation::Landscape,
        print_settings.ghostscript_path,
    )?;

    Ok(())
}

/// Try to find the best quality image available for the drawing(s).
/// SVG > PNG > JPG.
fn find_image(part_number: &str, image_root_path: &Path) -> anyhow::Result<PathBuf> {
    let mut image_path = image_root_path.join(part_number);
    image_path.set_extension("svg");

    if !image_path.exists() {
        image_path.set_extension("png");
    }
    if !image_path.exists() {
        image_path.set_extension("jpg");
    }
    if !image_path.exists() {
        image_path.set_file_name("DEFAUT");
        image_path.set_extension("jpg");
    }
    if !image_path.exists() {
        anyhow::bail!("Image du pièce introuvable: {}", part_number);
    } else {
        Ok(image_path)
    }
}
