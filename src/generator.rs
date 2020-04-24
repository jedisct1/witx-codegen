use crate::astype::*;
use crate::error::*;
use crate::pretty_writer::PrettyWriter;
use std;
use std::io::Write;
use std::path::Path;

pub struct Generator<W: Write> {
    w: PrettyWriter<W>,
}

impl<W: Write> Generator<W> {
    pub fn new(writer: W) -> Self {
        let w = PrettyWriter::new(writer, "    ");
        Generator { w }
    }

    pub fn generate<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let document = witx::load(&[path])?;
        self.header()?;
        for type_ in document.typenames() {
            self.define_type(type_.as_ref())?;
        }
        for module in document.modules() {
            self.define_module(module.as_ref())?;
        }
        Ok(())
    }

    fn header(&mut self) -> Result<(), Error> {
        let w0 = &mut self.w;
        w0.write_line("type handle = i32;")?
            .write_line("type char = u8;")?
            .write_line("type ptr<T> = usize;")?
            .write_line("type mut_ptr<T> = usize;")?
            .write_line("type untyped_ptr = usize;")?
            .write_line("type union_member = usize;")?
            .write_line("type struct = usize;")?
            .write_line("type wasi_string = ptr<char>;")?
            .eob()?;
        Ok(())
    }

    fn define_as_alias<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        other_type: &ASType,
    ) -> Result<(), Error> {
        w.write_line(format!("export type {} = {};", as_type, other_type))?;
        Ok(())
    }

    fn define_as_enum<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        enum_data_type: &witx::EnumDatatype,
    ) -> Result<(), Error> {
        let actual_as_type = ASType::from(enum_data_type.repr);
        w.write_line(format!("export namespace {} {{", as_type))?;
        {
            let mut w = w.new_block();
            for (i, variant) in enum_data_type.variants.iter().enumerate() {
                Self::write_docs(&mut w, &variant.docs)?;
                w.write_line("// @ts-ignore: decorator")?
                    .write_line("@inline")?
                    .write_line(format!(
                        "export const {}: {} = {};",
                        variant.name.as_str().to_uppercase(),
                        as_type,
                        i
                    ))?
                    .eob()?;
            }
        }
        w.write_line("}")?
            .write_line(format!("export type {} = {};", as_type, actual_as_type))?
            .eob()?;
        Ok(())
    }

    fn define_as_handle<T: Write>(w: &mut PrettyWriter<T>, as_type: &ASType) -> Result<(), Error> {
        w.write_line(format!("export type {} = {};", as_type, ASType::Handle))?;
        Ok(())
    }

    fn define_as_int<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        int: &witx::IntDatatype,
    ) -> Result<(), Error> {
        let actual_as_type = ASType::from(int);
        w.write_line(format!("export namespace {} {{", as_type))?;
        {
            let mut w = w.new_block();
            for (i, variant) in int.consts.iter().enumerate() {
                Self::write_docs(&mut w, &variant.docs)?;
                w.write_line("// @ts-ignore: decorator")?
                    .write_line("@inline")?
                    .write_line(format!(
                        "export const {}: {} = {};",
                        variant.name.as_str().to_uppercase(),
                        as_type,
                        i
                    ))?;
            }
        }
        w.write_line("}")?
            .write_line(format!("export type {} = {};", as_type, actual_as_type))?
            .eob()?;
        Ok(())
    }

    fn define_as_flags<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        flags: &witx::FlagsDatatype,
    ) -> Result<(), Error> {
        let actual_as_type = ASType::from(flags);
        w.write_line(format!("export namespace {} {{", as_type))?;
        {
            let mut w = w.new_block();
            for (i, variant) in flags.flags.iter().enumerate() {
                Self::write_docs(&mut w, &variant.docs)?;
                w.write_line("// @ts-ignore: decorator")?
                    .write_line("@inline")?
                    .write_line(format!(
                        "export const {}: {} = {};",
                        variant.name.as_str().to_uppercase(),
                        as_type,
                        1u64 << i
                    ))?;
            }
        }
        w.write_line("}")?
            .write_line(format!("export type {} = {};", as_type, actual_as_type))?
            .eob()?;
        Ok(())
    }

    fn define_union_variant_accessors<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        i: usize,
        variant: &witx::UnionVariant,
    ) -> Result<(), Error> {
        let variant_name = variant.name.as_str();
        match variant.tref.as_ref() {
            None => {
                w.write_line("// @ts-ignore: decorator")?
                    .write_line("@inline")?
                    .write_line(format!("new_{}(): {} {{", variant_name, as_type))?
                    .indent()?
                    .write_line(format!("return new {}({});", as_type, i))?
                    .write_line("}")?
                    .eob()?
                    .write_line("// @ts-ignore: decorator")?
                    .write_line("@inline")?
                    .write_line(format!("set_{}(): void {{", variant_name))?
                    .indent()?
                    .write_line(format!("this.tag = {};", i))?
                    .write_line("}")?
                    .eob()?
                    .write_line("// @ts-ignore: decorator")?
                    .write_line("@inline")?
                    .write_line(format!("is_{}(): bool {{", variant_name))?
                    .indent()?
                    .write_line(format!("return this.tag === {};", i))?
                    .write_line("}")?;
            }
            Some(witx::TypeRef::Name(variant_type)) => {
                let as_variant_type = ASType::from(variant_type.as_ref());
                w.write_line("// @ts-ignore: decorator")?
                    .write_line("@inline")?
                    .write_line(format!(
                        "new_{}(val: {}): {} {{",
                        variant_name, as_variant_type, as_type
                    ))?;
                {
                    w.new_block()
                        .write_line(format!("let u = new {}({});", as_type, i))?
                        .write_line(format!("u.{} = val;", variant_name))?
                        .write_line("return u;")?;
                }
                w.write_line("}")?.eob()?;

                w.write_line("// @ts-ignore: decorator")?
                    .write_line("@inline")?
                    .write_line(format!(
                        "set_{}(val: {}): void {{",
                        variant_name, as_variant_type
                    ))?;
                {
                    w.new_block()
                        .write_line(format!("this.tag = {};", i))?
                        .write_line(format!("this.{} = val;", variant_name))?;
                }
                w.write_line("}")?.eob()?;

                w.write_line("// @ts-ignore: decorator")?
                    .write_line("@inline")?
                    .write_line(format!(
                        "get_{}(): {} | null {{",
                        variant_name, as_variant_type
                    ))?;

                {
                    let mut w = w.new_block();
                    w.write_line(format!("if (this.tag !== {}) {{", i))?;
                    {
                        w.new_block().write_line("return null;")?;
                    }
                    w.write_line("}")?
                        .write_line(format!("return this.{};", variant_name))?;
                }
                w.write_line("}")?;
            }
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn define_union_variant<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        i: usize,
        variant: &witx::UnionVariant,
    ) -> Result<(), Error> {
        let variant_name = variant.name.as_str();
        match variant.tref.as_ref() {
            None => {
                w.write_line(format!("{}: void; // if tag={}", variant_name, i))?;
            }
            Some(witx::TypeRef::Name(another_type)) => {
                w.write_line(format!(
                    "{}: {}; // if tag={}",
                    variant_name,
                    ASType::from(another_type.as_ref()),
                    i
                ))?;
            }
            Some(witx::TypeRef::Value(witx_type)) => match witx_type.as_ref() {
                witx::Type::Enum(enum_data_type) => {
                    let as_type = ASType::from(enum_data_type);
                    w.write_line(format!("{}: {}; // if tag={}", variant_name, as_type, i))?;
                }
                _ => unimplemented!(),
            },
        }
        w.eob()?;
        Self::define_union_variant_accessors(w, as_type, i, variant)?;
        Ok(())
    }

    fn define_as_union<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        union: &witx::UnionDatatype,
    ) -> Result<(), Error> {
        let as_tag = ASType::from(union.tag.as_ref());
        let variants = &union.variants;
        w.write_line("// @ts-ignore: decorator")?
            .write_line("@unmanaged")?
            .write_line(format!("export class {} {{", as_type))?;
        {
            let mut w = w.new_block();
            w.write_line(format!("tag: {};", as_tag))?
                .eob()?
                .write_line("// @ts-ignore: decorator")?
                .write_line("@inline")?
                .write_line(format!("constructor(tag: {}) {{", as_tag))?;
            {
                w.new_block().write_line("this.tag = tag;")?;
            }
            w.write_line("}")?;
            for (i, variant) in variants.iter().enumerate() {
                w.eob()?;
                Self::define_union_variant(&mut w, as_type, i, variant)?;
            }
        }
        w.write_line("}")?;

        Ok(())
    }

    fn define_as_builtin<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        actual_as_type: &ASType,
    ) -> Result<(), Error> {
        w.write_line(format!("export type {} = {};", as_type, actual_as_type))?;
        Ok(())
    }

    fn define_as_witx_type<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        witx_type: &witx::Type,
    ) -> Result<(), Error> {
        match witx_type {
            witx::Type::Enum(enum_data_type) => Self::define_as_enum(w, as_type, enum_data_type)?,
            witx::Type::Handle(_handle) => Self::define_as_handle(w, as_type)?,
            witx::Type::Int(int) => Self::define_as_int(w, as_type, int)?,
            witx::Type::Flags(flags) => Self::define_as_flags(w, as_type, flags)?,
            witx::Type::Builtin(builtin) => Self::define_as_builtin(w, as_type, &builtin.into())?,
            witx::Type::Union(union) => Self::define_as_union(w, as_type, union)?,
            e => {
                dbg!(e);
                unimplemented!();
            }
        };
        Ok(())
    }

    fn define_type(&mut self, type_: &witx::NamedType) -> Result<(), Error> {
        let w0 = &mut self.w;
        let as_type = ASType::Alias(type_.name.as_str().to_string());
        let docs = &type_.docs;
        if docs.is_empty() {
            w0.write_line(format!("/** {} */", as_type))?;
        } else {
            Self::write_docs(w0, &type_.docs)?;
        }
        let tref = &type_.tref;
        match tref {
            witx::TypeRef::Name(other_type) => {
                Self::define_as_alias(w0, &as_type, &other_type.as_ref().into())?
            }
            witx::TypeRef::Value(witx_type) => {
                Self::define_as_witx_type(w0, &as_type, &witx_type.as_ref())?
            }
        };
        w0.eob()?;
        Ok(())
    }

    fn define_module(&mut self, module: &witx::Module) -> Result<(), Error> {
        let w = &mut self.w.clone();
        w.eob()?.write_line(format!(
            "// ----------------------[{}]----------------------",
            module.name.as_str()
        ))?;
        for func in module.funcs() {
            self.define_func(module.name.as_str(), func.as_ref())?;
            w.eob()?;
        }
        Ok(())
    }

    fn define_func(&mut self, module_name: &str, func: &witx::InterfaceFunc) -> Result<(), Error> {
        let w0 = &mut self.w;
        let docs = &func.docs;
        let name = func.name.as_str();
        if docs.is_empty() {
            w0.write_line(format!("\n/** {} */", name))?;
        } else {
            Self::write_docs(w0, docs)?;
        }
        let s_in: Vec<_> = func
            .params
            .iter()
            .map(|param| param.name.as_str().to_string())
            .collect();
        let s_out: Vec<_> = func
            .results
            .iter()
            .map(|param| param.name.as_str().to_string())
            .collect();
        w0.write_line("/**")?
            .write_line(format!(" * in:  {}", s_in.join(", ")))?
            .write_line(format!(" * out: {}", s_out.join(", ")))?
            .write_line(" */")?;
        w0.write_line("// @ts-ignore: decorator")?
            .write_line(format!("@external(\"{}\", \"{}\")", module_name, name))?
            .write_line(format!("export declare function {}(", name))?;

        let params = &func.params;
        let as_params = Self::params_to_as(params);
        let results = &func.results;
        let as_results = Self::params_to_as(results);
        let return_value = as_results.get(0);
        let as_results = if as_results.is_empty() {
            &[]
        } else {
            &as_results[1..]
        };
        let as_params: Vec<_> = as_params
            .iter()
            .map(|(v, t)| format!("{}: {}", v, t))
            .collect();
        let as_results: Vec<_> = as_results
            .iter()
            .map(|(v, t)| format!("{}_ptr: {}", v, ASType::MutPtr(Box::new(t.clone()))))
            .collect();
        if !as_params.is_empty() {
            if !as_results.is_empty() {
                w0.continuation()?
                    .write(as_params.join(", "))?
                    .write(",")?
                    .eol()?;
            } else {
                w0.continuation()?.write_line(as_params.join(", "))?;
            }
        }
        let return_as_type_and_comment = match return_value {
            None => (ASType::Void, "".to_string()),
            Some(x) => (x.1.clone(), format!(" /* {} */", x.0)),
        };
        if !as_results.is_empty() {
            w0.continuation()?.write_line(as_results.join(", "))?;
        }
        w0.write_line(format!(
            "): {}{};",
            return_as_type_and_comment.0, return_as_type_and_comment.1
        ))?;
        Ok(())
    }

    fn write_docs<T: Write>(w: &mut PrettyWriter<T>, docs: &str) -> Result<(), Error> {
        if docs.is_empty() {
            return Ok(());
        }
        w.write_line("/**")?;
        for docs_line in docs.lines() {
            w.write_line(format!(" * {}", docs_line))?;
        }
        w.write_line(" */")?;
        Ok(())
    }

    fn params_to_as(params: &[witx::InterfaceFuncParam]) -> Vec<(String, ASType)> {
        let mut as_params = vec![];
        for param in params {
            let leaf_type = Self::leaf_type(&param.tref);
            let as_leaf_type = ASType::from(leaf_type);
            let (first, second) = as_leaf_type.decompose();
            as_params.push((format!("{}{}", param.name.as_str(), first.1), first.0));
            if let Some(second) = second {
                as_params.push((format!("{}{}", param.name.as_str(), second.1), second.0))
            }
        }
        as_params
    }

    fn leaf_type(type_ref: &witx::TypeRef) -> &witx::Type {
        match type_ref {
            witx::TypeRef::Name(other_type) => {
                let x = other_type.as_ref();
                Self::leaf_type(&x.tref)
            }
            witx::TypeRef::Value(type_) => type_.as_ref(),
        }
    }
}
