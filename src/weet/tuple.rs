use std::io::Write;

use super::*;

pub struct Tuple;

impl Tuple {
    pub fn name_for(tuple_members: &[ASTupleMember]) -> String {
        format!(
            "tuple<{}>",
            tuple_members
                .iter()
                .map(|member| member.type_.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl WeetGenerator {
    pub fn define_as_tuple<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        members: &[ASTupleMember],
    ) -> Result<(), Error> {
        w.indent()?.write(format!("{}: tuple<", name.as_type()))?;
        {
            let mut w = w.new_block();
            let mut first = true;
            for (member) in members {
                if !first {
                    w.write(", ")?;
                }
                first = false;
                let member_type = member.type_.as_ref();
                w.write(format!("{}", member_type.as_lang()))?;
            }
        }
        w.write(">;")?.eol()?;
        Ok(())
    }
}
