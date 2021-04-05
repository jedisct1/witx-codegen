use super::*;
use std::io::Write;

impl RustGenerator {
    pub fn header<T: Write>(w: &mut PrettyWriter<T>) -> Result<(), Error> {
        w.write_lines(
            "
//
// This file was automatically generated by witx-codegen - Do not edit manually.
//",
        )?;
        w.write_lines(
            "
pub type WasiHandle = i32;
pub type Char8 = u8;
pub type Char32 = u32;
",
        )?;
        w.eob()?;
        Ok(())
    }
}
