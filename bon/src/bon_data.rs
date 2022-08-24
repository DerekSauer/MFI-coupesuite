use coupesuite_shared::database::{try_parse_lot, verify_lot, SkuInfo};

/// Row data needed to print a transfer manifest form.
#[derive(serde::Serialize, sqlx::FromRow, Debug)]
pub struct BonData {
    /// Letter for this part used as a reference in the assembly guide.
    pub part_letter: String,

    /// Quantity needed to ship.
    pub qty_to_ship: i32,

    /// Part's lot number, distinct from the model's lot number.
    pub part_lot_number: i32,

    /// The part's uniquely identifying part number.
    pub part_number: String,

    /// Description of the part.
    pub part_description: String,

    /// Other models where this part is used.
    pub common_skus: String,

    /// Kanban flag.
    pub machining_time: String,
}

impl BonData {
    /// Retrieve label data from the database.
    #[allow(dead_code)] // TODO: Remove when main app is complete.
    pub async fn from_lot(
        lot_number: &str,
        db_pool: &sqlx::PgPool,
    ) -> anyhow::Result<(Vec<Self>, SkuInfo)> {
        let query = include_str!("../sql/bon.sql");

        // SIGM's lot numbers are numeric
        let lot_number = try_parse_lot(lot_number)?;

        // verify_lot() will bail if the lot number doesn't exist or the DB fails
        let sku_info = verify_lot(lot_number, db_pool).await?;

        Ok((
            sqlx::query_as::<_, BonData>(query)
                .bind(lot_number)
                .fetch_all(db_pool)
                .await?,
            sku_info,
        ))
    }
}
