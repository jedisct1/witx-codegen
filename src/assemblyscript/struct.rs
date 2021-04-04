use super::*;
use std::io::Write;

impl Generator {
    pub fn define_as_struct<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        members: &[ASStructMember],
    ) -> Result<(), Error> {
        w.write_line("// @ts-ignore: decorator")?
            .write_line("@unmanaged")?
            .write_line(format!("export class {} {{", name.as_type()))?;
        {
            let mut w = w.new_block();
            for member in members {
                let member_type = member.type_.as_ref();
                w.write_line(format!("{}: {};", member.name.as_var(), member_type))?;

                let pad_len = member.padding;
                for i in 0..(pad_len & 1) {
                    w.write_line(format!("private __pad8_{}: u8;", i))?;
                }
                for i in 0..(pad_len & 3) / 2 {
                    w.write_line(format!("private __pad16_{}: u16;", i))?;
                }
                for i in 0..(pad_len & 7) / 4 {
                    w.write_line(format!("private __pad32_{}: u32;", i))?;
                }
                for i in 0..pad_len / 8 {
                    w.write_line(format!("private __pad64_{}: u64;", i))?;
                }
            }
        }
        w.write_line("}")?.eob()?;
        Ok(())
    }
}
