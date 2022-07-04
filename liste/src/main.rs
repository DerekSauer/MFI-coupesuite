use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::{database, settings::Settings};
use proc_lot::process_lot;
use proc_model::process_model;

mod cmd_line;
mod cut_list;
mod proc_lot;
mod proc_model;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let settings = Settings::load(&"./coupesuite.toml")?;
    let db_pool = database::get_database_pool(&settings.database).await?;

    // We either process lots or models, not both
    if !cmd_line_args.lots.is_empty() && !cmd_line_args.modèles.is_empty() {
        anyhow::bail!(
            "Entrez une liste de numéros de lot ou une liste de numéros de modèle, pas les deux."
        )
    } else if !cmd_line_args.lots.is_empty() {
        // Process lots
        for lot in cmd_line_args.lots.split(',') {
            let cutlist = process_lot(lot.parse()?, &db_pool).await?;
            cut_list::write_to_file(&cutlist, &"test.csv")?;
        }
    } else if !cmd_line_args.modèles.is_empty() {
        // Process models
        for model in cmd_line_args.modèles.split(',') {
            let cutlist = process_model(model, &db_pool).await?;
            cut_list::write_to_file(&cutlist, &"test.csv")?;
        }
    } else {
        anyhow::bail!("Entrez une liste de numéros de lot ou une liste de numéros de modèle.")
    }

    Ok(())
}
