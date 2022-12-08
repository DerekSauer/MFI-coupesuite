use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::htmltopdf::HtmlToPdf;
use coupesuite_shared::print::PrintSettings;
use coupesuite_shared::{database, settings::Settings, templates};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use print::print_label;
use std::io::{stdin, stdout, Write};

mod cmd_line;
mod label_data;
mod print;

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

    // Adhoc print tasks or server mode?
    if cmd_line_args.serveur {
        // Some craziness to clear the terminal
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

        // And the user help
        println!("      Coupesuite - Étiquette d'Edge - Mode serveur     ");
        println!("=======================================================");
        println!("Tapez un numéro de pièce et appuyez sur <ENTER> ou     ");
        println!("scannez un code-barres de numéro de pièce.             ");
        println!("Tapez <q> et appuyez sur <ENTER> pour quitter.         ");
        println!("=======================================================");
        println!("");

        // Remain in an endless loop accepting inputs until the user kills the
        // application with `q`
        loop {
            print!("> ");
            stdout().flush()?;

            let mut input = String::new();
            stdin().read_line(&mut input)?;

            // Handle quit option
            if input.to_lowercase().trim() == "q" {
                break;
            } else {
                // Handle other inputs
                match print_label(
                    &input.to_lowercase().trim(),
                    cmd_line_args.quantité,
                    cmd_line_args.enregistrer,
                    &app_settings,
                    &print_settings,
                )
                .await
                {
                    Ok(_) => continue,
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                }
            }
        }
    } else {
        // Add print jobs to the task pool
        if let Some(pieces) = cmd_line_args.pièces {
            let mut task_list = pieces
                .split(',')
                .map(|part| {
                    print_label(
                        part,
                        cmd_line_args.quantité,
                        cmd_line_args.enregistrer,
                        &app_settings,
                        &print_settings,
                    )
                })
                .collect::<FuturesUnordered<_>>();

            // Execute print jobs
            while let Some(task) = task_list.next().await {
                task?;
            }
        }
    }

    Ok(())
}
