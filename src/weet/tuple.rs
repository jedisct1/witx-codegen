use std::io::Write;

use super::*;

pub struct Tuple;

impl Tuple {
    pub fn name_for(tuple_members: &[ASTupleMember]) -> String {
        format!(
            "tuple<{}>",
            tuple_members
                .iter()
                .map(|member| member.type_.to_string().as_type())
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
        w.write_line(format!("{}: tuple<{}>;", name.as_type(), Tuple::name_for(members)))?;
        Ok(())
    }
}
