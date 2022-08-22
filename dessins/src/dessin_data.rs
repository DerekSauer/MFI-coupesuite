use coupesuite_shared::database::{try_parse_lot, verify_lot, SkuInfo};

/// Row data needed to print part drawings.
#[derive(serde::Serialize, sqlx::FromRow, Debug)]
pub struct DessinData {
    /// Unique identifying number for each part (effectively a lot number).
    pub project_number: i32,

    /// Part number.
    pub part_number: String,

    /// Description of the part.
    pub part_description: String,

    /// Number of parts to produce.
    pub req_quantity: i32,

    /// Part letter reference for the assembly guide.
    pub lettre: String,

    /// Filename of the CNC program.
    pub cnc_program: String,

    /// List of CNC machines that can produce this part.
    pub cnc_machines: String,

    /// Number of drill holes in the part
    pub num_holes: String,

    /// Length of the part in millimeters.
    pub length: f32,

    /// Width of the part in millimeters.
    pub width: f32,

    /// Thickness of the part in millimeters.
    pub thickness: f32,

    /// Material code of the panel(s) used to make this part.
    pub material_code: String,

    /// Description of the lamination or colors of the panel.
    pub material_color: String,

    /// Number of laminated faces on the panel.
    pub material_faces: i32,

    /// Quantity of parts that can be stacked on one pallet.
    pub parts_per_pallet: i32,

    /// Pallet number out of total number of pallets.
    pub pallet_number: i32,

    /// Total number of pallets needed to stack all the parts.
    pub total_pallets: i32,

    /// Flag indicating if the part is a kanban component or not.
    pub machining_time: String,
}

impl DessinData {
    /// Retrieve label data from the database.
    #[allow(dead_code)] // TODO: Remove when main app is complete.
    pub async fn from_lot(
        lot_number: &str,
        db_pool: &sqlx::PgPool,
    ) -> anyhow::Result<(Vec<Self>, SkuInfo)> {
        let query = include_str!("../sql/dessins.sql");

        // SIGM's lot numbers are numeric
        let lot_number = try_parse_lot(lot_number)?;

        // verify_lot() will bail if the lot number doesn't exist or the DB fails
        // Remove the leading '90-' from the SKU for a cleaner model number
        let mut sku_info = verify_lot(lot_number, db_pool).await?;
        sku_info.sku = sku_info.sku.replace("90-", "");

        Ok((
            sqlx::query_as::<_, DessinData>(query)
                .bind(lot_number)
                .fetch_all(db_pool)
                .await?,
            sku_info,
        ))
    }
}
