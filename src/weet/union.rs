use std::io::Write;

use super::*;

impl WeetGenerator {
    fn define_union_member_accessors<T: Write>(
        w: &mut PrettyWriter<T>,
        union_name: &str,
        _i: usize,
        member: &ASUnionMember,
    ) -> Result<(), Error> {
        let name = &member.name;
        let member_is_void = matches!(member.type_.as_ref(), ASType::Void);

        if member_is_void {
            // new_*
            w.write_line(format!(
                "fn new{}() {} {{",
                name.as_fn_suffix(),
                union_name.as_type()
            ))?;
            {
                let mut w = w.new_block();
                w.write_line(format!(
                    "return {} {{ .tag = .{} }};",
                    union_name.as_type(),
                    name.as_var(),
                ))?;
            }
            w.write_line("}")?.eob()?;
        } else {
            // !member_is_void
            // new_*
            w.write_line(format!(
                "fn new{}(val: {}) {} {{",
                name.as_fn_suffix(),
                member.type_.as_lang(),
                union_name.as_type()
            ))?;
            {
                let mut w = w.new_block();
                w.write_line(format!(
                    "return {}{{ .tag = .{}, .member = .{{ .{} = val }} }};",
                    union_name.as_type(),
                    name.as_var(),
                    name.as_var()
                ))?;
            }
            w.write_line("}")?.eob()?;

            // get_*
            w.write_line(format!(
                "pub fn {}(self: {}) {} {{",
                name.as_fn_suffix(),
                union_name.as_type(),
                member.type_.as_lang()
            ))?;
            {
                let mut w = w.new_block();
                w.write_line(format!("std.debug.assert(self.tag == .{});", name.as_var()))?;
                w.write_line(format!("return self.member.{};", member.name.as_var()))?;
            }
            w.write_line("}")?.eob()?;

            // set_*
            w.write_line(format!(
                "pub fn set{}(self: *{}, val: {}) void {{",
                name.as_fn_suffix(),
                union_name.as_type(),
                member.type_.as_lang()
            ))?;
            {
                let mut w = w.new_block();
                w.write_line(format!("std.debug.assert(self.tag == .{});", name.as_var()))?;
                w.write_line(format!("self.member.{} = val;", member.name.as_var()))?;
            }
            w.write_line("}")?.eob()?;
        }

        // is_*
        w.write_line(format!(
            "fn is{}(self: {}) bool {{",
            name.as_fn_suffix(),
            union_name.as_type(),
        ))?;
        {
            let mut w = w.new_block();
            w.write_line(format!("return self.tag == .{};", name.as_var()))?;
        }
        w.write_line("}")?.eob()?;

        Ok(())
    }

    fn define_union_member<T: Write>(
        w: &mut PrettyWriter<T>,
        union_name: &str,
        i: usize,
        member: &ASUnionMember,
    ) -> Result<(), Error> {
        Self::define_union_member_accessors(w, union_name, i, member)?;
        Ok(())
    }

    pub fn define_as_union<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        union_: &ASUnion,
    ) -> Result<(), Error> {
        w.write_line(format!("variant {} {{", name.as_type()))?;
        {
            let mut w = w.new_block();
            for (_i, member) in union_.members.iter().enumerate() {
                let member_is_void = matches!(member.type_.as_ref(), ASType::Void);
                if !member_is_void {
                    w.write_line(format!(
                        "{}: {},",
                        member.name.as_var(),
                        member.type_.as_lang(),
                    ))?;
                }
            }
        }
        w.write_line("}")?;
        w.eob()?;

        Ok(())
    }
}
