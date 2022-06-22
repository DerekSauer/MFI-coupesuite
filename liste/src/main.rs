mod cmd_line;

use clap::Parser;
use cmd_line::Args;

fn main() -> anyhow::Result<()> {
    let cmd_line_args = Args::parse();

    // We either process lots or models, not both
    if !cmd_line_args.lots.is_empty() && !cmd_line_args.modèles.is_empty() {
        anyhow::bail!("Enter a list of lot numbers, or a list of model numbers, not both.")
    } else if !cmd_line_args.lots.is_empty() {
        // Process lots
        for lot in cmd_line::split_cmd_string(&cmd_line_args.lots) {
            println!("{lot}");
        }
    } else if !cmd_line_args.modèles.is_empty() {
        // Process models
        for model in cmd_line::split_cmd_string(&cmd_line_args.modèles) {
            println!("{model}");
        }
    } else {
        anyhow::bail!("Enter a list of lot numbers, or a list of model numbers.")
    }

    // TODO: Refactor cmd line handling. Clap wants comma seperated lists, no spaces.

    Ok(())
}
