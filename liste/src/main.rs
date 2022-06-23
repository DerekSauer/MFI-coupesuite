mod cmd_line;

use clap::Parser;
use cmd_line::Args;

fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();

    // We either process lots or models, not both
    if !cmd_line_args.lots.is_empty() && !cmd_line_args.modèles.is_empty() {
        anyhow::bail!(
            "Entrez une liste de numéros de lot ou une liste de numéros de modèle, pas les deux."
        )
    } else if !cmd_line_args.lots.is_empty() {
        // Process lots
        for lot in cmd_line_args.lots.split(',') {
            println!("{lot}");
        }
    } else if !cmd_line_args.modèles.is_empty() {
        // Process models
        for model in cmd_line_args.modèles.split(',') {
            println!("{model}");
        }
    } else {
        anyhow::bail!("Entrez une liste de numéros de lot ou une liste de numéros de modèle.")
    }

    Ok(())
}
