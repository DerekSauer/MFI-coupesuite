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
    print_quantity: Option<u32>,
    save_local_pdf: bool,
    app_settings: &Settings,
    print_settings: &PrintSettings<'_>,
) -> anyhow::Result<()> {
    // Grab the label's data from the database
    let (mut form_data, sku_data) = FormData::from_lot(lot_number, print_settings.db_pool).await?;

    // Override default print quantity if needed
    let print_quantity = match print_quantity {
        Some(quantity) => quantity,
        None => {
            if sku_data.painted_parts {
                app_settings.formulaire.copies_mdf
            } else {
                app_settings.formulaire.copies_defaut
            }
        }
    };

    // Skip processing if the user doesn't want any documents
    if print_quantity > 0 || save_local_pdf {
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

        // Temporary location to cache rendered PDFs
        let pdf_temp_path = &print_settings
            .temp_path
            .join(format!("Prod Form {}.pdf", lot_number));

        // If the user wants a printed form, save the rendered PDF to a temp
        // directory prior to printing with Ghostscript
        let mut path_list: Vec<PathBuf> = Vec::new();
        if print_quantity > 0 {
            path_list.push(pdf_temp_path.clone());
        }

        // If the user wants a PDF copy for themselves save one to the current directory
        if save_local_pdf {
            path_list.push(PathBuf::from(format!(
                "Formulaire de production - {} ({}).pdf",
                &sku_data.sku, lot_number
            )));
        }

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
                &path_list,
            )
            .await?;

        // Print the PDF
        if print_quantity > 0 {
            print_to_printer(
                &pdf_path.to_string_lossy(),
                print_quantity,
                &app_settings.formulaire.nom_imprimante,
                &PaperSize::Letter,
                &PaperOrientation::Portrait,
                print_settings.ghostscript_path,
            )?;
        }
    }

    Ok(())
}
