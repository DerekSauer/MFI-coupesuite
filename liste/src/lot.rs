use crate::cut_list::{self, CutListRow};
use coupesuite_shared::database::{verify_lot, SkuInfo};
use sqlx::PgPool;
use std::path::{Path, PathBuf};

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

/// Export cut lists for furniture lot numbers to disk.
///
/// The export process will skip any invalid model numbers and continue
/// to process good data until the list of models is exhausted.
///
/// ## Parameters
///
/// - `lot_list`: String containing a comma seperated list of lot numbers.
/// - `export_path`: Path to the directory where cut lists will be written.
/// - `verbose`: Print additional details about the process.
/// - `db_pool`: The database connection pool.
pub async fn export_lots(
    lot_list: &str,
    export_path: &Path,
    verbose: bool,
    db_pool: &PgPool,
) -> anyhow::Result<()> {
    let lot_list: Vec<&str> = lot_list.split(',').collect();

    // Process lots, skip invalid lots and keep processing
    for lot_number in lot_list {
        // Lot numbers must be integers
        let lot_number = match lot_number.parse() {
            Ok(good_lot) => good_lot,
            Err(_) => {
                println!("Error: Numéro de lot incorrect: {}.\nAvez-vous entré un numéro de modèle par erreur?\n", lot_number);
                continue;
            }
        };

        let (mut cutlist, sku_info) = match process_lot(lot_number, db_pool).await {
            Ok(good_lot) => good_lot,
            Err(bad_lot) => {
                println!(
                    "Error: Problème avec le numéro de lot: {}\n{}\n",
                    lot_number, bad_lot
                );
                continue;
            }
        };

        // If the lot number is not a `kanban` production, filter out the `kanban` parts
        if !sku_info.kanban {
            cutlist.retain(|x| !x.kanban);
        }

        // The model number and lot number in parens is the file name
        let mut file_path: PathBuf = export_path.into();
        file_path.push(format!(
            "{} ({})",
            &cutlist.first().unwrap().product_information,
            lot_number
        ));
        file_path.set_extension("csv");

        cut_list::write_cutlist(&cutlist, &file_path).await?;

        if verbose {
            println!(
                "SKU: {}\nDescription: {}\nQuantité: {}\nFicher: {}\n",
                &sku_info.sku,
                &sku_info.description,
                &sku_info.quantity,
                &file_path.to_str().unwrap()
            );
        }
    }

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
