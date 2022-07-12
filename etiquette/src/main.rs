use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::database;
use coupesuite_shared::settings::Settings;

mod cmd_line;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let settings = Settings::load(&"./coupesuite.toml")?;
    let db_pool = database::get_database_pool(&settings.database).await?;

    println!("{:?}", settings);

    Ok(())
}
