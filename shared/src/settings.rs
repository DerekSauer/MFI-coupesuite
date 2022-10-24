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

    /// Etiquette application settings.
    pub etiquette: EtiquetteSettings,

    /// Edge application settings.
    pub edge: EdgeSettings,

    /// Production form application settings.
    pub formulaire: FormulaireSettings,

    /// Transfer manifest application settings.
    pub bon: BonSettings,

    /// Blueprint printing settings.
    pub dessins: DessinSettings,

    /// Database settings.
    pub database: Database,

    /// Chromium settings.
    pub chromium: Chromium,

    /// Ghostscript settings.
    pub ghostscript: GhostScript,
}

/// Settings for the `liste` application.
#[derive(Deserialize, Debug)]
pub struct ListeSettings {
    /// Directory to write cutlists for our cut plan optimizer.
    pub v12_import_dir: String,

    /// Default quantity of parts to use for model number only cutlists.
    pub default_bom_qty: i32,
}

/// Settings for the `etiquette` application.
#[derive(Deserialize, Debug)]
pub struct EtiquetteSettings {
    /// Windows printer name of the label printer device.
    pub nom_imprimante: String,

    /// Number of labels printed will be rounded up to this multiple.
    pub extra: u32,
}

/// Settings for the `edge` application.
#[derive(Deserialize, Debug)]
pub struct EdgeSettings {
    /// Windows printer name of the label printer device.
    pub nom_imprimante: String,
}

/// Settings for the `formulaire` application.
#[derive(Deserialize, Debug)]
pub struct FormulaireSettings {
    /// Windows printer name of the destination printer.
    pub nom_imprimante: String,

    /// Number of copies to print by default.
    pub copies_defaut: u32,

    /// Number of copies to print by default for jobs with MDF components.
    pub copies_mdf: u32,

    /// Chemin d'accès au répertoire contenant les images affichées sur le formulaire.
    pub fichier_images: String,
}

/// Settings for the `bon` application.
#[derive(Deserialize, Debug)]
pub struct BonSettings {
    /// Windows printer name of the destination printer.
    pub nom_imprimante: String,

    /// Number of copies to print by default.
    pub copies_defaut: u32,
}

/// Settings for the `dessin` application.
#[derive(Deserialize, Debug)]
pub struct DessinSettings {
    /// Windows printer name of the destination printer.
    pub nom_imprimante: String,

    /// Number of copies to print by default.
    pub copies_defaut: u32,

    /// Location of the drawings
    pub fichier_dessins: String,
}

/// Settings for the headless chromium browser we'll be using to generate PDFs.
#[derive(Deserialize, Debug)]
pub struct Chromium {
    /// File path to the web browser.
    pub location: String,
}

/// Settings for Ghostscript we'll be using to print PDFs.
#[derive(Deserialize, Debug)]
pub struct GhostScript {
    /// File path to Ghostscript.
    pub location: String,
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
