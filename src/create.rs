/// Module related to the ``create`` command.

use rand::distributions::Uniform;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use serde_json as sj;
use rand::{thread_rng, Rng};
use clap::ValueEnum;
use std::fs::File;
use std::time;
use std::env;

use crate::with_parent_path;
use crate::compiler;
use crate::preproc;
use crate::fs;


#[derive(ValueEnum, Clone)]
pub enum OutputFormat {
    #[clap(alias = "tex")]
    Latex,
    Pdf
}


/// Function that processes the CLI command ``create``
/// It returns a string representing the output file's path.
pub fn command_create(
    studis_csv_filepath: &PathBuf,
    response_json_filepath: &PathBuf,
    tex_template_filepath: &PathBuf,
    section: &String,
    format: &OutputFormat,
    output_filepath: &Option<PathBuf>
) -> String {
    const E_NOT_MAPPING: &str = "Not a JSON mapping";

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
    file.read_to_string(&mut output_fdata).expect(&format!("Unable to read file {tex_template_filepath:?}"));

    if !output_fdata.contains(C_OUTPUT_LATEX_REPLACE_KEY) {
        panic!(
            "output file ({tex_template_filepath:?}) does not mark the location \
            of automatically-generated content (generated by this script). Mark it by writing \
            \"{C_OUTPUT_LATEX_REPLACE_KEY}\" somewhere in the file"
        );
    }

    // Process STUDIS CSV file.
    let fdata = fs::read_file_universal(studis_csv_filepath).expect("unable to read STUDIS CSV");
    let csvgrades = preproc::extract_section_columns(preproc::preprocess_candidate_csv(fdata), section);

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

        let epoch_ns;
        let responses;
        let response_idx;
        let response;
        let response_str;
        for (sgrade, grade) in grades.iter() {
            if (grade * 10000.0) as usize <= (mean * 10000.0) as usize {  // Prevent influence of numeric error
                let v = grades_json.get(*sgrade).unwrap();

                // Query elapsed nanoseconds in order to improve randomness.
                epoch_ns = time::SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)
                    .expect("SystemTime is before EPOCH!")
                    .as_nanos();
                responses = v.as_array()
                    .expect(&format!("value of Category->Grade->Value must be an array of strings. Found {v:?}"));

                if responses.len() == 0 {
                    panic!("there are no defined responses for grade {sgrade}, category {cat:?}");
                }

                response_idx = (
                    (epoch_ns % responses.len() as u128
                    + rgn.sample(Uniform::new(0, responses.len())) as u128) % responses.len() as u128
                ) as usize;
                response = &responses[response_idx];
                response_str = response.as_str().expect(&format!("responses must be strings ({response} is not)"));
                output_parts.push(
                    response_str.replace(C_OUTPUT_LATEX_MEAN_KEY, smean)
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

    // Insert the generated LaTeX into our TeX source file
    output_fdata = output_fdata.replace(C_OUTPUT_LATEX_REPLACE_KEY, &(output_parts.join("\n")));

    // If no output path is given, assume the source file without extension as a basename, otherwise use the given path.
    let mut output = match output_filepath {
        Some(path) => path.display().to_string(),
        None => {
            // We can unwrap here because we tested the path by opening the file above.
            let mut filename = tex_template_filepath.file_name().unwrap().to_str().unwrap();

            // Find index of the last '.' and split, removing any file-ending.
            if let Some (idx) = filename.chars().rev().position(|x| x == '.') {
                (filename, _) = filename.split_at(filename.len() - idx - 1);
            }
            tex_template_filepath.parent()
                .unwrap_or(Path::new("./"))
                .join(format!("out_{filename}"))
                .display()
                .to_string()
        }
    };

    match format {
        OutputFormat::Latex => {
            if !output.ends_with(".tex") {
                output += ".tex";
            }

            file = File::create(&output).expect("could not write output LaTex");
            file.write_all(output_fdata.as_bytes()).unwrap();
        },
        OutputFormat::Pdf => {
            if !output.ends_with(".pdf") {
                output += ".pdf";
            }

            let pdfdata = with_parent_path!(tex_template_filepath, {compiler::compile_latex(output_fdata)});
            file = File::create(&output).expect("could not create final PDF");
            file.write_all(&pdfdata).unwrap();
        }
    }

    return output;
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use super::*;

    const CSV: &str = "anketa.csv";
    const JSON: &str = "odzivi.json";
    const LATEX: &str = "mnenja-template/mnenje.tex";

    #[test]
    fn test_create() {
        let path = command_create(
            &PathBuf::from(CSV),
            &PathBuf::from(JSON),
            &PathBuf::from(LATEX),
            &"Anketa o izvajalcu".to_string(),
            &OutputFormat::Pdf,
            &None
        );
        std::fs::remove_file(path).expect("ouput PDF file not created");
    }
}
