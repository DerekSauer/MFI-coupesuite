use std::path::{Path, PathBuf};

/// Data defining a part in a cutlist.
/// Used by our cut pattern optimization software to generate
/// programs for our CNC panels saws.
#[derive(serde::Serialize, sqlx::FromRow, Clone, Debug)]
pub struct CutListRow {
    /// Unique part identifier (part.prt_no in DB).
    pub part_code: String,

    /// Material code of the boards used to fabricate this part.
    pub material_code: String,

    /// Length of the part in mm.
    pub part_length: f32,

    /// Width of the part in mm.
    pub part_width: f32,

    /// Quantity of this part to be cut.
    pub required_quantity: i32,

    /// Unique lot number assigned to this part in this production batch.
    pub no_projet: i32,

    /// Material code of the edge band on the top edge of the part. Blank if none.
    pub code_edge_haut: String,

    /// Description of the edge band on the top edge of the part. Blank if none.
    pub desc_edge_haut: String,

    /// Material code of the edge band on the right edge of the part. Blank if none.
    pub code_edge_droite: String,

    /// Description of the edge band on the right edge of the part. Blank if none.
    pub desc_edge_droite: String,

    /// Material code of the edge band on the bottom edge of the part. Blank if none.
    pub code_edge_bas: String,

    /// Description of the edge band on the bottom edge of the part. Blank if none.
    pub desc_edge_bas: String,

    /// Material code of the edge band on the left edge of the part. Blank if none.
    pub code_edge_gauche: String,

    /// Description of the edge band on the left edge of the part. Blank if none.
    pub desc_edge_gauche: String,

    /// Unique lot number assigned to the furniture batch this part is being produced for.
    pub product_code: i32,

    /// Model number of the furniture model this part belongs to.
    pub product_information: String,

    /// Description of the furniture model this part belongs to.
    pub product_description: String,

    /// Reference letter assigned to this part. Used in the instruction guide.
    pub lettre_piece: String,

    /// File path to an image of this part's drawing.
    pub picture_filename: String,

    /// Description of the part.
    pub part_description: String,

    /// Style of palette this part should be stacked on.
    pub type_palette: String,

    /// Is this part a Kanban production?
    pub kanban: bool,

    /// Is this part painted?
    pub painted: bool,
}

pub fn write_cutlist(
    cut_list: &[CutListRow],
    file_path: &impl AsRef<std::path::Path>,
) -> anyhow::Result<()> {
    let mut csv_writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::NonNumeric)
        .from_path(file_path)?;

    for record in cut_list {
        csv_writer.serialize(record)?;
    }

    Ok(())
}

fn write_cutlist_collection(
    cutlist_collection: &[&CutListRow],
    file_path: &impl AsRef<std::path::Path>,
) -> anyhow::Result<()> {
    let mut csv_writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::NonNumeric)
        .from_path(file_path)?;

    for record in cutlist_collection {
        csv_writer.serialize(record)?;
    }

    Ok(())
}

pub fn write_merged_list(
    cutlist_collection: &Vec<Vec<CutListRow>>,
    export_path: &Path,
) -> anyhow::Result<()> {
    // Get the names of the SKUs in the collection for the export filename
    let mut sku_names: Vec<String> = Vec::with_capacity(cutlist_collection.len());
    for cutlist in cutlist_collection {
        sku_names.push(cutlist.first().unwrap().product_information.clone());
    }
    let sku_names = &sku_names.join(", ");

    // Flatten the collection of cutlists into one cutlist
    let cutlist_collection: Vec<&CutListRow> = cutlist_collection.iter().flatten().collect();

    // Build a filename made up of the SKU names
    let mut file_path: PathBuf = export_path.into();
    file_path.push(&sku_names);
    file_path.set_extension("csv");

    write_cutlist_collection(&cutlist_collection, &file_path)?;

    println!(
        "SKU(s): {}\nFicher: {}\n",
        &sku_names,
        &file_path.to_str().unwrap()
    );

    Ok(())
}
