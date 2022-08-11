use coupesuite_shared::database::{try_parse_lot, verify_lot, SkuInfo};

/// Row data needed to print production tracking forms.
#[derive(serde::Serialize, sqlx::FromRow, Debug)]
pub struct FormData {
    /// Unique identifier for each part in the work order.
    pub part_number: String,

    /// Human readable description of the part.
    pub description: String,

    /// Number of parts to produce.
    pub quantity: i32,

    /// Length of the part (mm).
    pub length: String,

    /// Width of the part (mm).
    pub width: String,

    /// Unusued.
    pub machining_time: String,

    /// Edge banding codes.
    pub edging: String,

    /// CNC machine recommendation.
    pub robot: String,

    /// Manual edge banding or painting flag.
    pub manual_edge: String,
}

impl FormData {
    /// Retrieve label data from the database.
    #[allow(dead_code)] // TODO: Remove when main app is complete.
    pub async fn from_lot(
        lot_number: &str,
        db_pool: &sqlx::PgPool,
    ) -> anyhow::Result<(Vec<Self>, SkuInfo)> {
        let query = include_str!("../sql/form.sql");

        // SIGM's lot numbers are numeric
        let lot_number = try_parse_lot(lot_number)?;

        // verify_lot() will bail if the lot number doesn't exist or the DB fails
        // Remove the leading '90-' from the SKU for a cleaner model number
        let mut sku_info = verify_lot(lot_number, db_pool).await?;
        sku_info.sku = sku_info.sku.replace("90-", "");

        Ok((
            sqlx::query_as::<_, FormData>(query)
                .bind(lot_number)
                .fetch_all(db_pool)
                .await?,
            sku_info,
        ))
    }
}
