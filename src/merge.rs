/// Module of the ``merge`` command
use crate::preproc::{extract_section_columns, preprocess_candidate_csv};
use crate::fs::read_file_universal;
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::Write;
use std::fs::File;
use glob::glob;
use csv;


const C_QUESTION_KEY: &str = "Vprašanje";
const C_MEAN_KEY: &str = "Povprečje";
const C_STD_KEY: &str = "Standardni odklon";
const C_PRECISION: usize = 2;


/// Accepts a ``file`` parameter, which is a path, preprocesses it and returns a mapping
/// that maps a STUDIS question to the mean grade.
fn csv_parse_question_means(file: &PathBuf, section: &str) -> HashMap<String, f64> {
    let mut mapping = HashMap::new();
    let sections = preprocess_candidate_csv(
        read_file_universal(file).expect(&format!("unable to read file ({})", file.display()))
    );
    let extracted = extract_section_columns(sections, section);
    let questions = extracted.get(C_QUESTION_KEY).unwrap();
    let means = extracted.get(C_MEAN_KEY).unwrap();
    let mut mean;
    let mut smean;
    for (i, question) in questions.into_iter().enumerate() {
        smean = &means[i];
        mean = smean.parse().unwrap_or_else(|_| panic!("could not parse mean value ({smean})"));
        mapping.insert(question.clone(), mean);
    }

    mapping
}


/// Command processing function for the ``merge`` command.
pub fn command_merge(file_patterns: &Vec<PathBuf>, section: &str, output: &PathBuf) {
    /// Minimum number of files each pattern in `file_patterns` should match.
    /// If any matches less, a panic occurs.
    const MIN_FILES_TO_MATCH: usize = 2;

    let mut qvalues: HashMap<String, Vec<f64>> = HashMap::new();  // Question values

    // Iterate all files and create a mapping that maps a question to a vector of mean values.
    let mut files = Vec::new();
    let mut matches;
    let mut p;

    // Combine all the file patterns together
    for pattern in file_patterns {
        p = pattern.to_str().unwrap_or_else(|| panic!("{} contains invalid unicode!", pattern.display()));
        matches = glob(p).expect("invalid pattern was given");

        for entry in matches {
            match entry {
                Ok(path) => files.push(path),
                Err(e) => println!("warning: an error occurred during glob iteration. Error: {:?}", e)
            }
        }
    }

    if files.len() < MIN_FILES_TO_MATCH {
        panic!(
            "{file_patterns:?} together need to match at least {MIN_FILES_TO_MATCH} files, but they matched {}.",
            files.len()
        );
    }

    // Create mean grade mapping that maps Question => [mean grade of each file]
    for file in &files {
        for (question, mean) in csv_parse_question_means(file, section) {
            qvalues.entry(question).or_insert_with(Vec::new).push(mean);
        }
    }

    // Create a mapping that maps question to the mean value (over files) of question means and standard deviation.
    let mut qmerged = HashMap::new();
    for (question, values) in qvalues {
        // Mean calculated over file question means.
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        // Standard deviation calculated over file question means.
        let std = (values.iter().map(|num| (num - mean).powi(2)).sum::<f64>() / values.len() as f64).sqrt();
        qmerged.insert(
            question, 
            ((mean * 1000.0).round() / 1000.0, std)
        );
    }

    let mut file = File::create(output)
        .unwrap_or_else(|e| panic!("unable to open file '{}' ({e})", output.display()));
    file.write_all(section.as_bytes()).expect("unable to write grades section title");
    let mut writer = csv::Writer::from_writer(file);
    writer.write_record(&[C_QUESTION_KEY, C_MEAN_KEY, C_STD_KEY]).expect("unable to write header");
    for (k, (mean,  std)) in &qmerged {
        // Write record in format (question, mean (rounded to 4 decimals), std (rounded to 4 decimals))
        writer.write_record(&[k, &format!("{mean:.0$}", C_PRECISION), &format!("{std:.0$}", C_PRECISION)])
            .expect("unable to write record");
    }

}
