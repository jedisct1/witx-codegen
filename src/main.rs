#![forbid(unsafe_code)]

use anyhow::Error;
use structopt::StructOpt;

fn main() -> Result<(), Error> {
    // Load options from CLI
    let cfg = witx_codegen::Config::from_args();

    // Generate outputs
    witx_codegen::generate(&cfg)?;

    Ok(())
}
