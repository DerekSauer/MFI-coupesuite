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
    part_number: &str,
    print_quantity: Option<u32>,
    save_local_pdf: bool,
    app_settings: &Settings,
    print_settings: &PrintSettings<'_>,
) -> anyhow::Result<()> {
    // If the user asked for a specific quantity, print that other wise just one label
    let print_quantity: u32 = print_quantity.unwrap_or(1);

    // Skip this if the user does not want a PDF and no print copies
    if print_quantity > 0 || save_local_pdf {
        // Grab the label's data from the database
        let label_data =
            match label_data::LabelData::from_part(part_number, print_settings.db_pool).await {
                Ok(data) => data,
                Err(_) => anyhow::bail!("Numéro de pièce invalide: {}", part_number),
            };

        let datamatrix_path = &print_settings
            .temp_path
            .join(format!("EdgeLabel DM {}.svg", label_data.no_piece));
        generate_datamatrix_svg(&label_data.no_piece, datamatrix_path)?;

        // Load label data into the templating engine
        let mut context = tera::Context::from_serialize(&label_data)?;
        context.insert("working_dir", &std::env::current_dir()?);
        context.insert("datamatrix", datamatrix_path);

        // Cache the rendered HTML to disk
        let html_path = print_settings
            .temp_path
            .join(format!("EdgeLabel {}.html", label_data.no_piece));
        templates::render_to_file(
            "labels/edge.html",
            print_settings.tera,
            &context,
            &html_path,
        )?;

        // Temporary location to cache rendered PDFs
        let pdf_temp_path = &print_settings
            .temp_path
            .join(format!("EdgeLabel {}.pdf", label_data.no_piece));

        // If the user wants a printed form, save the rendered PDF to a temp
        // directory prior to printing with Ghostscript
        let mut path_list: Vec<PathBuf> = Vec::new();
        if print_quantity > 0 {
            path_list.push(pdf_temp_path.clone());
        }

        // If the user wants a PDF copy for themselves save one to the current directory
        if save_local_pdf {
            path_list.push(
                PathBuf::from(&app_settings.pdf.location)
                    .join(format!("./Etiquette Edge - {}.pdf", &label_data.no_piece)),
            );
        }

        // Convert the rendered HTML to PDF
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
                &pdf_temp_path.to_string_lossy(),
                print_quantity,
                &app_settings.edge.nom_imprimante,
                &PaperSize::CSLabel,
                &PaperOrientation::Portrait,
                print_settings.ghostscript_path,
            )?;
        }
    }

    Ok(())
}

/// Generate a datamatix SVG image representing the part number.
///
/// * `part_number`: Part number.
/// * `filepath`: Path to where the datamatrix image should be stored.
fn generate_datamatrix_svg(
    part_number: &str,
    filepath: &impl AsRef<std::path::Path>,
) -> anyhow::Result<()> {
    use datamatrix::{placement::PathSegment, DataMatrix, SymbolList};
    use std::fmt::Write;
    use std::fs::File;

    let bitmap = DataMatrix::encode(
        part_number.as_bytes(),
        SymbolList::default().enforce_square(),
    )
    .unwrap()
    .bitmap();

    let width = bitmap.width();
    let height = bitmap.height();

    let mut svg =
        format!("<?xml version=\"1.0\"?><svg width=\"33mm\" height=\"33mm\" viewBox=\"1 1 {width} {height}\" class=\"datamatrix\" xmlns=\"http://www.w3.org/2000/svg\"><path fill-rule=\"evenodd\" d=\"M1,1");

    // Now add the path segments. They map nicely to the SVG path syntax.
    // One way to increase or decrease the size is to multiply everything
    // with a constant scale factor.
    for part in bitmap.path() {
        match part {
            PathSegment::Horizontal(n) => write!(svg, "h{}", n),
            PathSegment::Vertical(n) => write!(svg, "v{}", n),
            PathSegment::Move(dx, dy) => write!(svg, "m{},{}", dx, dy),
            PathSegment::Close => write!(svg, "z"),
        }
        .unwrap();
    }
    svg.push_str("\"/></svg>");

    {
        use std::io::Write;
        let mut file = File::create(filepath)?;
        file.write_all(svg.as_bytes())?;
    }

    Ok(())
}
