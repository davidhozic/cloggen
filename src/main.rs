//! # CLOGGEN
//! Generator študentskih mnenj (za habilitacijo).
//! 
//! ## Namestitev pisave (font)
//! Generiran dokument uporablja pisavo *Roboto*. V primeru, da pisava na sistemu ni nameščena, se dokument ne bo generiral.
//! Za namestitev uporabi datoteke v mapi ``data/fonts/Roboto``. Namesti vse datoteke.
//! 
//! ## Generiranje dokumentov
//! 
//! Za generiranje dokumenta uporabi ukaz:
//! 
//!     cloggen create <CSV DATOTEKA STUDIS ANKET> <JSON NABOR ODZIVOV> <TEX DOKUMENT> -f <FORMAT> -o <IZHODNA POT>
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
//! - ``<FORMAT>`` predstavlja izhodni format. Privzeta vrednost je ``pdf`` (izhod bo .pdf datoteka),
//!     lahko pa se izbere tudi ``latex`` (izhod bo .tex latex datoteka).
//! - ``<IZHODNA POT>`` predstavlja pot, kamor bo shranjen generiran dokument.
//!     Privzeto je ta vrednost enaka ``output_<TEX DOKUMENT>.<tex/pdf>``.
//! 
//! ## Združevanje STUDIS anket
//! Cloggen omogoča združevanje večih STUDIS CSV datotek v eno skupno datoteko.
//! Združijo se le povprečne ocene posameznih datotekek, tako, da se povprečijo.
//! Standardni odklon je na novo izračunan iz povprečij datotek.
//! 
//! Na primer, če imamo dve datoteki s povprečji ocen 3.2 in 5.0, potem bo novo povprečje enako 4.1, standarndi odklon pa
//! bo enak 0.90.
//! 
//! ### Uporaba
//! 
//!     cloggen merge <csv1> <csv2> ...
//! 
//! Izhodna pot združene datoteke je privzeto ``./merged.csv``. Za lastno pot uporabi ``-o <IZHODNA POT>``


use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[cfg(feature = "gui")]
mod gui;

mod compiler;
mod preproc;
mod config;
mod macros;
mod create;
mod merge;
mod fs;


#[derive(Parser)]
#[command(version)]
#[command(author = "David Hozic")]
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
        tex_template_filepath: PathBuf,

        /// Name of the grades section to use (e.g., "Anketa o izvajalcu")
        #[clap(short, long, default_value = config::create::SECTION_DEFAULT)]
        section: String,

        /// The format of output file.
        #[clap(short, long, default_value = config::create::FORMAT_DEFAULT_STR)]
        format: create::OutputFormat,

        /// Path of the output file.
        #[clap(short)]
        output_filepath: Option<PathBuf>
    },

    /// Access to the underlaying LaTeX compiler. Use this when you want to directly
    /// compile a file. If you wish to create a habilitation report, use the [`create`] command.
    Compile {
        tex_file: PathBuf
    },

    /// Merges grades of multiple CSV files
    Merge {
        /// The CSV files to merge.
        /// Each item can be either a glob pattern or an individual file.
        /// Files matched by different patterns will all be combined as one.
        #[clap(num_args = 1.., required = true)]
        csv_file_patterns: Vec<PathBuf>,

        /// Name of the grades section to use (e.g., "Anketa o izvajalcu")
        #[clap(short, long, default_value = config::merge::SECTION_DEFAULT)]
        section: String,

        /// Path of the output (merged) file.
        #[clap(short, long, default_value = config::merge::OUTPUT_DEFAULT)]
        output: PathBuf
    }
}


fn main() {
    match Args::try_parse() {  // Terminal mode
        Ok(cli) => {
            match &cli.command {
                Commands::Create {
                    studis_csv_filepath,
                    response_json_filepath,
                    tex_template_filepath,
                    section,
                    format,
                    output_filepath,
                } => {
                    create::command_create(
                        studis_csv_filepath,
                        response_json_filepath,
                        tex_template_filepath,
                        section,
                        format,
                        output_filepath
                    ).unwrap();
                }

                Commands::Compile { tex_file } => {
                    compiler::cmd_compile(tex_file).unwrap();
                }

                Commands::Merge { csv_file_patterns , section, output}  => {
                    merge::command_merge(csv_file_patterns, section, output);
                }
            }
        }
        #[cfg(feature = "gui")]
        Err(_) if std::env::args().len() == 1 => {  // Only the exe is in args => GUI mode
            gui::main_gui();
        }
        Err(e) => {
            println!("{e}");
        }
    }
}
