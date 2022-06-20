use serde::Deserialize;

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
