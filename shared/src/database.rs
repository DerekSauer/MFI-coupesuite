use serde::Deserialize;
use sqlx::PgPool;

/// Settings container for database connection parameters.
#[derive(Deserialize, Debug)]
pub struct Database {
    /// Database hostname or IP address.
    pub hostname: String,

    /// Database port number.
    pub port: String,

    /// Name of the database to access in the database server.
    pub name: String,

    /// Username used to connect to the database.
    pub username: String,
}

/// Stores basic information about the Sku being verified.
#[derive(sqlx::FromRow, Debug)]
pub struct SkuInfo {
    /// SKU part number.
    pub sku: String,

    /// Description of the SKU.
    pub description: String,

    /// Quantity to produce.
    pub quantity: i32,

    /// Is this SKU a kanban production (effects reports printed)?
    pub kanban: bool,

    /// Does this SKU contain painted parts (effects reports printed)?
    pub painted_parts: bool,
}

/// Retrieve a database connection pool.
///
/// # Example
/// ```
/// # fn main() -> anyhow::Result<()> {
/// #     tokio_test::block_on(async {
/// #         use coupesuite_shared::database::get_database_pool;
/// #         use coupesuite_shared::settings::Settings;
/// #
/// #         let setting_file = String::from(std::env!("CARGO_MANIFEST_DIR")) + "/../coupesuite.toml";
/// #         let settings = Settings::load(&setting_file)?;
/// #
///           let db_pool = get_database_pool(&settings.database).await?;
///
///           let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&db_pool).await?;
///
///           assert_eq!(row.0, 1);
/// #
/// #         Ok(())
/// #     })
/// #  }
/// ```
pub async fn get_database_pool(connection_settings: &Database) -> anyhow::Result<sqlx::PgPool> {
    let connection_string = format!(
        "postgres://{}@{}:{}/{}",
        &connection_settings.username,
        &connection_settings.hostname,
        &connection_settings.port,
        &connection_settings.name
    );

    Ok(PgPool::connect(&connection_string).await?)
}

/// Verify if a lot number is a valid furniture production lot.
///
/// Our database assigns a unique project number to each run of furniture
/// produced in the factory. We use that project number as a lot number
/// for tracking purposes. This function assures a lot number is a unit
/// of furniture and not an individual part or a typo.
///
/// # Returns
/// Returns a `Result` since the underlying database operation can fail. A return
/// of `Err` indicates a failure at the database. `Ok` contains a boolean where
/// `true` indicates a valid lot number and `false` indicates an invalid lot number.
///
/// # Example
///
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// # use coupesuite_shared::database::verify_lot;
/// # use coupesuite_shared::database::get_database_pool;
/// # use coupesuite_shared::settings::Settings;
/// #
/// # let setting_file = String::from(std::env!("CARGO_MANIFEST_DIR")) + "/../coupesuite.toml";
/// # let settings = Settings::load(&setting_file)?;
/// #
/// let db_pool = get_database_pool(&settings.database).await?;
/// let valid_lot = verify_lot(730887, &db_pool).await?;
///
/// assert_eq!(valid_lot.sku, "90-101706");
/// #
/// # Ok(())
/// # })
/// # }
/// ```
pub async fn verify_lot(lot_number: i32, db_pool: &sqlx::PgPool) -> anyhow::Result<SkuInfo> {
    let query = include_str!("../sql/verify_lot.sql");

    match sqlx::query_as::<_, SkuInfo>(query)
        .bind(lot_number)
        .fetch_optional(db_pool)
        .await?
    {
        Some(result) => Ok(result),
        None => anyhow::bail!("Numéro de lot invalide: {}", lot_number),
    }
}

/// Verify if a model number is a valid furniture sku.
///
/// # Returns
/// Returns a `Result` since the underlying database operation can fail. A return
/// of `Err` indicates a failure at the database. `Ok` contains a boolean where
/// `true` indicates a valid model number and `false` indicates an invalid model number.
///
/// # Example
///
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// # use coupesuite_shared::database::verify_model;
/// # use coupesuite_shared::database::get_database_pool;
/// # use coupesuite_shared::settings::Settings;
/// #
/// # let setting_file = String::from(std::env!("CARGO_MANIFEST_DIR")) + "/../coupesuite.toml";
/// # let settings = Settings::load(&setting_file)?;
/// #
/// let db_pool = get_database_pool(&settings.database).await?;
/// let valid_model = verify_model("90-5092", &db_pool).await?;
///
/// assert_eq!(valid_model.sku, "90-5092");
/// #
/// # Ok(())
/// # })
/// # }
/// ```
pub async fn verify_model(model_number: &str, db_pool: &sqlx::PgPool) -> anyhow::Result<SkuInfo> {
    let query = include_str!("../sql/verify_model.sql");

    match sqlx::query_as::<_, SkuInfo>(query)
        .bind(model_number)
        .fetch_optional(db_pool)
        .await?
    {
        Some(result) => Ok(result),
        None => anyhow::bail!("Numéro de modèle invalide: {}", model_number),
    }
}
