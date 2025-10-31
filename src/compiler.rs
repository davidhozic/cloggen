use tectonic::driver::{OutputFormat, ProcessingSessionBuilder};
use tectonic::status::termcolor::TermcolorStatusBackend;
use tectonic::config::PersistentConfig;
use tectonic::status::ChatterLevel;


use std::fs::{read_to_string, File};
use std::time::SystemTime;
use std::path::PathBuf;
use std::io::Write;
use std::env;

use crate::with_parent_path;


/// Modification of [`tectonic::latex_to_pdf`] which adds stdout print to the console.
pub fn compile_latex(latex: impl AsRef<str>) -> std::io::Result<Vec<u8>> {
    let mut status = TermcolorStatusBackend::new(ChatterLevel::Normal);
    let config = PersistentConfig::open(false)?;
    let bundle = config.default_bundle(false, &mut status)?;
    let mut files = {
        let mut sess;
        let mut sb = ProcessingSessionBuilder::default();
        let format_cache_path = config.format_cache_path()?;
        sb.bundle(bundle)
            .primary_input_buffer(latex.as_ref().as_bytes())
            .tex_input_name("texput.tex")
            .format_name("latex")
            .format_cache_path(format_cache_path)
            .keep_logs(false)
            .keep_intermediates(false)
            .print_stdout(false)
            .output_format(OutputFormat::Pdf)
            .build_date(SystemTime::now())
            .do_not_write_output_files();
        sess = sb.create(&mut status)?;
        sess.run(&mut status)?;        
        sess.into_file_data()
    };
    Ok(files.remove("texput.pdf").expect("compilation was successful but file data was not created").data)
}


pub fn cmd_compile(path: &PathBuf) -> std::io::Result<()> {
    let fdata = read_to_string(path)?;
    let compiled = with_parent_path!(path, {compile_latex(fdata)?});
    let mut file = File::create(path.display().to_string() + ".pdf")?;
    file.write_all(&compiled)?;
    Ok(())
}
