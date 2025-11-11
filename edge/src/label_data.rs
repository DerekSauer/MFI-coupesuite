#[derive(serde::Serialize, sqlx::FromRow, Debug)]
pub struct LabelData {
    /// Part number.
    pub no_piece: String,

    /// Part description.
    pub desc_piece: String,

    /// Length of the part.
    longueur: f32,

    /// Width of the part.
    largeur: f32,

    /// Thickness of the part.
    epaisseur: f32,

    /// Material code for edgeband on top edge.
    edge_haut: String,

    /// Material code for edgeband on right edge.
    edge_droite: String,

    /// Material code for edgeband on bottom edge.
    edge_bas: String,

    /// Material code for edgeband on left edge.
    edge_gauche: String,
}

impl LabelData {
    /// Retrieve label data from the database.
    ///
    /// * `part_number`: Part number.
    /// * `db_pool`: Database pool.
    pub async fn from_part(part_number: &str, db_pool: &sqlx::PgPool) -> anyhow::Result<Self> {
        let query = include_str!("../sql/edge.sql");

        Ok(sqlx::query_as::<_, LabelData>(query)
            .bind(part_number)
            .fetch_one(db_pool)
            .await?)
    }
}
