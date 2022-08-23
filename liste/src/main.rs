use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::{database, settings::Settings};
use futures::{stream::FuturesUnordered, StreamExt};
use lot::export_lot;
use model::export_model;

mod cmd_line;
mod cut_list;
mod lot;
mod model;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let app_settings = Settings::load(&std::env::current_dir()?.join("coupesuite.toml"))?;
    let db_pool = database::get_database_pool(&app_settings.database).await?;

    if cmd_line_args.lots.is_empty() && cmd_line_args.modèles.is_empty() {
        anyhow::bail!("Entrez une liste de numéros de lot et/ou une liste de numéros de modèle.");
    }

    // Concurrently process lot numbers
    if !cmd_line_args.lots.is_empty() {
        let mut task_list = cmd_line_args
            .lots
            .split(',')
            .map(|lot| export_lot(lot, &app_settings, &db_pool))
            .collect::<FuturesUnordered<_>>();

        while let Some(task) = task_list.next().await {
            task?;
        }
    }

    // Concurrently process model numbers
    if !cmd_line_args.modèles.is_empty() {
        // If the quantity command line arg is zero, use the default specified in settings
        let quantity = if cmd_line_args.quantité == 0 {
            app_settings.liste.default_bom_qty
        } else {
            cmd_line_args.quantité
        };

        let mut task_list = cmd_line_args
            .modèles
            .split(',')
            .map(|model| export_model(model, quantity, &app_settings, &db_pool))
            .collect::<FuturesUnordered<_>>();

        while let Some(task) = task_list.next().await {
            task?;
        }
    }

    Ok(())
}
