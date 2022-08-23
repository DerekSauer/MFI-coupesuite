use bon::print::print_bon;
use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::{
    database, htmltopdf::HtmlToPdf, print::PrintSettings, settings::Settings, templates,
};
use dessins::print::print_dessins;
use etiquette::print::print_label;
use formulaire::print::print_form;
use liste::lot::export_lot;

mod cmd_line;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let app_settings = Settings::load(&std::env::current_dir()?.join("coupesuite.toml"))?;
    let temp_dir = tempfile::tempdir()?;

    // Setup label printing dependencies common to all prints
    let print_settings = PrintSettings {
        ghostscript_path: &app_settings.ghostscript.location,
        tera: &templates::load_templates().await?,
        pdf_renderer: &HtmlToPdf::new(Some(&app_settings.chromium.location)).await?,
        temp_path: temp_dir.path(),
        db_pool: &database::get_database_pool(&app_settings.database).await?,
    };

    for lot_number in cmd_line_args.lots.split(',') {
        print_lot(lot_number, &app_settings, &print_settings).await?;
    }

    Ok(())
}

/// Print all the productiondocumentation needed for a lot of furniture.
///
/// ## Remarks
/// It does not matter in which order the documents for each lot exit the
/// printer(s), but I do want the documentation for a particular lot to
/// be kept together while printing. Hence, we'll process each lot serially
/// but print each lot's documentation concurrently.
async fn print_lot(
    lot_number: &str,
    app_settings: &Settings,
    print_settings: &PrintSettings<'_>,
) -> anyhow::Result<()> {
    tokio::try_join!(
        export_lot(lot_number, app_settings, print_settings.db_pool),
        print_label(lot_number, None, app_settings, print_settings),
        print_form(lot_number, None, app_settings, print_settings),
        print_bon(lot_number, None, app_settings, print_settings),
        print_dessins(lot_number, None, app_settings, print_settings)
    )?;

    Ok(())
}
