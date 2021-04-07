#![forbid(unsafe_code)]

mod assemblyscript;
mod astype;
mod doc;
mod error;
mod generator;
mod overview;
mod pretty_writer;
mod rust;
mod zig;

#[macro_use]
extern crate clap;

use crate::error::*;
use crate::generator::*;
use clap::Arg;
use std::fs::File;
use std::io::Write;

pub struct Options {
    skip_imports: bool,
    skip_header: bool,
}

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
            Arg::with_name("skip_imports")
                .short("I")
                .long("--skip-imports")
                .help("Ignores imported types and functions"),
        )
        .arg(
            Arg::with_name("skip_header")
                .short("H")
                .long("--skip-header")
                .help("Do not generate a header"),
        )
        .arg(
            Arg::with_name("witx_files")
                .multiple(true)
                .required(true)
                .help("WITX files"),
        )
        .arg(
            Arg::with_name("output_type")
                .short("-t")
                .long("--output-type")
                .value_name("output_type")
                .multiple(false)
                .default_value("assemblyscript")
                .help("Output type. One in: {assemblyscript, zig, rust, overview, markdown}"),
        )
        .get_matches();
    // generate all or generate no heade,r no imports
    let mut writer: Box<dyn Write> = match matches.value_of("output_file") {
        None | Some("-") => Box::new(std::io::stdout()),
        Some(file) => Box::new(File::create(file).unwrap()),
    };
    let module_name = matches.value_of("module_name").map(|x| x.to_string());
    let skip_imports = matches.is_present("skip_imports");
    let skip_header = matches.is_present("skip_header");
    let mut options = Options {
        skip_imports,
        skip_header,
    };
    let output_type = matches.value_of("output_type").unwrap();
    let witx_files = matches.values_of("witx_files").unwrap();
    for witx_file in witx_files {
        let witx = witx::load(witx_file).unwrap();
        let generator = match output_type {
            "assemblyscript" => Box::new(assemblyscript::AssemblyScriptGenerator::new(
                module_name.clone(),
            )) as Box<dyn Generator<_>>,
            "zig" => Box::new(zig::ZigGenerator::new(module_name.clone())) as Box<dyn Generator<_>>,
            "rust" => {
                Box::new(rust::RustGenerator::new(module_name.clone())) as Box<dyn Generator<_>>
            }
            "overview" => Box::new(overview::OverviewGenerator::new(module_name.clone()))
                as Box<dyn Generator<_>>,
            "markdown" | "doc" => {
                Box::new(doc::DocGenerator::new(module_name.clone())) as Box<dyn Generator<_>>
            }
            _ => panic!("Unsupported output type"),
        };
        generator.generate(&mut writer, witx, &options).unwrap();
        options.skip_imports = true;
        options.skip_header = true;
    }
}
