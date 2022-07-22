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
    paper_size: (f32, f32),
    ghostscript_path: &str,
) -> anyhow::Result<()> {
    let output_printer = format!("-sOutputFile=%printer%{}", printer_name);
    let print_quantity: String = format!("-dNumCopies={}", copies);

    // Function accepts paper size in inches but Ghostscript uses Points
    // A point is 1/72 of an inch
    let width = (paper_size.0 * 72.0).floor() as i32;
    let height = (paper_size.1 * 72.0).floor() as i32;
    let width = format!("-dDEVICEWIDTHPOINTS={}", width);
    let height = format!("-dDEVICEHEIGHTPOINTS={}", height);

    // Build arguments list to pass to Ghostscript
    let ghostscript_args = vec![
        "-dBATCH",
        "-dNOPAUSE",
        "-dNoCancel",
        "-dNEWPDF",
        "-sDEVICE=mswinpr2",
        &print_quantity,
        &width,
        &height,
        &output_printer,
        &file_path,
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
