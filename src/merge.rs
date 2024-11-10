/// Module of the ``merge`` command
use crate::preproc::{preprocess_candidate_csv, C_GRADES_KEY};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use csv;


const C_QUESTION_KEY: &str = "Vprašanje";
const C_MEAN_KEY: &str = "Povprečje";


fn csv_parse_question_means(file: &PathBuf) -> HashMap<String, f64> {
    let mut mapping = HashMap::new();

    let sections = preprocess_candidate_csv(read_to_string(&file).expect("unable to read file"));
    let sec_grades = sections.get(C_GRADES_KEY)
        .unwrap_or_else(|| panic!("csv missing candidate grades section ('{C_GRADES_KEY}')"));
    
    let mut reader = csv::Reader::from_reader(sec_grades.as_bytes());
    let headers: Vec<_> = reader.headers().unwrap().iter().map(|x| x.to_string()).collect();
    let question_idx = headers.iter().position(|x| x == &C_QUESTION_KEY).unwrap_or_else(
        || panic!("csv missing questions key ({C_QUESTION_KEY})")
    );

    let mean_idx = headers.iter().position(|x| x == &C_MEAN_KEY).unwrap_or_else(
        || panic!("csv missing mean key ({C_MEAN_KEY})")
    );

    for record in reader.records() {
        let record = record.unwrap();
        let question = &record[question_idx];
        let mean = &record[mean_idx];
        let mean: f64 = mean.parse().unwrap_or_else(|_| panic!("could not parse mean value ({mean})"));
        mapping.insert(question.to_string(), mean);
    }
    mapping
}



/// Command processing function for the ``merge`` command.
pub fn command_merge(files: &Vec<PathBuf>, output: &PathBuf) {
    let mut qvalues: HashMap<String, Vec<f64>> = HashMap::new();  // Question values

    // Iterate all files and create a mapping that maps a question to a vector of mean values.
    for file in files {
        for (question, mean) in csv_parse_question_means(file) {
            qvalues.entry(question).or_insert(Vec::new()).push(mean);
        }
    }

    // Create a mapping that maps question to the mean value (over files) of question means and standard deviation.
    let mut qmerged = HashMap::new();
    for (question, values) in qvalues {
        // Mean calculated over file question means.
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        // Standard deviation calculated over file question means.
        let std = values.iter().map(|num| (num - mean).powi(2)).sum::<f64>() / values.len() as f64;
        qmerged.insert(
            question, 
            (mean, std)
        );
    }

    println!("{qmerged:#?}");
}
