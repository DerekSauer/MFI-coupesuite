use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::database::{self, verify_lot};
use coupesuite_shared::settings::Settings;
use thirtyfour::common::capabilities::firefox::FirefoxPreferences;
use thirtyfour::{prelude::*, FirefoxCapabilities};

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

    // Enable silent printing to desired printer
    let mut prefs = FirefoxPreferences::new();
    prefs.set("print.always_print_silent", true)?;
    prefs.set("print_printer", "Zebra (Bureau Derek)")?;
    //prefs.set("print_printer", "Back Office Printer")?;

    // Disable page headers
    prefs.set("print.print_headercenter", "")?;
    prefs.set("print.print_headerleft", "")?;
    prefs.set("print.print_headerright", "")?;

    // Disable page footers
    prefs.set("print.print_footercenter", "")?;
    prefs.set("print.print_footerleft", "")?;
    prefs.set("print.print_footerright", "")?;

    // Printer specific settings
    // TODO: Break these out into the settings file
    prefs.set("print.printer_Zebra_(Bureau_Derek).print_orientation", 0)?;
    prefs.set("print.printer_Zebra_(Bureau_Derek).print_margin_bottom", 0)?;
    prefs.set("print.printer_Zebra_(Bureau_Derek).print_margin_left", 0)?;
    prefs.set("print.printer_Zebra_(Bureau_Derek).print_margin_right", 0)?;
    prefs.set("print.printer_Zebra_(Bureau_Derek).print_margin_top", 0)?;

    let mut caps = FirefoxCapabilities::new();
    caps.set_preferences(prefs)?;
    caps.set_headless()?;

    let driver = WebDriver::new("http://localhost:4444", caps).await?;
    driver
        .get("file://C:/Users/DSauer/Source/coupesuite/etiquette/www/label.html")
        .await?;
    println!("Page title: {}", driver.title().await?);

    driver.quit().await?;

    Ok(())
}
