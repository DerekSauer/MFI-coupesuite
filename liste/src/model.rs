use crate::cut_list::{self, write_cutlist_collection, CutListRow};
use coupesuite_shared::database::{verify_model, SkuInfo};
use sqlx::PgPool;
use std::path::{Path, PathBuf};

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

/// Export cut lists for furniture models to disk.
///
/// The export process will skip any invalid model numbers and continue
/// to process good data until the list of models is exhausted.
///
/// ## Parameters
///
/// - `model_list`: String containing a comma seperated list of model numbers.
/// - `merged`: If true, the cutlists will be merged into a single file.
/// - `quantity`: A model number has no implicit quantity of parts to produce. This argument lets you specific the quantity.
/// - `export_path`: Path to the directory where cut lists will be written.
/// - `db_pool`: The database connection pool.
pub async fn export_models(
    model_list: &str,
    merged: bool,
    quantity: i32,
    export_path: &Path,
    db_pool: &PgPool,
) -> anyhow::Result<()> {
    let model_list: Vec<&str> = model_list.split(',').collect();
    let list_length = model_list.len();
    let mut cutlist_collection: Vec<Vec<CutListRow>> = if list_length > 1 {
        Vec::with_capacity(list_length)
    } else {
        Vec::new()
    };

    // Process models. Skip invalid models and keep processing
    for model_number in model_list {
        let (cutlist, sku_info) = match process_model(model_number, db_pool).await {
            Ok(mut good_model) => {
                // Add real quantity to each row
                for row in good_model.0.iter_mut() {
                    row.required_quantity *= quantity
                }
                good_model
            }
            Err(bad_model) => {
                println!(
                    "Error: Problème avec le numéro de modèle: {}\n{}\n",
                    model_number, bad_model
                );
                continue;
            }
        };

        // If merging cutlists add this model to the master list and skip exporting it to disk
        // If only one model number was passed, ignore the merge argument
        if merged && list_length > 1 {
            cutlist_collection.push(cutlist);
            continue;
        }

        // Use the model number as the file name
        let mut file_path: PathBuf = export_path.into();
        file_path.push(&cutlist.first().unwrap().product_information);
        file_path.set_extension("csv");

        cut_list::write_cutlist(&cutlist, &file_path)?;

        println!(
            "SKU: {}\nDescription: {}\nQuantité: {}\nFicher: {}\n",
            &sku_info.sku,
            &sku_info.description,
            &sku_info.quantity,
            &file_path.to_str().unwrap()
        );
    }

    if merged && list_length > 1 {
        write_cutlist_collection(&cutlist_collection, export_path)?;
    }

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
