use clap::{Parser, Subcommand};
use std::collections::HashMap;
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

    /// Replaces '\t' with commas and fixes the numbers to be separated by '.' instead of ','.
    fn fix_content(lines: &Vec<&str>) -> String {
        let mut olines = Vec::with_capacity(lines.len());
        let mut line_out;
        let mut split_i;
        for line in lines {
            split_i = line.find("\t").unwrap();
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


fn command_create(habilitation: &String, grades_file: &PathBuf, grade_responses: &PathBuf) {
    const E_NOT_MAPPING: &str = "Not a JSON mapping";
    const C_GRADES_KEY: &str = "Anketa o izvajalcu";
    const C_MEAN_CSV_KEY: &str = "Povprečje";
    const C_STD_CSV_KEY: &str = "Standardni odklon";
    const C_JSON_MAP_QUESTION_KEY: &str = "Vprašanje";

    let mut file: File;
    let mut fdata: String= String::new();

    // Process CSV file. This is the file exported from STUDIS
    file = File::open(grades_file).unwrap();
    file.read_to_string(&mut fdata).unwrap();
    let preprocessed = preprocess_candidate_csv(fdata);
    let csvgrades: &String = preprocessed.get(C_GRADES_KEY)
        .expect(&format!("Could not find key \"{C_GRADES_KEY}\" in {grades_file:?} ({:?})", preprocessed.keys()));
    
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
    file = File::open(grade_responses).unwrap();   
    let json_map: sj::Map<_, _> = sj::from_reader(file).expect(E_NOT_MAPPING);
    let categories: &sj::Map<_, _> = &json_map[C_JSON_MAP_QUESTION_KEY].as_object().expect(E_NOT_MAPPING);
    let mut idx: usize;
    let mut mean: f64;
    let mut std: f64;
    for (cat, grades) in categories {
        idx = csvgrades.get(C_JSON_MAP_QUESTION_KEY).expect("CSV is missing questions key.")
            .iter().position(|x| x == cat).expect(&format!("CSV is missing category \"{cat}\""));
        mean = csvgrades.get(C_MEAN_CSV_KEY).expect("CSV is missing the mean grade value key")[idx].parse().unwrap();
        std = csvgrades.get(C_STD_CSV_KEY).expect("CSV is missing the std of grade key")[idx].parse().unwrap();

        let grades_map = grades.as_object().expect(E_NOT_MAPPING);
        let mut grades: Vec<f64> = grades_map.keys().map(|key| key.parse()
            .expect(&format!("the grades in respose file should be floating-point, which \"{key}\" is not"))).collect();
        grades.sort_by(|a, b| b.partial_cmp(a).unwrap());
        dbg!(mean >= grades[0]);
    }
}


fn main() {
    // let cli = Args::parse();

    // match &cli.command {
    //     Commands::Create { habilitation, grades_file, grade_responses} => {
    //         command_create(habilitation, grades_file, grade_responses);
    //     }
    // }
    command_create(&String::from("Asistent"), &PathBuf::from("ocena.csv"), &PathBuf::from("mnenje.json"));
}
