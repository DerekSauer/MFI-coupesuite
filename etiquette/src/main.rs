use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::htmltopdf::HtmlToPdf;
use coupesuite_shared::print::PrintSettings;
use coupesuite_shared::{database, settings::Settings, templates};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use print::print_label;

mod cmd_line;
mod label_data;
mod print;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let settings = Settings::load(&std::env::current_dir()?.join("coupesuite.toml"))?;
    let temp_dir = tempfile::tempdir()?;

    // Setup label printing dependencies common to all prints
    let print_settings = PrintSettings {
        printer_name: &settings.etiquette.nom_imprimante,
        ghostscript_path: &settings.ghostscript.location,
        print_multiple: settings.etiquette.multiple,
        tera: &templates::load_templates().await?,
        pdf_renderer: &HtmlToPdf::new(Some(&settings.chromium.location)).await?,
        temp_path: temp_dir.path(),
        db_pool: &database::get_database_pool(&settings.database).await?,
    };

    // Add print jobs to the task pool
    let mut task_list = cmd_line_args
        .lots
        .split(',')
        .map(|lot| print_label(lot, &cmd_line_args.quantité, &print_settings))
        .collect::<FuturesUnordered<_>>();

    // Execute print jobs
    while let Some(task) = task_list.next().await {
        task?;
    }

    Ok(())
}
