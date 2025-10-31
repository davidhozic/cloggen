//! Common configuration.
/// Constants used for the Create command.
#[allow(unused)]
pub mod create {
    use crate::create::OutputFormat;

    /// Default section inside the CSV file to parse. STUDIS CSV files can have multiple sections ---
    /// e.g., section about the subject, section about the teacher, etc.
    pub const SECTION_DEFAULT: &str = "Anketa o izvajalcu";
    
    /// Default output format.
    pub const FORMAT_DEFAULT: OutputFormat = OutputFormat::Pdf;
    pub const FORMAT_DEFAULT_STR: &str = "pdf";
}

/// Constants used for the Merge command.
pub mod merge {
    /// Default section inside the CSV file to parse. STUDIS CSV files can have multiple sections ---
    /// e.g., section about the subject, section about the teacher, etc.
    pub const SECTION_DEFAULT: &str = super::create::SECTION_DEFAULT;
    /// The default output file of the merged CSV data.
    pub const OUTPUT_DEFAULT: &str = "merged.csv";
}
