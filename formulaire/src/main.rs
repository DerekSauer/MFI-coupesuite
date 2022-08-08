use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::settings::Settings;

mod cmd_line;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let settings = Settings::load(&std::env::current_dir()?.join("coupesuite.toml"))?;
    let temp_dir = tempfile::tempdir()?;

    Ok(())
}
