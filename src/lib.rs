#![forbid(unsafe_code)]

use std::fs::File;
use std::io::Write;

use structopt::StructOpt;

use strum::VariantNames;
use strum_macros::{Display, EnumString, EnumVariantNames};

mod assemblyscript;
mod astype;
mod doc;
mod error;
mod overview;
mod pretty_writer;
mod rust;
mod zig;

pub use crate::error::*;

/// Generator output types
#[derive(Debug, Copy, Clone, PartialEq, Display, EnumString, EnumVariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum OutputType {
    AssemblyScript,
    Rust,
    Zig,
    Overview,
    #[strum(serialize = "doc", serialize = "markdown")]
    Doc,
}

#[derive(Debug, Clone, PartialEq, StructOpt)]
pub struct Config {
    /// Set the module name to use instead of reading it from the witx file
    #[structopt(short, long)]
    pub module_name: Option<String>,

    /// Output file, or - for the standard output
    #[structopt(short, long)]
    pub output_file: Option<String>,

    /// WITX files
    #[structopt()]
    pub witx_files: Vec<String>,

    /// Output type
    #[structopt(short="t", long, possible_values=OutputType::VARIANTS)]
    pub output_type: OutputType,

    #[structopt(flatten)]
    pub flags: Options,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            module_name: None,
            output_file: None,
            witx_files: vec![],
            output_type: OutputType::Doc,
            flags: Options {
                skip_header: false,
                skip_imports: false,
            },
        }
    }
}

/// Options for WITX generators
#[derive(Debug, Clone, PartialEq, StructOpt)]
pub struct Options {
    /// Ignores imported types and functions
    #[structopt(long)]
    skip_imports: bool,

    /// Do not generate a header
    #[structopt(long)]
    skip_header: bool,
}

/// Abstract generator interface
pub trait Generator<T: Write> {
    fn generate(
        &self,
        writer: &mut T,
        module_witx: witx::Module,
        options: &Options,
    ) -> Result<(), Error>;
}

fn get_generator<T: Write>(module: Option<&str>, output: OutputType) -> Box<dyn Generator<T>> {
    let m = module.map(|v| v.to_string());

    match output {
        OutputType::AssemblyScript => Box::new(assemblyscript::AssemblyScriptGenerator::new(m)),
        OutputType::Zig => Box::new(zig::ZigGenerator::new(m)),
        OutputType::Rust => Box::new(rust::RustGenerator::new(m)),
        OutputType::Overview => Box::new(overview::OverviewGenerator::new(m)),
        OutputType::Doc => Box::new(doc::DocGenerator::new(m)),
    }
}

/// Generate sources from WITX files using the provided config
pub fn generate(cfg: &Config) -> Result<(), Error> {
    // generate all or generate no header no imports

    // Setup writer based on output file config
    let mut writer: Box<dyn Write> = match cfg.output_file.as_deref() {
        None | Some("-") => Box::new(std::io::stdout()),
        Some(file) => Box::new(File::create(file).unwrap()),
    };

    let mut flags = cfg.flags.clone();

    for witx_file in &cfg.witx_files {
        // Parse WITX file
        let witx = witx::load(witx_file).unwrap();

        // Create generator for the specified output type
        let generator = get_generator(cfg.module_name.as_deref(), cfg.output_type);

        // Generate output file
        generator.generate(&mut writer, witx, &flags).unwrap();

        // Generate definitions only once if we have multiple input files
        flags.skip_imports = true;
        flags.skip_header = true;
    }

    Ok(())
}
