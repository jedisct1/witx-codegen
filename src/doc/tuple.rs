use super::*;
use std::io::Write;

impl DocGenerator {
    pub fn define_as_tuple<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        members: &[ASTupleMember],
    ) -> Result<(), Error> {
        w.write_lines(format!(
            "### {}\nTuple, representing ({}).",
            name.as_type(),
            members
                .iter()
                .map(|member| { member.type_.as_lang() })
                .collect::<Vec<_>>()
                .join(", ")
        ))?
        .eob()?;
        Ok(())
    }
}
