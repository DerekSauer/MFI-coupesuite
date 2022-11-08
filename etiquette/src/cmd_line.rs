use clap::Parser;

/// Command line arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Liste délimitée par des virgules de numéros de lot.
    #[clap(value_parser)]
    pub lots: String,

    /// Remplacer le nombre d'étiquettes imprimées.
    #[clap(short, long, value_parser)]
    pub quantité: Option<u32>,

    /// Enregistrez une copie des documents au format PDF.
    #[clap(short, long, action)]
    pub enregistrer: bool,
}
