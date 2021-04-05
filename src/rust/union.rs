use super::*;
use std::io::Write;

impl RustGenerator {
    fn define_union_member_accessors<T: Write>(
        w: &mut PrettyWriter<T>,
        _union_name: &str,
        i: usize,
        member: &ASUnionMember,
        inner_name: &str,
    ) -> Result<(), Error> {
        let name = &member.name;
        let member_is_void = matches!(member.type_.as_ref(), ASType::Void);

        if member_is_void {
            // new_*
            w.write_line(format!("pub fn new_{}() -> Self {{", name.as_fn_suffix(),))?;
            {
                let mut w = w.new_block();
                w.write_line(format!("Self::new({})", i))?;
            }
            w.write_line("}")?.eob()?;
        } else {
            // !member_is_void
            // new_*
            w.write_line(format!(
                "pub fn new_{}(val: {}) -> Self {{",
                name.as_fn_suffix(),
                member.type_.as_lang()
            ))?;
            {
                let mut w = w.new_block();
                w.write_line(format!("let mut tu = Self::new({});", i))?;
                w.write_line(format!(
                    "tu.member = std::mem::MaybeUninit::new({} {{ {}: val }});",
                    inner_name.as_type(),
                    member.name.as_var()
                ))?;
                w.write_line("tu")?;
            }
            w.write_line("}")?.eob()?;

            // get_*
            w.write_line(format!(
                "pub fn get_{}(&self) -> {} {{",
                name.as_fn_suffix(),
                member.type_.as_lang()
            ))?;
            {
                let mut w = w.new_block();
                w.write_line(format!("unsafe {{ assert_eq!(self.tag, {}) }};", i))?;
                w.write_line(format!(
                    "unsafe {{ self.member.assume_init().{} }}",
                    member.name.as_var()
                ))?;
            }
            w.write_line("}")?.eob()?;

            // set_*
            w.write_line(format!(
                "pub fn set_{}(&mut self, val: {}) {{",
                name.as_fn_suffix(),
                member.type_.as_lang()
            ))?;
            {
                let mut w = w.new_block();
                w.write_line(format!("unsafe {{ assert_eq!(self.tag, {}) }};", i))?;
                w.write_line(format!(
                    "let uval = TestTaggedUnionMember {{ {}: val }};",
                    member.name.as_var()
                ))?;
                w.write_line("unsafe { *self.member.as_mut_ptr() = uval };")?;
            }
            w.write_line("}")?.eob()?;
        }

        // is_*
        w.write_line(format!(
            "pub fn is_{}(&self) -> bool {{",
            name.as_fn_suffix()
        ))?;
        {
            let mut w = w.new_block();
            w.write_line(format!("self.tag == {}", i))?;
        }
        w.write_line("}")?.eob()?;

        Ok(())
    }

    fn define_union_member<T: Write>(
        w: &mut PrettyWriter<T>,
        union_name: &str,
        i: usize,
        member: &ASUnionMember,
        inner_name: &str,
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
        Self::define_union_member_accessors(w, union_name, i, member, inner_name)?;
        Ok(())
    }

    pub fn define_as_union<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        union_: &ASUnion,
    ) -> Result<(), Error> {
        let tag_repr = union_.tag_repr.as_ref();
        let inner_name = format!("{}_member", name);
        w.write_line("#[repr(C)]")?
            .write_line("#[derive(Copy, Clone)]")?
            .write_line(format!("pub union {} {{", inner_name.as_type()))?;
        {
            let mut w = w.new_block();
            for (i, member) in union_.members.iter().enumerate() {
                let member_is_void = matches!(member.type_.as_ref(), ASType::Void);
                if member_is_void {
                    w.write_line(format!(
                        "// {} with no associated value if tag={}",
                        member.name.as_var(),
                        i
                    ))?;
                } else {
                    w.write_line(format!(
                        "{}: {}, // if tag={}",
                        member.name.as_var(),
                        member.type_.as_lang(),
                        i
                    ))?;
                }
            }
        }
        w.write_line("}")?;
        w.eob()?;

        w.write_line("#[repr(C, packed)]")?
            .write_line("#[derive(Copy, Clone)]")?
            .write_line(format!("pub struct {} {{", name.as_type()))?;
        {
            let mut w = w.new_block();
            w.write_line(format!("pub tag: {},", tag_repr.as_lang()))?;
            let pad_len = union_.padding_after_tag;
            for i in 0..(pad_len & 1) {
                w.write_line(format!("__pad8_{}: u8,", i))?;
            }
            for i in 0..(pad_len & 3) / 2 {
                w.write_line(format!("__pad16_{}: u16,", i))?;
            }
            for i in 0..(pad_len & 7) / 4 {
                w.write_line(format!("__pad32_{}: u32,", i))?;
            }
            for i in 0..pad_len / 8 {
                w.write_line(format!("__pad64_{}: u64,", i))?;
            }
            w.write_line(format!(
                "pub member: std::mem::MaybeUninit<{}>,",
                inner_name.as_type()
            ))?;
        }
        w.write_line("}")?;
        w.eob()?;

        w.write_line(format!("impl {} {{", name.as_type()))?;
        {
            let mut w = w.new_block();
            w.write_line(format!("fn new(tag: {}) -> Self {{", tag_repr.as_lang()))?;
            {
                let mut w = w.new_block();
                w.write_line("let mut tu = unsafe { std::mem::zeroed::<Self>() };")?;
                w.write_line("tu.tag = tag;")?;
                w.write_line("tu")?;
            }
            w.write_line("}")?.eob()?;

            for (i, member) in union_.members.iter().enumerate() {
                w.eob()?;
                Self::define_union_member(&mut w, name, i, member, &inner_name)?;
            }
        }
        w.write_line("}")?.eob()?;
        Ok(())
    }
}
