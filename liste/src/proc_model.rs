use crate::cut_list::{self, CutListRow};
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
/// Returns a tuple containing the list of parts to cut an SKU info.
pub async fn process_model(
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

pub async fn export_models(
    model_list: &str,
    merged: bool,
    quantity: i32,
    export_path: &Path,
    db_pool: &PgPool,
) -> anyhow::Result<()> {
    // Collection of cut data for merging cutlists
    let mut cutlist_collection: Vec<Vec<CutListRow>> = Vec::new();

    // Process models. Skip invalid models and keep processing
    for model_number in model_list.split(',') {
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

        // If merging cutlists add this model to the master list
        // and skip exporting it to disk
        if merged {
            cutlist_collection.push(cutlist);
            continue;
        }

        let mut file_path: PathBuf = export_path.into();
        file_path.push(&cutlist.first().unwrap().product_information);
        file_path.set_extension("csv");

        cut_list::write_to_file(&cutlist, &file_path)?;

        println!(
            "SKU: {}\nDescription: {}\nQuantité: {}\nFicher: {}\n",
            &sku_info.sku,
            &sku_info.description,
            &sku_info.quantity,
            &file_path.to_str().unwrap()
        );
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

    let (results, _) = process_model("90-101706", &db_pool).await?;

    assert_eq!(results[0].part_code, "2262-0281");

    Ok(())
}
