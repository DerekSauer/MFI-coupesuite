use crate::cut_list::{self, CutListRow};
use anyhow::Context;
use coupesuite_shared::{
    database::{verify_lot, SkuInfo},
    settings::Settings,
};
use sqlx::PgPool;
use std::path::PathBuf;

/// Generate a cut list from a furniture lot number.
/// The lot number will be validated to make sure it is a unit
/// of furniture instead of a part.
///
/// # Returns
///
/// Returns a tuple of the list of parts to cut and information about the sku.
async fn process_lot(
    lot_number: i32,
    db_pool: &sqlx::PgPool,
) -> anyhow::Result<(Vec<CutListRow>, SkuInfo)> {
    let query = include_str!("../sql/lot_query.sql");

    let sku_info = verify_lot(lot_number, db_pool).await?;

    Ok((
        sqlx::query_as::<_, CutListRow>(query)
            .bind(lot_number)
            .fetch_all(db_pool)
            .await?,
        sku_info,
    ))
}

/// Export a cut list for a furniture lot number to disk.
///
/// ## Parameters
///
/// - `lot_list`: String containing a comma seperated list of lot numbers.
/// - `export_path`: Path to the directory where cut lists will be written.
/// - `verbose`: Print additional details about the process.
/// - `db_pool`: The database connection pool.
pub async fn export_lot(
    lot_number: &str,
    app_settings: &Settings,
    db_pool: &PgPool,
) -> anyhow::Result<()> {
    // Lot numbers must be integers
    let lot_number = lot_number.parse().with_context(|| {
        format!(
            "Numéro de lot incorrect: {lot_number}.\nAvez-vous entré un numéro de modèle par erreur?")
    })?;

    let (mut cutlist, sku_info) = process_lot(lot_number, db_pool)
        .await
        .with_context(|| format!("Problème avec le numéro de lot: {lot_number}"))?;

    // If the lot number is not a `kanban` production, filter out the `kanban` parts
    if !sku_info.kanban {
        cutlist.retain(|x| !x.kanban);
    }

    // The model number and lot number in parens is the file name
    let mut file_path: PathBuf = app_settings.liste.v12_import_dir.to_string().into();
    file_path.push(format!("{} ({})", &sku_info.sku, lot_number));
    file_path.set_extension("csv");

    cut_list::write_cutlist(&cutlist, &file_path).await?;

    Ok(())
}

#[tokio::test]
async fn process_lot_test() -> anyhow::Result<()> {
    use coupesuite_shared::database::get_database_pool;
    use coupesuite_shared::settings::Settings;

    let settings_file = String::from(std::env!("CARGO_MANIFEST_DIR")) + "/../coupesuite.toml";
    let settings = Settings::load(&settings_file)?;

    let db_pool = get_database_pool(&settings.database).await?;

    let (results, _) = process_lot(730887, &db_pool).await?;

    assert_eq!(results[0].part_code, "2262-0281");

    Ok(())
}
