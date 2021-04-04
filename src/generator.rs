use super::*;
use crate::error::*;
use std::io::Write;

pub trait Generator<T: Write> {
    fn generate(
        &self,
        writer: &mut T,
        module_witx: witx::Module,
        options: &Options,
    ) -> Result<(), Error>;
}
