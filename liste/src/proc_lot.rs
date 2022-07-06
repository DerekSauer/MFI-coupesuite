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
pub async fn process_lot(
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

pub async fn export_lots(
    lot_list: &str,
    merged: bool,
    export_path: &Path,
    db_pool: &PgPool,
) -> anyhow::Result<()> {
    // Master list of cut data for merging cutlists
    let mut cutlist_collection: Vec<Vec<CutListRow>> = Vec::new();

    // Process lots, skip invalid lots and keep processing
    for lot in lot_list.split(',') {
        // Lot numbers are numeric.
        // Parse will fail if the user passes in a model number
        let lot_number = match lot.parse() {
            Ok(good_lot) => good_lot,
            Err(_) => {
                println!("Error: Numéro de lot incorrect: {}.\nAvez-vous entré un numéro de modèle par erreur?\n", lot);
                continue;
            }
        };

        let (mut cutlist, sku_info) = match process_lot(lot_number, db_pool).await {
            Ok(good_data) => good_data,
            Err(bad_data) => {
                println!(
                    "Error: Problème avec le numéro de lot: {}\n{}\n",
                    lot, bad_data
                );
                continue;
            }
        };

        // If the lot number is not a `kanban` production
        // filter out the `kanban` parts
        if !sku_info.kanban {
            cutlist.retain(|x| !x.kanban);
        }

        // If merging cutlists, add this list to the collection
        // and skip exporting it to disk
        if merged {
            cutlist_collection.push(cutlist);
            continue;
        }

        let mut file_path: PathBuf = export_path.into();
        file_path.push(format!(
            "{} ({})",
            &cutlist.first().unwrap().product_information,
            lot
        ));
        file_path.set_extension("csv");

        println!(
            "SKU: {}\nDescription: {}\nQuantité: {}\nFicher: {}\n",
            &sku_info.sku,
            &sku_info.description,
            &sku_info.quantity,
            &file_path.to_str().unwrap()
        );

        cut_list::write_to_file(&cutlist, &file_path)?;
    }

    if merged {
        // Get the names of the SKUs in the collection for the export filename
        let mut sku_names: Vec<String> = Vec::with_capacity(cutlist_collection.len());
        for cutlist in &cutlist_collection {
            sku_names.push(cutlist.first().unwrap().product_information.clone());
        }
        let sku_names = &sku_names.join(", ");

        // Flatten the cutlists into one list
        let cutlist_collection: Vec<CutListRow> =
            cutlist_collection.into_iter().flatten().collect();

        // Build a filename made up of the SKU names
        let mut file_path: PathBuf = export_path.into();
        file_path.push(&sku_names);
        file_path.set_extension("csv");

        println!(
            "SKU(s): {}\nFicher: {}\n",
            &sku_names,
            &file_path.to_str().unwrap()
        );

        cut_list::write_to_file(&cutlist_collection, &file_path)?;
    }

    Ok(())
}

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    use coupesuite_shared::database::get_database_pool;
    use coupesuite_shared::settings::Settings;

    let settings_file = String::from(std::env!("CARGO_MANIFEST_DIR")) + "/../coupesuite.toml";
    let settings = Settings::load(&settings_file)?;

    let db_pool = get_database_pool(&settings.database).await?;

    let (results, _) = process_lot(730887, &db_pool).await?;

    assert_eq!(results[0].part_code, "2262-0281");

    Ok(())
}
