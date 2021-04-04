use super::*;
use std::io::Write;

impl AssemblyScriptGenerator {
    fn define_union_member_accessors<T: Write>(
        w: &mut PrettyWriter<T>,
        union_name: &str,
        i: usize,
        member: &ASUnionMember,
    ) -> Result<(), Error> {
        let member_type = member.type_.as_ref();
        match member_type {
            ASType::Void => {
                w.write_line(format!(
                    "static {}(): {} {{",
                    member.name.as_fn(),
                    union_name.as_type()
                ))?
                .indent()?
                .write_line(format!("return {}.new({});", union_name.as_type(), i))?
                .write_line("}")?
                .eob()?;

                w.write_line(format!("set{}(): void {{", member.name.as_fn_suffix()))?
                    .indent()?
                    .write_line(format!("this.tag = {};", i))?
                    .write_line("}")?
                    .eob()?;

                w.write_line(format!("is{}(): bool {{", member.name.as_fn_suffix()))?
                    .indent()?
                    .write_line(format!("return this.tag === {};", i))?
                    .write_line("}")?;
            }
            _ => {
                w.write_line(format!(
                    "static {}(val: {}): {} {{",
                    member.name.as_fn(),
                    member_type.as_lang(),
                    union_name.as_type()
                ))?;
                w.new_block().write_line(format!(
                    "return {}.new({}, val);",
                    union_name.as_type(),
                    i
                ))?;
                w.write_line("}")?.eob()?;

                w.write_line(format!(
                    "set{}(val: {}): void {{",
                    member.name.as_fn_suffix(),
                    member_type.as_lang()
                ))?;
                {
                    w.new_block()
                        .write_line(format!("this.tag = {};", i))?
                        .write_line("this.set(val);")?;
                }
                w.write_line("}")?.eob()?;

                w.write_line(format!("is{}(): bool {{", member.name.as_fn_suffix(),))?
                    .indent()?
                    .write_line(format!("return this.tag === {};", i))?
                    .write_line("}")?
                    .eob()?;

                if member_type.is_nullable() {
                    w.write_line(format!(
                        "get{}(): {} | null {{",
                        member.name.as_fn_suffix(),
                        member_type.as_lang()
                    ))?;
                } else {
                    w.write_line(format!(
                        "get{}(): {} {{",
                        member.name.as_fn_suffix(),
                        member_type.as_lang()
                    ))?;
                }
                {
                    let mut w = w.new_block();
                    if member_type.is_nullable() {
                        w.write_line(format!("if (this.tag !== {}) {{ return null; }}", i))?;
                    }
                    w.write_line(format!("return this.get<{}>();", member_type.as_lang()))?;
                }
                w.write_line("}")?;
            }
        }
        Ok(())
    }

    fn define_union_member<T: Write>(
        w: &mut PrettyWriter<T>,
        union_name: &str,
        i: usize,
        member: &ASUnionMember,
    ) -> Result<(), Error> {
        let member_type = member.type_.as_ref();
        match member_type {
            ASType::Void => {
                w.write_line(format!(
                    "// --- {}: (no associated content) if tag={}",
                    member.name.as_var(),
                    i
                ))?;
            }
            _ => {
                w.write_line(format!(
                    "// --- {}: {} if tag={}",
                    member.name.as_var(),
                    member_type.as_lang(),
                    i
                ))?;
            }
        }
        w.eob()?;
        Self::define_union_member_accessors(w, union_name, i, member)?;
        Ok(())
    }

    pub fn define_as_union<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        union_: &ASUnion,
    ) -> Result<(), Error> {
        let tag_repr = union_.tag_repr.as_ref();
        w.write_line("// @ts-ignore: decorator")?
            .write_line("@unmanaged")?
            .write_line(format!("export class {} {{", name.as_type()))?;
        {
            let mut w = w.new_block();
            w.write_line(format!("tag: {};", tag_repr.as_lang()))?;
            let pad_len = union_.padding_after_tag;
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
            w.eob()?;

            w.write_line(format!("constructor(tag: {}) {{", tag_repr.as_lang()))?;
            {
                let mut w = w.new_block();
                w.write_line("this.tag = tag;")?.write_line(format!(
                    "memory.fill(changetype<usize>(this) + {}, 0, {});",
                    union_.member_offset, union_.max_member_size
                ))?;
            }
            w.write_line("}")?.eob()?;

            w.write_line("// @ts-ignore: default")?.write_line(format!(
                "static new<T>(tag: u8, val: T = 0): {} {{",
                name.as_type()
            ))?;
            {
                let mut w = w.new_block();
                w.write_line(format!("let tu = new {}(tag);", name.as_type()))?
                    .write_line("tu.set(val);")?
                    .write_line("return tu;")?;
            }
            w.write_line("}")?.eob()?;

            w.write_line("get<T>(): T {")?;
            {
                let mut w = w.new_block();
                w.write_line("// @ts-ignore: cast")?
                    .write_line(format!(
                        "let valBuf = changetype<usize>(this) + {};",
                        union_.member_offset
                    ))?
                    .write_line("if (isReference<T>()) {")?;
                w.new_block().write_line("return changetype<T>(valBuf);")?;
                w.write_line("} else {")?;
                w.new_block().write_line("return load<T>(valBuf);")?;
                w.write_line("}")?;
            }
            w.write_line("}")?.eob()?;

            w.write_line("// @ts-ignore: default")?
                .write_line("set<T>(val: T = 0): void {")?;
            {
                let mut w = w.new_block();
                w.write_line("// @ts-ignore: cast")?
                    .write_line(format!(
                        "let valBuf = changetype<usize>(this) + {};",
                        union_.member_offset
                    ))?
                    .write_line(format!(
                        "memory.fill(valBuf, 0, {});",
                        union_.max_member_size
                    ))?
                    .write_line("if (isReference<T>()) {")?;
                w.new_block().write_line(
                    "(val !== null) && memory.copy(valBuf, changetype<usize>(val), offsetof<T>());",
                )?;
                w.write_line("} else {")?;
                w.new_block().write_line("store<T>(valBuf, val)")?;
                w.write_line("}")?;
            }
            w.write_line("}")?;

            for (i, member) in union_.members.iter().enumerate() {
                w.eob()?;
                Self::define_union_member(&mut w, name, i, member)?;
            }
        }
        w.write_line("}")?.eob()?;
        Ok(())
    }
}
