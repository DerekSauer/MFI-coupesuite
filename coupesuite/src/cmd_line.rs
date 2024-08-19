use clap::Parser;

/// Command line arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Liste délimitée par des virgules de numéros de lot.
    #[clap(value_parser)]
    pub lots: Option<String>,

    /// Pause pas entre chaque lot.
    #[clap(short, long, action)]
    pub sans_pause: bool,

    /// Traitez tous les projets générés à la date d'aujourd'hui?
    #[clap(short, long, action)]
    pub jour: bool,

    /// Nombre de copies à imprimer. Cela s'appliquera à tous les documents.
    #[clap(short, long, value_parser)]
    pub quantité: Option<u32>,

    /// Enregistrez une copie des documents au format PDF?
    #[clap(short, long, action)]
    pub enregistrer: bool,
}
