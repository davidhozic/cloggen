use std::fs::{read_to_string, File};
use std::path::PathBuf;
use tectonic as tec;
use std::io::Write;
use std::env;


/// Modification of [`tectonic::latex_to_pdf`] which adds stdout print to the console.
pub fn compile_latex(latex: impl AsRef<str>) -> Vec<u8> {
    let mut status: tectonic::status::NoopStatusBackend = tec::status::NoopStatusBackend::default();
    let config = tec::config::PersistentConfig::open(false).expect("could not open config");
    let bundle = config.default_bundle(false, &mut status).expect("could not get bundle");
    let mut files = {
        let mut sess;
        let mut sb = tec::driver::ProcessingSessionBuilder::default();
        let format_cache_path = config.format_cache_path().expect("could not get format cache path");
        sb.bundle(bundle)
            .primary_input_buffer(latex.as_ref().as_bytes())
            .tex_input_name("texput.tex")
            .format_name("latex")
            .format_cache_path(format_cache_path)
            .keep_logs(false)
            .keep_intermediates(false)
            .print_stdout(false)
            .output_format(tec::driver::OutputFormat::Pdf)
            .do_not_write_output_files();
        sess = sb.create(&mut status).unwrap();
        sess.run(&mut status).unwrap_or_else(|_| panic!("{}", String::from_utf8(sess.get_stdout_content()).unwrap()));
        sess.into_file_data()
    };
    files.remove("texput.pdf").expect("compilation was successful but file data was not created").data
}


pub fn cmd_compile(path: &PathBuf) {
    let fdata = read_to_string(path).expect("unable to read file");
    let root = env::current_dir().unwrap();
    std::env::set_current_dir(path.parent().unwrap()).unwrap();
    let compiled = compile_latex(fdata);
    std::env::set_current_dir(root).unwrap();
    let mut file = File::create(path.display().to_string() + ".pdf").expect("unable to create output file");
    file.write_all(&compiled).expect("unable to write latex to file");
}
