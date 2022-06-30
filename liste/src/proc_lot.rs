use crate::cut_list::CutListRow;
use coupesuite_shared::database::verify_lot;

/// Generate a cut list from a furniture lot number.
/// The lot number will be validated to make sure it is a unit
/// of furniture instead of a part.
pub async fn process_lot(
    lot_number: i32,
    db_pool: &sqlx::PgPool,
) -> anyhow::Result<Vec<CutListRow>> {
    let query = include_str!("../sql/lot_query.sql");

    verify_lot(lot_number, db_pool).await?;

    Ok(sqlx::query_as::<_, CutListRow>(query)
        .bind(lot_number)
        .fetch_all(db_pool)
        .await?)
}

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    use coupesuite_shared::database::get_database_pool;
    use coupesuite_shared::settings::Settings;

    let settings_file = String::from(std::env!("CARGO_MANIFEST_DIR")) + "/../coupesuite.toml";
    let settings = Settings::load(&settings_file)?;

    let db_pool = get_database_pool(&settings.database).await?;

    let results = process_lot(730887, &db_pool).await?;

    assert_eq!(results[0].part_code, "2262-0281");

    Ok(())
}
