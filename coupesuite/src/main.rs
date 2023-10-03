use bon::print::print_bon;
use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::{
    database::{self, verify_lot},
    htmltopdf::HtmlToPdf,
    print::PrintSettings,
    settings::Settings,
    templates,
};
use dessins::print::print_dessins;
use etiquette::print::print_label;
use formulaire::print::print_form;
use liste::lot::export_lot;
use lot_data::LotData;

mod cmd_line;
mod lot_data;

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

    // Retrieve a list of lot number from the DB if printing today's lots (-j flag),
    // or print the list given by the user
    let lot_numbers: String = if cmd_line_args.jour {
        match LotData::today(print_settings.db_pool).await?.lot_numbers {
            Some(lots) => lots,
            None => anyhow::bail!("Aucun numéro de lot n'a été généré à la date d'aujourd'hui."),
        }
    } else {
        match cmd_line_args.lots {
            Some(lots) => lots,
            None => anyhow::bail!(
                "Veuillez entrer une liste de numéros de lots séparés par des virgules."
            ),
        }
    };

    // Split the lots numbers by comma and keep a running tally of the number printed.
    let lot_numbers: Vec<&str> = lot_numbers.split(',').collect();
    let total_lots = lot_numbers.len();
    let mut current_lot: usize = 1;

    // Print the documentation for each lot retrieved above
    for lot_number in lot_numbers {
        // Remove any leading or trailing whitespace after the comma seperators
        let lot_number = lot_number.trim();

        // Parse the lot number string into an `i32`
        let integer_lot_number: i32 = match lot_number.parse() {
            Ok(lot) => lot,
            Err(err) => {
                // If there is only one lot number bail immediately, otherwise keep processing lots
                if total_lots == 1 {
                    println!("Erreur avec le numéro de lot: {}", lot_number);
                    anyhow::bail!(err);
                } else {
                    println!("Erreur avec le numéro de lot: {}", lot_number);
                    println!("{}", err);
                    println!("Continue le traitement des lots suivants.\n");
                    continue;
                }
            }
        };

        // Get the SKU info for this lot number
        let sku_info = match verify_lot(integer_lot_number, print_settings.db_pool).await {
            Ok(sku_info) => sku_info,
            Err(err) => {
                // If there is only one lot number bail immediately, otherwise keep processing lots
                if total_lots == 1 {
                    anyhow::bail!(err);
                } else {
                    println!("{}", err);
                    println!("Continue le traitement des lots suivants.\n");
                    continue;
                }
            }
        };

        println!(
            "Impression de documentation pour lot #{lot_number} ({current_lot} de {total_lots})."
        );
        println!("SKU: {}", &sku_info.sku);
        println!("Description: {}", &sku_info.description);
        println!("Quantité: {}\n", &sku_info.quantity);

        print_lot(
            lot_number.trim(),
            cmd_line_args.quantité,
            cmd_line_args.enregistrer,
            &app_settings,
            &print_settings,
        )
        .await?;

        // Pause processing if the user wants a delay between each lot
        if cmd_line_args.pause {
            use std::io::stdin;
            println!("Lot #{lot_number} terminé, appuyez sur ENTER pour continuer.");
            let mut temp = String::new();
            stdin().read_line(&mut temp)?;
        }

        current_lot += 1;
    }

    Ok(())
}

/// Print all the production documentation needed for a lot of furniture.
///
/// ## Remarks
/// It does not matter in which order the documents for each lot exit the
/// printer(s), but I do want the documentation for a particular lot to
/// be kept together while printing. Hence, we'll process each lot serially
/// but print each lot's documentation concurrently.
async fn print_lot(
    lot_number: &str,
    copies: Option<u32>,
    save: bool,
    app_settings: &Settings,
    print_settings: &PrintSettings<'_>,
) -> anyhow::Result<()> {
    tokio::try_join!(
        export_lot(lot_number, app_settings, print_settings.db_pool),
        print_label(lot_number, copies, save, app_settings, print_settings),
        print_form(lot_number, copies, save, app_settings, print_settings),
        print_bon(lot_number, copies, save, app_settings, print_settings),
        print_dessins(lot_number, copies, save, app_settings, print_settings)
    )?;

    Ok(())
}
