use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::{database, settings::Settings};
use lot::export_lot;
use model::export_model;

mod cmd_line;
mod cut_list;
mod lot;
mod model;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let working_dir = std::env::current_dir()?;
    let settings = Settings::load(&working_dir.join("coupesuite.toml"))?;
    let db_pool = database::get_database_pool(&settings.database).await?;
    let export_path = std::path::PathBuf::from(&settings.liste.v12_import_dir);

    if cmd_line_args.lots.is_empty() && cmd_line_args.modèles.is_empty() {
        anyhow::bail!("Entrez une liste de numéros de lot et/ou une liste de numéros de modèle.");
    }

    // Process and export furniture production lots
    // When processing a batch, print an error message for invalid
    // lots and finish processing the remainder
    if !cmd_line_args.lots.is_empty() {
        for lot in cmd_line_args.lots.split(',') {
            match export_lot(&lot, &export_path, cmd_line_args.verbeux, &db_pool).await {
                Ok(_) => {}
                Err(err) => {
                    println!("{}", err);
                    continue;
                }
            };
        }
    }

    // Process and export furniture models
    // When processing a batch, print an error message for invalid
    // models and finish processing the remainder
    if !cmd_line_args.modèles.is_empty() {
        // If the quantity command line arg is zero, use the default specified in settings
        let quantity = if cmd_line_args.quantité == 0 {
            settings.liste.default_bom_qty
        } else {
            cmd_line_args.quantité
        };

        for model in cmd_line_args.modèles.split(',') {
            match export_model(
                &model,
                quantity,
                &export_path,
                cmd_line_args.verbeux,
                &db_pool,
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    println!("Erreur: {}", err);
                    continue;
                }
            }
        }
    }

    Ok(())
}
