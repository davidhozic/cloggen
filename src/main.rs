//! # CLOGGEN
//! Generator študentskih mnenj (za habilitacijo).
//! 
//! ## Namestitev pisave (font)
//! Generiran dokument uporablja pisavo *Roboto*. V primeru, da pisava na sistemu ni nameščena, se dokument ne bo generiral.
//! Za namestitev uporabi datoteke v mapi ``data/fonts/Roboto``. Namesti vse datoteke.
//! 
//! ## Uporaba
//! 
//! Za generiranje dokumenta uporabi ukaz:
//! 
//!     cloggen create <CSV DATOTEKA STUDIS ANKET> <JSON NABOR ODZIVOV> <TEX DOKUMENT>   
//! 
//! - ``<CSV DATOTEKA STUDIS ANKET>`` predstavlja izvoženo CSV datoteko z ocenami kandidata za posamezno vprašanje STUDIS anket
//! - ``<JSON NABOR ODZIVOV>`` predstavlja JSON datoteko, ki definira odgovore za posamezno mejo ocene v formatu:
//!     ```json
//!         {
//!         "Vprašanje": {
//!             "Gledano v celoti, je delo izvajalca/ke kakovostno.": {
//!                 "1": ["Odziv 1", "Odziv 2", ...],
//!                 "1.5": ["Odziv 1", "Odziv 2", ...],
//!                 ...
//!                 "4": ["Odziv 1", "Odziv 2", ...],
//!                 "4.5": ["Kandidat ima super ocene (povprečje {MEAN} $\\pm$ {STD}).", "Odziv 2", ...],
//!             }
//!         }
//!     }
//!     ```
//! 
//!     Odzivi so razporejeni po večih številkah. Številke so minimalna meja povprečne ocene pri posameznem vprašanju, ki
//!     jo mora kandidat imeti, zato da dobi enega izmed pripadajočih odzivov.
//!     
//!     Odziv bo izbran iz možnih odzivov, ki pripadajo prvi manjši oceni od povprečne ocene kandidata. Na primer, če ima
//!     kandidat pri vprašanju *Gledano v celoti, je delo izvajalca/ke kakovostno.* povprečno oceno 4.3, bo ob uporabi
//!     zgornjega JSON primera odziv izbran iz odzivov, ki pripadajo oceni 4.0 (``"4": ["Odziv 1", "Odziv 2", ...]``)
//! 
//!     V odziv se lahko dinamično vključi tudi **povprečje** in **standardni odklon**, kot prikazuje zgornjni JSON primer:
//!     ``"4.5": ["Kandidat ima super ocene (povprečje {MEAN} $\\pm$ {STD}).", ...]``. Tu bo ``{MEAN}`` z povprečno oceno za 
//!     pripadajoče vprašanje, ``{STD}`` pa s standardnim odklonom za pripadajoče vprašanje.
//! 
//! - ``<TEX DOKUMENT>`` predstavlja glavni LaTeX dokument (datoteko),
//!     ki bo uporabljen za generacijo izhodnega mnenja v PDF obliki.
//!     Dokument mora vsebovati ``{AUTO_GEN}`` tekst, ki predstavlja lokacijo
//!     vstavitve odzivov/odgovorov, generiranih iz zgornje JSON datoteke odzivov.
//! 
//! ### Popoln primer zagona
//! ``cloggen create ocena.csv mnenje.json data/mnenje.tex``


use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod compiler;
mod preproc;
mod create;
mod merge;


#[derive(Parser)]
#[command(version, author="David")]
struct Args {
    #[command(subcommand)]
    command: Commands
}


#[derive(Subcommand)]
enum Commands {
    /// Generate a habilitation report
    Create {
        /// CSV file of the STUDIS grades.
        studis_csv_filepath: PathBuf,
        /// JSON file of the possible responses per category per grade.
        response_json_filepath: PathBuf,
        /// Path to the output LaTeX file.
        tex_template_filepath: PathBuf
    },

    /// *NOTE* **Use the ``create`` command instead to to create a habilitation report**.
    /// Access to the underlaying LaTeX compiler. Use this when you want to directly
    /// compile a file. If you wish to create a habilitation report, use the ``create`` command.
    Compile {
        tex_file: PathBuf
    },

    /// Merges grades of multiple CSV grades
    Merge {
        /// The CSV files to merge. At least two values.
        #[clap(value_delimiter = ' ', num_args = 2.., required = true)]
        csv_files: Vec<PathBuf>,

        /// Path of the output (merged) file.
        #[clap(short, default_value = "./merged.csv")]
        output: PathBuf
    }
}


fn main() {
    let cli = Args::parse();

    match &cli.command {
        Commands::Create { studis_csv_filepath, response_json_filepath, tex_template_filepath } => {
            create::command_create(studis_csv_filepath, response_json_filepath, tex_template_filepath);
        }

        Commands::Compile { tex_file } => {
            compiler::cmd_compile(tex_file);
        }

        Commands::Merge { csv_files , output} => {
            merge::command_merge(csv_files, output);
        }
    }
}
