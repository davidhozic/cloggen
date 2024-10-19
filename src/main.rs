use clap::{Parser, Subcommand};
use std::collections::HashMap;
use rand::seq::SliceRandom;
use std::path::PathBuf;
use serde_json as sj;
use std::io::Read;
use std::fs::File;
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
        /// The position (habilitation) candidate wants
        habilitation: String,
        /// CSV file of the STUDIS grades
        grades_file: PathBuf,
        /// JSON file of the possible responses per category per grade.
        grade_responses: PathBuf
    }
}


/// Preprocesses a CSV file exported from STUDIS. The result is a mapping, mapping different sections (as keys)
/// to CSV-compatible tables, which can be parsed by the csv crate.
fn preprocess_candidate_csv(filedata: String) -> HashMap<String, String> {   
    enum CSVParsingState {
        Header = 0,
        Columns
    }

    let mut map = HashMap::new();
    let mut state = CSVParsingState::Header;
    let mut key= String::new();
    let mut key_data = Vec::new();

    for line in filedata.lines() {
        match state {
            CSVParsingState::Header => {
                key = String::from(line);
                key_data.clear();
                state = CSVParsingState::Columns;
            }
            CSVParsingState::Columns => {
                if line.trim() == "" {  // Empty line
                    map.insert(key.clone(), key_data.join("\n"));
                    state = CSVParsingState::Header;
                }
                key_data.push(line);
            }
        }
    }

    if let CSVParsingState::Columns = state {  // In case there was no new empty line
        map.insert(key.clone(), key_data.join("\n"));
    }

    map
}


fn command_create(habilitation: &String, grades_file: &PathBuf, grade_responses: &PathBuf) {
    const E_NOT_MAPPING: &str = "Not a JSON mapping";
    const C_CANDIDATE_KEY: &str = "Kandidat";
    const C_GRADES_KEY: &str = "Ocene";

    let mut file: File;
    let mut fdata: String= String::new();

    // Process CSV file. This is the file exported from STUDIS
    file = File::open(grades_file).unwrap();
    file.read_to_string(&mut fdata).unwrap();
    let preprocessed = preprocess_candidate_csv(fdata);
    let candidate = preprocessed.get(C_CANDIDATE_KEY)
        .expect(&format!("Could not find key {C_CANDIDATE_KEY} from {grades_file:?} ({:?})", preprocessed.keys()));
    let csvgrades = preprocessed.get(C_GRADES_KEY)
        .expect(&format!("Could not find key {C_GRADES_KEY} {grades_file:?} ({:?})", preprocessed.keys()));

    // Process JSON file. This is the file containing responses for each category and each grade.
    file = File::open(grade_responses).unwrap();   
    let json_map: sj::Map<_, _> = sj::from_reader(file).expect(E_NOT_MAPPING);
    let categories: &sj::Map<_, _> = &json_map["category"].as_object().expect(E_NOT_MAPPING);
    for (cat, grades) in categories {
        let grades : &sj::Map<_, _> = grades.as_object().expect(E_NOT_MAPPING);
    }
}


fn main() {
    let cli = Args::parse();

    match &cli.command {
        Commands::Create { habilitation, grades_file, grade_responses} => {
            command_create(habilitation, grades_file, grade_responses);
        }
    }
}
