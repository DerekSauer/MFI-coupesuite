#[derive(serde::Serialize, sqlx::FromRow, Debug)]
pub struct LotData {
    /// A production run's lot number.
    pub lot_numbers: Option<String>,
}

impl LotData {
    /// Retrieve lot numbers generate today from the database.
    pub async fn today(db_pool: &sqlx::PgPool) -> anyhow::Result<Self> {
        let query = include_str!("../sql/lot.sql");

        Ok(sqlx::query_as::<_, LotData>(query)
            .fetch_one(db_pool)
            .await?)
    }
}
