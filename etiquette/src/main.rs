use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::database::{self, verify_lot};
use coupesuite_shared::settings::Settings;

mod cmd_line;
mod label_data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let settings = Settings::load(&"./coupesuite.toml")?;
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

        let label_data = label_data::LabelData::from_lot(lot, &db_pool).await?;

        println!(
            "SKU: {}\nLot: {}\nDocument 1: {}\nDocument 2: {}\nDocument 3: {}\nLettre: {}\nQuantité: {}\n",
            label_data.model_number,
            label_data.project_number,
            label_data.document_1,
            label_data.document_2,
            label_data.document_3,
            label_data.letter,
            label_data.print_quantity
        );
    }

    Ok(())
}
