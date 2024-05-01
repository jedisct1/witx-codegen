use std::io::Write;

use super::*;

impl WeetGenerator {
    pub fn define_as_struct<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        members: &[ASStructMember],
    ) -> Result<(), Error> {
        w.write_line(format!("record {} {{", name.as_type()))?;
        {
            let mut w = w.new_block();
            for member in members {
                let member_type = member.type_.as_ref();
                w.write_line(format!(
                    "{}: {},",
                    member.name.as_var(),
                    member_type.as_lang()
                ))?;
            }
        }
        w.write_line("};")?.eob()?;
        Ok(())
    }
}
