use serde::Deserialize;
use std::{fs::File, io::Read, path::Path};

use crate::database::Database;

/// Container for program settings.
#[derive(Deserialize, Debug)]
pub struct Settings {
    /// Settings file version.
    pub version: String,

    /// Liste application settings.
    pub liste: ListeSettings,

    /// Database settings.
    pub database: Database,
}

/// Settings for the `liste` application.
#[derive(Deserialize, Debug)]
pub struct ListeSettings {
    /// Directory to write cutlists for our cut plan optimizer.
    pub v12_import_dir: String,

    /// Default quantity of parts to use for model number only cutlists.
    pub default_bom_qty: i32,
}

impl Settings {
    /// Loads settings from a TOML file and parses them into a Settings container.
    ///
    /// # Example
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// #     use coupesuite_shared::settings::Settings;
    /// #     let setting_file = String::from(std::env!("CARGO_MANIFEST_DIR")) + "/../coupesuite.toml";
    /// #
    ///       let settings = Settings::load(&setting_file)?;
    ///
    ///       assert_eq!(settings.version, std::env!("CARGO_PKG_VERSION"));
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn load(file_path: &impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut toml_file = String::new();

        File::open(&file_path).and_then(|mut file| file.read_to_string(&mut toml_file))?;

        Ok(toml::from_str(&toml_file)?)
    }
}
