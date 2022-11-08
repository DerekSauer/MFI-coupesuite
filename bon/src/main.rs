use bon::print::print_bon;
use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::{
    database, htmltopdf::HtmlToPdf, print::PrintSettings, settings::Settings, templates,
};
use futures::stream::FuturesUnordered;
use futures::StreamExt;

mod bon_data;
mod cmd_line;
mod print;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let settings = Settings::load(&std::env::current_dir()?.join("coupesuite.toml"))?;
    let temp_dir = tempfile::tempdir()?;

    // Setup label printing dependencies common to all prints
    let print_settings = PrintSettings {
        ghostscript_path: &settings.ghostscript.location,
        tera: &templates::load_templates().await?,
        pdf_renderer: &HtmlToPdf::new(Some(&settings.chromium.location)).await?,
        temp_path: temp_dir.path(),
        db_pool: &database::get_database_pool(&settings.database).await?,
    };

    // Add all print tasks to the task list
    let mut task_list = cmd_line_args
        .lots
        .split(',')
        .map(|lot| {
            print_bon(
                lot,
                cmd_line_args.quantité,
                cmd_line_args.enregistrer,
                &settings,
                &print_settings,
            )
        })
        .collect::<FuturesUnordered<_>>();

    // Execute the task list
    while let Some(task) = task_list.next().await {
        task?;
    }

    Ok(())
}
