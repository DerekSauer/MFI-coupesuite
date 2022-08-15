use clap::Parser;

/// Command line arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Liste délimitée par des virgules de numéros de lot.
    #[clap(value_parser)]
    pub lots: String,

    /// Nombre de copies à imprimer.
    #[clap(short, long, value_parser)]
    pub quantité: Option<u32>,
}
