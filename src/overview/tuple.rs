use std::io::Write;

use super::*;

impl OverviewGenerator {
    pub fn define_as_tuple<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        members: &[ASTupleMember],
    ) -> Result<(), Error> {
        w.write_line(format!(
            "tuple {} = ({})",
            name.as_type(),
            members
                .iter()
                .map(|member| { member.type_.as_lang() })
                .collect::<Vec<_>>()
                .join(", ")
        ))?;
        Ok(())
    }
}
