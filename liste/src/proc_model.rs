use crate::cut_list::CutListRow;
use coupesuite_shared::database::verify_model;

/// Generate a cut list from a furniture model number.
/// The model number will be validated to make sure it is a unit
/// of furniture instead of a part.
/// Quantities of parts will be the number needed to produce
/// on unit.
pub async fn process_model(
    model_number: &str,
    db_pool: &sqlx::PgPool,
) -> anyhow::Result<Vec<CutListRow>> {
    let query = include_str!("../sql/model_query.sql");

    verify_model(model_number, db_pool).await?;

    Ok(sqlx::query_as::<_, CutListRow>(query)
        .bind(model_number)
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

    let results = process_model("90-101706", &db_pool).await?;

    assert_eq!(results[0].part_code, "2262-0281");

    Ok(())
}
