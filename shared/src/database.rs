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

/// Creates and returns a pool of database connections.
///
/// # Examples
/// ```no_run
/// let db_pool = get_database_pool(&settings.database).await?;
///
/// let row: (i32,) = sqlx::query_as("SELECT 1")
///     .fetch_one(&db_pool)
///     .await?;
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

#[tokio::test]
async fn get_database_pool_test() {
    use crate::settings::Settings;
    let setting_file = String::from(std::env!("CARGO_MANIFEST_DIR")) + "/../coupesuite.toml";
    let settings = Settings::load(&setting_file).unwrap();

    let db_pool = get_database_pool(&settings.database).await.unwrap();

    let row: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&db_pool)
        .await
        .unwrap();

    assert_eq!(row.0, 1)
}
