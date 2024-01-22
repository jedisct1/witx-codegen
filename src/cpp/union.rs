use super::*;
use std::io::Write;

impl CppGenerator {
    // fn define_union_member_accessors<T: Write>(
    //     w: &mut PrettyWriter<T>,
    //     union_name: &str,
    //     i: usize,
    //     member: &ASUnionMember,
    // ) -> Result<(), Error> {
        // let member_type = member.type_.as_ref();
        // match member_type {
        //     ASType::Void => {
        //         w.write_line(format!(
        //             "static {}(): {} {{",
        //             member.name.as_fn(),
        //             union_name.as_type()
        //         ))?
        //         .indent()?
        //         .write_line(format!("return {}.new({});", union_name.as_type(), i))?
        //         .write_line("}")?
        //         .eob()?;

        //         w.write_line(format!("set{}(): void {{", member.name.as_fn_suffix()))?
        //             .indent()?
        //             .write_line(format!("this.tag = {};", i))?
        //             .write_line("}")?
        //             .eob()?;

        //         w.write_line(format!("is{}(): bool {{", member.name.as_fn_suffix()))?
        //             .indent()?
        //             .write_line(format!("return this.tag === {};", i))?
        //             .write_line("}")?;
        //     }
        //     _ => {
        //         w.write_line(format!(
        //             "static {}(val: {}): {} {{",
        //             member.name.as_fn(),
        //             member_type.as_lang(),
        //             union_name.as_type()
        //         ))?;
        //         w.new_block().write_line(format!(
        //             "return {}.new({}, val);",
        //             union_name.as_type(),
        //             i
        //         ))?;
        //         w.write_line("}")?.eob()?;

        //         w.write_line(format!(
        //             "set{}(val: {}): void {{",
        //             member.name.as_fn_suffix(),
        //             member_type.as_lang()
        //         ))?;
        //         {
        //             w.new_block()
        //                 .write_line(format!("this.tag = {};", i))?
        //                 .write_line("this.set(val);")?;
        //         }
        //         w.write_line("}")?.eob()?;

        //         w.write_line(format!("is{}(): bool {{", member.name.as_fn_suffix(),))?
        //             .indent()?
        //             .write_line(format!("return this.tag === {};", i))?
        //             .write_line("}")?
        //             .eob()?;

        //         if member_type.is_nullable() {
        //             w.write_line(format!(
        //                 "get{}(): {} | null {{",
        //                 member.name.as_fn_suffix(),
        //                 member_type.as_lang()
        //             ))?;
        //         } else {
        //             w.write_line(format!(
        //                 "get{}(): {} {{",
        //                 member.name.as_fn_suffix(),
        //                 member_type.as_lang()
        //             ))?;
        //         }
        //         {
        //             let mut w = w.new_block();
        //             if member_type.is_nullable() {
        //                 w.write_line(format!("if (this.tag !== {}) {{ return null; }}", i))?;
        //             }
        //             w.write_line(format!("return this.get<{}>();", member_type.as_lang()))?;
        //         }
        //         w.write_line("}")?;
        //     }
        // }
        // Ok(())
    // }

    fn define_union_member<T: Write>(
        w: &mut PrettyWriter<T>,
        // union_name: &str,
        i: usize,
        member: &ASUnionMember,
    ) -> Result<(), Error> {
        let member_type = member.type_.as_ref();
        match member_type {
            ASType::Void => {
                w.write_line(format!(
                    "// {}: (no associated content) if tag={}",
                    member.name.as_var(),
                    i
                ))?;
            }
            _ => {
                w.write_line(format!(
                    "{} {}; // if tag={}",
                    member_type.as_lang(),
                    member.name.as_var(),
                    i
                ))?;
            }
        }
        // Self::define_union_member_accessors(w, union_name, i, member)?;
        Ok(())
    }

    pub fn define_as_union<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        union_: &ASUnion,
    ) -> Result<(), Error> {
        let tag_repr = union_.tag_repr.as_ref();
        let inner_name = format!("{}_member", name);
        w.write_line(format!("union {} {{", inner_name.as_type()))?;
        for (i, member) in union_.members.iter().enumerate() {
            // w.eob()?;
            Self::define_union_member(&mut w.new_block(), /* name, */ i, member)?;
        }
        w.write_line("};")?.eob()?;

        w.write_line(format!(
            "struct __attribute__((packed)) {} {{",
            name.as_type()
        ))?;
        {
            let mut w = w.new_block();
            w.write_line(format!("{} tag;", tag_repr.as_lang()))?;
            let pad_len = union_.padding_after_tag;
            for i in 0..(pad_len & 1) {
                w.write_line(format!("uint8_t __pad8_{};", i))?;
            }
            for i in 0..(pad_len & 3) / 2 {
                w.write_line(format!("uint16_t __pad16_{};", i))?;
            }
            for i in 0..(pad_len & 7) / 4 {
                w.write_line(format!("uint32_t __pad32_{};", i))?;
            }
            for i in 0..pad_len / 8 {
                w.write_line(format!("uint64_t __pad64_{};", i))?;
            }

            w.write_line(format!(
                "{} member;",
                inner_name.as_type()
            ))?;
        }

        w.write_line("};")?.eob()?;
        Ok(())
    }
}
