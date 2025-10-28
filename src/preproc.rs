use std::collections::HashMap;
use csv;


/// Possible delimiters of a CSV file.
const DELIMITERS: [char; 4] = [',', ';', '\t', ' '];


/// Finds the delimiter of a CSV file.
/// If the delimiter cannot be found, the default comma is returned.
pub fn get_delimiter(content: &str) -> char {
    enum ReadState {
        Normal,
        Quote
    }

    const MAX_CONTENT_ERR_PRINT: usize = 5000;


    let mut counts = Vec::new();
    let mut read_state = ReadState::Normal;

    for line in content.lines() {
        let mut per_line_cnt = [0; DELIMITERS.len()];
        for (i, delimiter) in DELIMITERS.iter().enumerate() {
            for c in line.chars() {
                match read_state {
                    ReadState::Normal => {
                        per_line_cnt[i] += (*delimiter == c) as usize;
                        if c == '"' {
                            read_state = ReadState::Quote;
                        }
                    },
                    // Don't consider characters inside quotes as delimiters.
                    ReadState::Quote => {
                        if c == '"' {
                            read_state = ReadState::Normal;
                        }
                    }
                }   
            }
        }

        counts.push(per_line_cnt);
    }

    for delim_i in 0..DELIMITERS.len() {
        for line_cnts_i in 0..counts.len() - 1 {
            if counts[line_cnts_i][delim_i] != counts[line_cnts_i + 1][delim_i] {
                // If the potential delimiter's count differs across lines, then this cannot
                // be a valid delimiter. We will set its count to 0 through on the first line.
                // We don't need to set the other lines, because we later only consider
                // the first line when searching for the delimiter with greatest occurrence.
                counts[0][delim_i] = 0;
                break;
            }
        }
    }

    // The detected delimiter is the one with biggest occurrence.
    // We enumerate the counts in the first line (record), maximize by its value and
    // then return the largest value's index. Then we take a look at max count's at index
    // from the possible delimiters tables.
    let (index, count) = counts[0].iter().enumerate().max_by_key(|(_, cnt)| **cnt).unwrap();
    if *count == 0 {
        panic!(
            "could not detect the CSV delimiter (inconsistent use or not a CSV). Content:\n\n{}",
            &content[0..MAX_CONTENT_ERR_PRINT.min(content.len())]
        );
    }

    DELIMITERS[index]
}


/// Preprocesses a CSV file exported from STUDIS. The result is a mapping, mapping different sections (as keys)
/// to CSV-compatible tables, which can be parsed by the csv crate.
pub fn preprocess_candidate_csv(filedata: String) -> HashMap<String, String> {   
    enum CSVParsingState {
        Header = 0,
        Columns
    }

    /// Replaces the slovenian style float notation (comma separated) with regular float notation.
    fn fix_floats(lines: &Vec<&str>) -> String {
        enum NumberState {
            NaN,
            Whole,
            Decimal,
        }
        let mut state = NumberState::NaN;
        let mut olines = Vec::with_capacity(lines.len());
        let mut chars;
        let mut sline;

        for line in lines {
            chars = line.chars();
            sline = String::with_capacity(line.len());
            for char in chars {
                sline.push(match &state {
                    NumberState::NaN => {
                        if char.is_numeric() {
                            state = NumberState::Whole;
                        }
                        char
                    },
                    NumberState::Whole => {
                        if char == ',' || char == '.' {
                            state = NumberState::Decimal;
                            '.'
                        }
                        else {
                            char
                        }
                    }
                    NumberState::Decimal => {
                        if !char.is_numeric() {
                            state = NumberState::NaN
                        }
                        char
                    }
                });
            }

            olines.push(sline);
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
                key = String::from(line.trim_matches(DELIMITERS).trim());
                key_data.clear();
                state = CSVParsingState::Columns;
            }
            CSVParsingState::Columns => {
                if line.trim_matches(DELIMITERS).trim() == "" {  // Empty line
                    map.insert(key.clone(), fix_floats(&key_data));
                    state = CSVParsingState::Header;
                }
                key_data.push(line);
            }
        }
    }

    if let CSVParsingState::Columns = state {  // In case there was no new empty line
        map.insert(key.clone(), fix_floats(&key_data));
    }

    map
}


pub fn extract_section_columns(sections: HashMap<String, String>, section: &str) -> HashMap<String, Vec<String>> {
    let csvgrades: &String = sections.get(section)
        .expect(&format!("could not find key \"{section}\" in CSV STUDIS file ({:?})", sections.keys()));

    let delimiter = get_delimiter(&csvgrades);
    let mut csvgrades = csv::ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .from_reader(csvgrades.as_bytes());

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
