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
    print_quantity: Option<u32>,
    save_local_pdf: bool,
    app_settings: &Settings,
    print_settings: &PrintSettings<'_>,
) -> anyhow::Result<()> {
    // Override default print quantity if needed
    let print_quantity = match print_quantity {
        Some(quantity) => quantity,
        None => app_settings.dessins.copies_defaut,
    };

    // Not much point in doing any of this if the users doesn't want to save the PDF
    // and has a print quantity of zero
    if print_quantity > 0 || save_local_pdf {
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

        // Temporary location to cache rendered PDFs
        let pdf_temp_path = &print_settings
            .temp_path
            .join(format!("Dessins {}.pdf", lot_number));

        // If the user wants a printed form, save the rendered PDF to a temp
        // directory prior to printing with Ghostscript
        let mut path_list: Vec<PathBuf> = Vec::new();
        if print_quantity > 0 {
            path_list.push(pdf_temp_path.clone());
        }

        // If the user wants a PDF copy for themselves save one to the current directory
        if save_local_pdf {
            path_list.push(PathBuf::from(format!(
                "./Dessins - {} ({}).pdf",
                &sku_data.sku, lot_number
            )));
        }

        // Convert the rendered HTML to PDF
        print_settings
            .pdf_renderer
            .save_pdf(
                &html_path,
                &PaperSize::Legal,
                &PaperOrientation::Landscape,
                &path_list,
            )
            .await?;

        // Print the PDF
        if print_quantity > 0 {
            print_to_printer(
                &pdf_temp_path.to_string_lossy(),
                print_quantity,
                &app_settings.dessins.nom_imprimante,
                &PaperSize::Legal,
                &PaperOrientation::Landscape,
                print_settings.ghostscript_path,
            )?;
        }
    }

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
