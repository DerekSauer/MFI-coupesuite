use crate::cut_list::{self, CutListRow};
use anyhow::Context;
use coupesuite_shared::{
    database::{verify_model, SkuInfo},
    settings::Settings,
};
use sqlx::PgPool;
use std::path::PathBuf;

/// Generate a cut list from a furniture model number.
/// The model number will be validated to make sure it is a unit
/// of furniture instead of a part.
/// Quantities of parts will be the number needed to produce
/// one unit.
///
/// # Returns
///
/// Returns a tuple containing the list of parts to cut and SKU info.
async fn process_model(
    model_number: &str,
    db_pool: &sqlx::PgPool,
) -> anyhow::Result<(Vec<CutListRow>, SkuInfo)> {
    let query = include_str!("../sql/model_query.sql");

    let sku_info = verify_model(model_number, db_pool).await?;

    Ok((
        sqlx::query_as::<_, CutListRow>(query)
            .bind(model_number)
            .fetch_all(db_pool)
            .await?,
        sku_info,
    ))
}

/// Export a cut list for a furniture model to disk.
///
/// ## Parameters
///
/// - `model_list`: String containing a comma seperated list of model numbers.
/// - `quantity`: A model number has no implicit quantity of parts to produce. This argument lets you specific the quantity.
/// - `export_path`: Path to the directory where cut lists will be written.
/// - `verbose`: Print additional details about the process.
/// - `db_pool`: The database connection pool.
pub async fn export_model(
    model_number: &str,
    quantity: i32,
    app_settings: &Settings,
    db_pool: &PgPool,
) -> anyhow::Result<()> {
    let (mut cutlist, sku_info) = process_model(model_number, db_pool)
        .await
        .with_context(|| format!("Problème avec le numéro de modèle: {model_number}"))?;

    // Add real quantity to each row
    for row in cutlist.iter_mut() {
        row.required_quantity = (row.required_quantity * quantity as f32).ceil();
    }

    // Use the model number as the file name
    let mut file_path = PathBuf::from(&app_settings.liste.v12_import_dir);
    file_path.push(&sku_info.sku);
    file_path.set_extension("csv");

    cut_list::write_cutlist(&cutlist, &file_path).await?;

    Ok(())
}

#[tokio::test]
async fn process_model_test() -> anyhow::Result<()> {
    use coupesuite_shared::database::get_database_pool;
    use coupesuite_shared::settings::Settings;

    let settings_file = String::from(std::env!("CARGO_MANIFEST_DIR")) + "/../coupesuite.toml";
    let settings = Settings::load(&settings_file)?;

    let db_pool = get_database_pool(&settings.database).await?;

    let (results, _) = process_model("90-101706", &db_pool).await?;

    assert_eq!(results[0].part_code, "2262-0281");

    Ok(())
}
