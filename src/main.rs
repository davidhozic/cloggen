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
use std::collections::HashMap;
use rand::seq::SliceRandom;
use std::io::{Read, Write};
use std::path::PathBuf;
use serde_json as sj;
use rand::thread_rng;
use tectonic as tec;
use std::fs::File;
use std::env;
use csv;


#[derive(Parser)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Commands
}


#[derive(Subcommand)]
enum Commands {
    /// Generate a habilitation report
    Create {
        /// CSV file of the STUDIS grades
        studis_csv_filepath: PathBuf,
        /// JSON file of the possible responses per category per grade.
        response_json_filepath: PathBuf,
        /// Path to the output LaTeX file.
        tex_template_filepath: PathBuf
    }
}


/// Preprocesses a CSV file exported from STUDIS. The result is a mapping, mapping different sections (as keys)
/// to CSV-compatible tables, which can be parsed by the csv crate.
fn preprocess_candidate_csv(filedata: String) -> HashMap<String, String> {   
    enum CSVParsingState {
        Header = 0,
        Columns
    }

    /// Replaces '\t' with commas and fixes the numbers to be separated by '.' instead of ','.
    fn fix_content(lines: &Vec<&str>) -> String {
        let mut olines = Vec::with_capacity(lines.len());
        let mut line_out;
        let mut split_i;
        for line in lines {
            split_i = line.find("\t").expect("Could not find '\t' when processing CSV");
            line_out = (format!("\"{}\"", &line[..split_i]) + &line[split_i..].replace(",", ".")).replace("\t", ",");
            olines.push(line_out);
        }
        olines.join("\n")
    }

    let mut map = HashMap::new();
    let mut state = CSVParsingState::Header;
    let mut key= String::new();
    let mut key_data = Vec::new();
    for line in filedata.lines() {
        match state {
            CSVParsingState::Header => {
                key = String::from(line.trim());
                key_data.clear();
                state = CSVParsingState::Columns;
            }
            CSVParsingState::Columns => {
                if line.trim() == "" {  // Empty line
                    map.insert(key.clone(), fix_content(&key_data));
                    state = CSVParsingState::Header;
                }
                key_data.push(line);
            }
        }
    }

    if let CSVParsingState::Columns = state {  // In case there was no new empty line
        map.insert(key.clone(), fix_content(&key_data));
    }

    map
}


/// Modification of [`tectonic::latex_to_pdf`] which adds stdout print to the console.
fn compile_latex(latex: impl AsRef<str>) -> Vec<u8> {
    let mut status: tectonic::status::NoopStatusBackend = tec::status::NoopStatusBackend::default();
    let config = tec::config::PersistentConfig::open(false).expect("could not open config");
    let bundle = config.default_bundle(false, &mut status).expect("could not get bundle");
    let mut files = {
        // Looking forward to non-lexical lifetimes!
        let mut sess;
        let mut sb = tec::driver::ProcessingSessionBuilder::default();
        let format_cache_path = config.format_cache_path().expect("could not get format cache path");
        sb.bundle(bundle)
            .primary_input_buffer(latex.as_ref().as_bytes())
            .tex_input_name("texput.tex")
            .format_name("latex")
            .format_cache_path(format_cache_path)
            .keep_logs(false)
            .keep_intermediates(false)
            .print_stdout(false)
            .output_format(tec::driver::OutputFormat::Pdf)
            .do_not_write_output_files();
        sess = sb.create(&mut status).unwrap();
        sess.run(&mut status).unwrap_or_else(|_| panic!("{}", String::from_utf8(sess.get_stdout_content()).unwrap()));
        sess.into_file_data()
    };
    files.remove("texput.pdf").expect("compilation was successful but file data was not created").data
}


fn command_create(studis_csv_filepath: &PathBuf, response_json_filepath: &PathBuf, tex_template_filepath: &PathBuf) {
    const E_NOT_MAPPING: &str = "Not a JSON mapping";

    const C_GRADES_KEY: &str = "Anketa o izvajalcu";
    const C_MEAN_CSV_KEY: &str = "Povprečje";
    const C_STD_CSV_KEY: &str = "Standardni odklon";
    const C_JSON_MAP_QUESTION_KEY: &str = "Vprašanje";

    const C_OUTPUT_LATEX_REPLACE_KEY: &str = "{AUTO_GEN}";
    const C_OUTPUT_LATEX_MEAN_KEY: &str = "{MEAN}";
    const C_OUTPUT_LATEX_STD_KEY: &str = "{STD}";

    // Check the output file
    let mut file: File;
    let mut output_fdata: String = String::new();
    let mut output_parts = Vec::new();
    file = File::open(tex_template_filepath).expect(&format!("could not open tex file ({tex_template_filepath:?})"));
    file.read_to_string(&mut output_fdata).unwrap();

    if !output_fdata.contains(C_OUTPUT_LATEX_REPLACE_KEY) {
        panic!(
            "output file ({tex_template_filepath:?}) does not mark the location \
            of automatically-generated content (generated by this script). Mark it by writing \
            \"{C_OUTPUT_LATEX_REPLACE_KEY}\" somewhere in the file"
        );
    }

    // Process CSV file. This is the file exported from STUDIS
    let mut fdata= String::new();
    file = File::open(studis_csv_filepath).expect(&format!("could not open studis csv file ({studis_csv_filepath:?})"));
    file.read_to_string(&mut fdata).unwrap();
    let preprocessed = preprocess_candidate_csv(fdata);
    let csvgrades: &String = preprocessed.get(C_GRADES_KEY)
        .expect(&format!("could not find key \"{C_GRADES_KEY}\" in {studis_csv_filepath:?} ({:?})", preprocessed.keys()));

    let mut csvgrades = csv::Reader::from_reader(csvgrades.as_bytes());
    let headers = csvgrades.headers().unwrap().clone();
    let records: Vec<Vec<String>> = csvgrades.records()
        .map(|x| x.unwrap().iter().map(|x| x.to_string()).collect()).collect();

    let mut csvgrades = HashMap::new();
    for (c, header) in headers.iter().enumerate() {
        let mut column_data = Vec::new();
        for record in &records {
            column_data.push(record[c].clone());
        }
        csvgrades.insert(header, column_data);
    }

    // Process JSON file. This is the file containing responses for each category and each grade.
    file = File::open(response_json_filepath).expect(&format!("could not open responses file ({response_json_filepath:?})"));   
    let json_map: sj::Map<_, _> = sj::from_reader(file).expect(E_NOT_MAPPING);
    let categories: &sj::Map<_, _> = &json_map[C_JSON_MAP_QUESTION_KEY].as_object().expect(E_NOT_MAPPING);
    let mut idx: usize;
    let mut start_size: usize;

    let mut mean: f64;
    // let mut std: f64;
    let mut smean: &str;
    let mut sstd: &str;

    let mut rgn = thread_rng();

    // Iterate each category/question of the JSON responses file
    for (cat, grades_json) in categories {
        // Get index of the question matching JSON category
        idx = csvgrades.get(C_JSON_MAP_QUESTION_KEY).expect("CSV is missing questions key.")
            .iter().position(|x| x == cat).expect(&format!("CSV is missing category \"{cat}\""));


        // Read the String of the mean and std, then parse them to float
        smean = &csvgrades.get(C_MEAN_CSV_KEY).expect("CSV is missing the mean grade value key")[idx];
        mean = smean.parse().unwrap();
        sstd = &csvgrades.get(C_STD_CSV_KEY).expect("CSV is missing the std of grade key")[idx];
        // std = sstd.parse().unwrap();

        // Obtain the mapping of min. grade => array of String responses
        let grades_json = grades_json.as_object().cloned().expect(E_NOT_MAPPING);
        let mut grades: Vec<(&String, f64)> = grades_json.keys().map(
            // Save grades in format (String (original), parsed float)
            |x| (x, x.parse().expect(&format!("grades must be floats (\"{x}\")")))
        ).collect();

        // Sort the grades by the parsed value
        grades.sort_by(|(_, a), (_, b)| b.total_cmp(a));

        // Iterate sorted keys from largest grade to lowest, compare each parsed grade to the mean value of CSV grades
        // and stop when we find the key that is lower or equal than the mean.
        start_size = output_parts.len();
        for (sgrade, grade) in grades.iter() {
            if (grade * 10000.0) as usize <= (mean * 10000.0) as usize {  // Prevent influence of numeric error
                let v = grades_json.get(*sgrade).unwrap();
                let response = v.as_array()
                    .expect(&format!("value of Category->Grade->Value must be an array of strings. Found {v:?}"))
                    .choose(&mut rgn)
                    .expect(&format!("there are no defined responses for grade {sgrade}, category {cat:?}"));
                let response = response.as_str().expect(&format!("responses must be strings ({response} is not)"));
                output_parts.push(
                    response.replace(C_OUTPUT_LATEX_MEAN_KEY, smean)
                            .replace(C_OUTPUT_LATEX_STD_KEY, sstd)
                );
                break;
            }
        }

        // Check if loop was not break-ed;
        if start_size == output_parts.len() {
            panic!("could not find grade below mean ({mean}) for category \"{cat}\"");
        }
    }

    // Write output file
    output_fdata = output_fdata.replace(C_OUTPUT_LATEX_REPLACE_KEY, &(output_parts.join("\n\n")));
    let root = env::current_dir().unwrap();

    // Compile latex in the same directory as the main TeX file.
    std::env::set_current_dir(tex_template_filepath.parent().unwrap()).unwrap();
    let pdfdata = compile_latex(output_fdata);
    std::env::set_current_dir(root).unwrap();

    file = File::create(format!("{}.pdf", tex_template_filepath.display())).expect("could not create final PDF");
    file.write_all(&pdfdata).unwrap();
}


fn main() {
    let cli = Args::parse();

    match &cli.command {
        Commands::Create { studis_csv_filepath, response_json_filepath, tex_template_filepath } => {
            command_create(studis_csv_filepath, response_json_filepath, tex_template_filepath);
        }
    }
    // command_create(&PathBuf::from("ocena.csv"), &PathBuf::from("mnenje.json"), &PathBuf::from("mnenje.tex"));
}
