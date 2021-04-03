mod assemblyscript;
mod astype;
mod error;
mod pretty_writer;

#[macro_use]
extern crate clap;

use crate::error::*;
use clap::Arg;
use std::fs::File;
use std::io::Write;

fn main() {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("module_name")
                .short("-m")
                .long("--module-name")
                .value_name("module_name")
                .help("Set the module name to use instead of reading it from the witx file"),
        )
        .arg(
            Arg::with_name("output_file")
                .short("-o")
                .long("--output")
                .value_name("output_file")
                .multiple(false)
                .help("Output file, or - for the standard output"),
        )
        .arg(
            Arg::with_name("witx_file")
                .multiple(false)
                .required(true)
                .help("WITX file"),
        )
        .get_matches();

    let writer: Box<dyn Write> = match matches.value_of("output_file") {
        None | Some("-") => Box::new(std::io::stdout()),
        Some(file) => Box::new(File::create(file).unwrap()),
    };
    let witx_file = matches.value_of("witx_file").unwrap();
    let module_name = matches.value_of("module_name").map(|x| x.to_string());
    let witx = witx::load(witx_file).unwrap();
    let mut generator = assemblyscript::Generator::new(writer, module_name);
    generator.generate(witx).unwrap();
}
