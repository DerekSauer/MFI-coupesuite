use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::{database, settings::Settings};
use proc_lot::export_lots;
use proc_model::export_models;

mod cmd_line;
mod cut_list;
mod proc_lot;
mod proc_model;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let settings = Settings::load(&"./coupesuite.toml")?;
    let db_pool = database::get_database_pool(&settings.database).await?;
    let export_path = std::path::PathBuf::from(&settings.liste.v12_import_dir);

    if cmd_line_args.lots.is_empty() && cmd_line_args.modèles.is_empty() {
        anyhow::bail!("Entrez une liste de numéros de lot et/ou une liste de numéros de modèle.");
    }

    // Process and export furniture production lots
    if !cmd_line_args.lots.is_empty() {
        export_lots(
            &cmd_line_args.lots,
            cmd_line_args.fusionner,
            &export_path,
            &db_pool,
        )
        .await?;
    }

    // Process and export furniture models
    if !cmd_line_args.modèles.is_empty() {
        // If the quantity command line arg is zero, use the default specified in settings
        let quantity = if cmd_line_args.quantité == 0 {
            settings.liste.default_bom_qty
        } else {
            cmd_line_args.quantité
        };

        export_models(
            &cmd_line_args.modèles,
            cmd_line_args.fusionner,
            quantity,
            &export_path,
            &db_pool,
        )
        .await?;
    }

    Ok(())
}
