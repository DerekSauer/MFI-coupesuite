/// Data needed to print customer support label.
#[derive(serde::Serialize, sqlx::FromRow, Debug)]
pub struct LabelData {
    /// SKU.
    pub model_number: String,

    /// The SKU's lot number.
    pub project_number: i32,

    /// SKU's assembly guide #1.
    pub document_1: String,

    /// SKU's assembly guide #2.
    pub document_2: String,

    /// SKU's assembly guide #3.
    pub document_3: String,

    /// Part letter where the customer service label should be affixed.
    pub letter: String,

    /// Number of labels to print.
    pub print_quantity: i32,
}

impl LabelData {
    /// Retrieve label data from the database.
    pub async fn from_lot(lot_number: i32, db_pool: &sqlx::PgPool) -> anyhow::Result<Self> {
        let query = include_str!("../sql/label.sql");

        Ok(sqlx::query_as::<_, LabelData>(query)
            .bind(lot_number)
            .fetch_one(db_pool)
            .await?)
    }
}
