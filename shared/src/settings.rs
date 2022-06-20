use serde::Deserialize;
use std::{fs::File, io::Read, path::Path};

/// Container for program settings.
#[derive(Deserialize)]
pub struct Settings {
    /// Settings file version.
    pub version: String,
}

impl Settings {
    /// Loads settings from a TOML file and parses them into a Settings container.
    ///
    /// # Example
    /// ```
    /// # use coupesuite_shared::settings::Settings;
    /// # let setting_file = String::from(std::env!("CARGO_MANIFEST_DIR")) + "/../coupesuite.toml";
    /// let settings = Settings::load(&setting_file).unwrap();
    /// assert_eq!(settings.version, "0.1.0")
    /// ```
    pub fn load(file_path: &impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut toml_file = String::new();

        File::open(&file_path).and_then(|mut file| file.read_to_string(&mut toml_file))?;

        Ok(toml::from_str(&toml_file)?)
    }
}
