use clap::Parser;

/// Command line arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Liste délimitée par des virgules de numéros de lot.
    #[clap(short, long, value_parser, default_value = "")]
    pub lots: String,

    /// Liste délimitée par des virgules de numéros de modèles.
    #[clap(short, long, value_parser, default_value = "")]
    pub modèles: String,

    /// Quantité de modèles lors du traitement d'une liste de modèles.
    #[clap(short, long, value_parser, default_value_t = 0)]
    pub quantité: i32,
}
