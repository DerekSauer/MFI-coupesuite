use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::{
    database::{self, verify_lot, verify_model},
    settings::Settings,
};

mod cmd_line;
mod cut_list;
mod proc_lot;

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
            println!("{}: {:?}", &lot, verify_lot(lot.parse()?, &db_pool).await?);
        }
    } else if !cmd_line_args.modèles.is_empty() {
        // Process models
        for model in cmd_line_args.modèles.split(',') {
            println!("{}: {:?}", &model, verify_model(model, &db_pool).await?);
        }
    } else {
        anyhow::bail!("Entrez une liste de numéros de lot ou une liste de numéros de modèle.")
    }

    Ok(())
}
