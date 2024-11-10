/// Module of the ``merge`` command
use crate::preproc::{preprocess_candidate_csv, C_GRADES_KEY};
use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::PathBuf;
use csv;


const C_QUESTION_KEY: &str = "Vprašanje";
const C_MEAN_KEY: &str = "Povprečje";
const C_STD_KEY: &str = "Standardni odklon";
const C_PRECISION: usize = 2;


/// Accepts a ``file`` parameter, which is a path, preprocesses it and returns a mapping
/// that maps a STUDIS question to the mean grade.
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
    let mut file = File::create(output)
        .unwrap_or_else(|e| panic!("unable to open file '{}' ({e})", output.display()));
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
            ((mean * 1000.0).round() / 1000.0, std)
        );
    }

    file.write_all((C_GRADES_KEY.to_string() + "\n").as_bytes()).expect("unable to write grades section title");
    let mut writer = csv::Writer::from_writer(file);
    writer.write_record(&[C_QUESTION_KEY, C_MEAN_KEY, C_STD_KEY]).expect("unable to write header");
    for (k, (mean,  std)) in &qmerged {
        // Write record in format (question, mean (rounded to 4 decimals), std (rounded to 4 decimals))
        writer.write_record(&[k, &format!("{mean:.0$}", C_PRECISION), &format!("{std:.0$}", C_PRECISION)])
            .expect("unable to write record");
    }

}
