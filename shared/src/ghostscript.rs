use crate::htmltopdf::{PaperOrientation, PaperSize};

/// Send a PDF file to a printer.
///
/// # Parameters
/// - `file_path`: Location on disk of the PDF file to print.
/// - `copies`: Number of copies of the document to print.
/// - `printer_name`: Windows name of the printer to use.
/// - `paper_size`: Tuple of the width and height of the paper in inches.
/// - `ghostscript_path`: Location on disk of the Ghostscript executable.
///
/// # Remarks
///
/// Printer names can be listed with `wmic printer get name` in the Terminal.
pub fn print_to_printer(
    file_path: &str,
    copies: u32,
    printer_name: &str,
    paper_size: &PaperSize,
    paper_orientation: &PaperOrientation,
    ghostscript_path: &str,
) -> anyhow::Result<()> {
    let output_printer = format!("-sOutputFile=%printer%{}", printer_name);
    let print_quantity: String = format!("-dNumCopies={}", copies);

    // Function accepts paper size in inches but Ghostscript uses Points
    // A point is 1/72 of an inch
    // Swap values if needed for portrait or landscape orientations
    let (width, height) = match paper_orientation {
        PaperOrientation::Portrait => (paper_size.value().0, paper_size.value().1),
        PaperOrientation::Landscape => (paper_size.value().1, paper_size.value().0),
    };
    let width = format!("-dDEVICEWIDTHPOINTS={}", width * 72.0);
    let height = format!("-dDEVICEHEIGHTPOINTS={}", height * 72.0);

    // Build arguments list to pass to Ghostscript
    let ghostscript_args = vec![
        "-dBATCH",
        "-dNOPAUSE",
        "-dNoCancel",
        "-dQUIET",
        "-dNEWPDF",
        "-sDEVICE=mswinpr2",
        &print_quantity,
        &width,
        &height,
        &output_printer,
        file_path,
    ];

    // Spawn Ghostscript to print the PDF
    let status = std::process::Command::new(&ghostscript_path)
        .args(&ghostscript_args)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Impossible d'imprimer le fichier PDF: {}\n", file_path);
    }
}
