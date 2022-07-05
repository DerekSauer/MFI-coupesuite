use clap::Parser;
use cmd_line::Args;
use coupesuite_shared::{database, settings::Settings};
use proc_lot::process_lot;
use proc_model::process_model;

mod cmd_line;
mod cut_list;
mod proc_lot;
mod proc_model;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();
    let settings = Settings::load(&"./coupesuite.toml")?;
    let db_pool = database::get_database_pool(&settings.database).await?;
    let export_path = std::path::PathBuf::from(&settings.liste.v12_import_dir);

    // We either process lots or models, not both
    if !cmd_line_args.lots.is_empty() && !cmd_line_args.modèles.is_empty() {
        anyhow::bail!(
            "Entrez une liste de numéros de lot ou une liste de numéros de modèle, pas les deux."
        )
    } else if !cmd_line_args.lots.is_empty() {
        // Process lots
        for lot in cmd_line_args.lots.split(',') {
            let (mut cutlist, sku_info) = process_lot(lot.parse()?, &db_pool).await?;

            // If the lot number is not a `kanban` production
            // filter out the `kanban` parts
            if !sku_info.kanban {
                cutlist.retain(|x| !x.kanban);
            }

            let mut file_path = export_path.clone();
            file_path.push(format!("{} ({})", &sku_info.sku, lot));
            file_path.set_extension("csv");

            println!(
                "SKU: {}\nDescription: {}\nQuantité: {}\nFicher: {}\n",
                &sku_info.sku,
                &sku_info.description,
                &sku_info.quantity,
                &file_path.to_str().unwrap()
            );

            cut_list::write_to_file(&cutlist, &file_path)?;
        }
    } else if !cmd_line_args.modèles.is_empty() {
        // If the quantity command line arg is zero,
        // use the default specified in settings
        let quantity = if cmd_line_args.quantité == 0 {
            settings.liste.default_bom_qty
        } else {
            cmd_line_args.quantité
        };

        // Process models
        for model in cmd_line_args.modèles.split(',') {
            let (mut cutlist, sku_info) = process_model(model, &db_pool).await?;

            // Update quantities in the cutlist
            for row in cutlist.iter_mut() {
                row.required_quantity *= quantity;
            }

            let mut file_path = export_path.clone();
            file_path.push(&sku_info.sku);
            file_path.set_extension("csv");

            println!(
                "SKU: {}\nDescription: {}\nQuantité: {}\nFicher: {}\n",
                &sku_info.sku,
                &sku_info.description,
                &quantity,
                &file_path.to_str().unwrap()
            );

            cut_list::write_to_file(&cutlist, &file_path)?;
        }
    } else {
        anyhow::bail!("Entrez une liste de numéros de lot ou une liste de numéros de modèle.")
    }

    Ok(())
}
