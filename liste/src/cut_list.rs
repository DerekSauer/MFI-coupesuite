/// Data defining a part in a cutlist.
/// Used by our cut pattern optimization software to generate
/// programs for our CNC panels saws.
#[derive(serde::Serialize, sqlx::FromRow, Debug)]
pub struct CutListRow {
    /// Unique part identifier (part.prt_no in DB).
    part_code: String,

    /// Material code of the boards used to fabricate this part.
    material_code: String,

    /// Length of the part in mm.
    part_length: f32,

    /// Width of the part in mm.
    part_width: f32,

    /// Quantity of this part to be cut.
    required_quantity: u32,

    /// Unique lot number assigned to this part in this production batch.
    no_project: u32,

    /// Material code of the edge band on the top edge of the part. Blank if none.
    code_edge_haut: String,

    /// Description of the edge band on the top edge of the part. Blank if none.
    desc_edge_haut: String,

    /// Material code of the edge band on the right edge of the part. Blank if none.
    code_edge_droite: String,

    /// Description of the edge band on the right edge of the part. Blank if none.
    desc_edge_droite: String,

    /// Material code of the edge band on the bottom edge of the part. Blank if none.
    code_edge_bas: String,

    /// Description of the edge band on the bottom edge of the part. Blank if none.
    desc_edge_bas: String,

    /// Material code of the edge band on the left edge of the part. Blank if none.
    code_edge_gauche: String,

    /// Description of the edge band on the left edge of the part. Blank if none.
    desc_edge_gauche: String,

    /// Unique lot number assigned to the furniture batch this part is being produced for.
    product_code: u32,

    /// Model number of the furniture model this part belongs to.
    product_information: String,

    /// Description of the furniture model this part belongs to.
    product_description: String,

    /// Reference letter assigned to this part. Used in the instruction guide.
    lettre_piece: String,

    /// File path to an image of this part's drawing.
    picture_filename: String,

    /// Description of the part.
    part_description: String,

    /// Style of palette this part should be stacked on.
    type_palette: String,
}
