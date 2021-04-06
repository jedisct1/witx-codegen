use super::*;
use std::io::Write;

impl DocGenerator {
    fn define_union_member<T: Write>(
        w: &mut PrettyWriter<T>,
        _union_name: &str,
        _i: usize,
        member: &ASUnionMember,
    ) -> Result<(), Error> {
        let member_type = member.type_.as_ref();
        w.write_line(format!(
            "{}: {}",
            member.name.as_var(),
            member_type.as_lang(),
        ))?;
        Ok(())
    }

    pub fn define_as_union<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        union_: &ASUnion,
    ) -> Result<(), Error> {
        let tag_repr = union_.tag_repr.as_ref();
        w.write_lines(format!(
            "### {}\nTagged union with tag type: {} and the following possibilities:",
            name.as_type(),
            tag_repr.as_lang()
        ))?
        .eob()?;
        {
            let mut w = w.new_block();
            for (i, member) in union_.members.iter().enumerate() {
                Self::define_union_member(&mut w, name, i, member)?;
            }
        }
        Ok(())
    }
}
