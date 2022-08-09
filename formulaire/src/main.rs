use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::{
    database, htmltopdf::HtmlToPdf, print::PrintSettings, settings::Settings, templates,
};
use formulaire::print::print_form;

mod cmd_line;
mod form_data;
mod print;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let settings = Settings::load(&std::env::current_dir()?.join("coupesuite.toml"))?;
    let temp_dir = tempfile::tempdir()?;

    // Setup label printing dependencies common to all prints
    let print_settings = PrintSettings {
        printer_name: &settings.formulaire.nom_imprimante,
        ghostscript_path: &settings.ghostscript.location,
        print_multiple: 0, // Unused
        tera: &templates::load_templates().await?,
        pdf_renderer: &HtmlToPdf::new(Some(&settings.chromium.location)).await?,
        temp_path: temp_dir.path(),
        db_pool: &database::get_database_pool(&settings.database).await?,
    };

    print_form(
        &cmd_line_args.lots,
        cmd_line_args.quantité,
        &settings,
        &print_settings,
    )
    .await?;

    Ok(())
}
