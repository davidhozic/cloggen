use std::collections::HashMap;
use csv;


/// The STUDIS file section name of candidate grades.
pub const C_GRADES_KEY: &str = "Anketa o izvajalcu";


/// Preprocesses a CSV file exported from STUDIS. The result is a mapping, mapping different sections (as keys)
/// to CSV-compatible tables, which can be parsed by the csv crate.
pub fn preprocess_candidate_csv(filedata: String) -> HashMap<String, String> {   
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


pub fn parse_grades_section(sections: HashMap<String, String>) -> HashMap<String, Vec<String>> {
    let csvgrades: &String = sections.get(C_GRADES_KEY)
        .expect(&format!("could not find key \"{C_GRADES_KEY}\" in CSV STUDIS file ({:?})", sections.keys()));

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
        csvgrades.insert(header.to_string(), column_data);
    }
    csvgrades
}