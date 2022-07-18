use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::database::{self, verify_lot};
use coupesuite_shared::settings::Settings;

mod cmd_line;
mod label_data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let working_dir = std::env::current_dir()?;
    let settings = Settings::load(&working_dir.join("coupesuite.toml"))?;
    let db_pool = database::get_database_pool(&settings.database).await?;

    for lot in cmd_line_args.lots.split(',') {
        // SIGM's lot numbers are numeric, make sure of that
        let lot: i32 = match lot.parse() {
            Ok(good_lot) => good_lot,
            Err(_) => anyhow::bail!(
                "Numéro de lot invalide. Les numéros de lot doivent être des chiffres."
            ),
        };

        // verify_lot() will bail if the lot number doesn't exist or the DB fails
        verify_lot(lot, &db_pool).await?;

        // Grab the label's data from the database
        let label_data = label_data::LabelData::from_lot(lot, &db_pool).await?;

        // TODO: Templatize the HTML file and insert label data into the placeholders
    }

    Ok(())
}
