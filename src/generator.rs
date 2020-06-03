use crate::astype::*;
use crate::error::*;
use crate::pretty_writer::PrettyWriter;
use std::io::Write;
use std::path::Path;

pub struct Generator<W: Write> {
    w: PrettyWriter<W>,
    module_name: Option<String>,
}

impl<W: Write> Generator<W> {
    pub fn new(writer: W, module_name: Option<String>) -> Self {
        let w = PrettyWriter::new(writer, "    ");
        Generator { w, module_name }
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
        w0.write_lines("API overview")?.eob()?;
        Ok(())
    }

    fn define_as_alias<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        other_type: &ASType,
    ) -> Result<(), Error> {
        w.write_line(format!("{}: alias({})", as_type, other_type))?;
        Ok(())
    }

    fn define_as_enum<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        enum_data_type: &witx::EnumDatatype,
    ) -> Result<(), Error> {
        let actual_as_type = ASType::from(enum_data_type.repr);
        w.write_line(format!("{}: enum({})", as_type, actual_as_type))?;
        {
            let mut w = w.new_block();
            for (i, variant) in enum_data_type.variants.iter().enumerate() {
                Self::write_docs(&mut w, &variant.docs)?;
                w.write_line(format!("- {} = {}", variant.name.as_str(), i))?;
            }
        }
        Ok(())
    }

    fn define_as_handle<T: Write>(w: &mut PrettyWriter<T>, as_type: &ASType) -> Result<(), Error> {
        w.write_line(format!("{}: handle", as_type))?;
        Ok(())
    }

    fn define_as_int<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        int: &witx::IntDatatype,
    ) -> Result<(), Error> {
        let actual_as_type = ASType::from(int);
        w.write_line(format!("{}: int({})", as_type, actual_as_type))?;
        {
            let mut w = w.new_block();
            for (i, variant) in int.consts.iter().enumerate() {
                Self::write_docs(&mut w, &variant.docs)?;
                w.write_line(format!("- {} = {}", variant.name.as_str(), i))?;
            }
        }
        Ok(())
    }

    fn define_as_flags<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        flags: &witx::FlagsDatatype,
    ) -> Result<(), Error> {
        let actual_as_type = ASType::from(flags);
        w.write_line(format!("{}: flags({})", as_type, actual_as_type))?;
        {
            let mut w = w.new_block();
            for (i, variant) in flags.flags.iter().enumerate() {
                Self::write_docs(&mut w, &variant.docs)?;
                w.write_line(format!("- {} = 1 << {}", variant.name.as_str(), i))?;
            }
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
                w.write_line(format!("- {}: void (if tag={})", variant_name, i))?;
            }
            Some(variant_type) => {
                w.write_line(format!(
                    "- {}: {} (if tag={})",
                    variant_name,
                    ASType::from(variant_type),
                    i
                ))?;
            }
        }
        Ok(())
    }

    fn define_as_union<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        union: &witx::UnionDatatype,
    ) -> Result<(), Error> {
        let as_tag = ASType::from(union.tag.as_ref());
        let variants = &union.variants;
        let val_offset = union.union_layout().contents_offset;
        let val_size = union.union_layout().contents_size;
        let pad_len = val_offset + val_size;
        w.write_line(format!(
            "union {} (tag: {}, padding: {} bytes)",
            as_type, as_tag, pad_len
        ))?;
        {
            let mut w = w.new_block();
            for (i, variant) in variants.iter().enumerate() {
                Self::define_union_variant(&mut w, as_type, i, variant)?;
            }
        }
        Ok(())
    }

    fn define_as_builtin<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        actual_as_type: &ASType,
    ) -> Result<(), Error> {
        w.write_line(format!("alias {} = {}", as_type, actual_as_type))?;
        Ok(())
    }

    fn define_as_struct<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        witx_struct: &witx::StructDatatype,
    ) -> Result<(), Error> {
        let variants = &witx_struct.members;
        w.write_line(format!("{}: struct", as_type))?;
        {
            let mut w = w.new_block();
            for variant in variants {
                let variant_name = variant.name.as_str();
                let variant_type = ASType::from(&variant.tref);
                Self::write_docs(&mut w, &variant.docs)?;
                w.write_line(format!("- {}: {}", variant_name, variant_type))?;
            }
        }
        Ok(())
    }

    fn define_as_array<T: Write>(
        w: &mut PrettyWriter<T>,
        as_type: &ASType,
        actual_as_type: &ASType,
    ) -> Result<(), Error> {
        w.write_line(format!("{}: WasiArray<{}>;", as_type, actual_as_type))?;
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
            witx::Type::Struct(witx_struct) => Self::define_as_struct(w, as_type, witx_struct)?,
            witx::Type::Array(array) => Self::define_as_array(w, as_type, &ASType::from(array))?,
            witx::Type::ConstPointer(_) | witx::Type::Pointer(_) => {
                panic!("Typedef's pointers are not implemented")
            }
        };
        Ok(())
    }

    fn define_type(&mut self, type_: &witx::NamedType) -> Result<(), Error> {
        let w0 = &mut self.w;
        let as_type = ASType::Alias(type_.name.as_str().to_string());
        let docs = &type_.docs;
        if !docs.is_empty() {
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
        w.eob()?
            .write_line(format!(
                "----------------------[Module: {}]----------------------",
                module.name.as_str()
            ))?
            .eob()?;
        for func in module.funcs() {
            self.define_func(module.name.as_str(), func.as_ref())?;
            w.eob()?;
        }
        Ok(())
    }

    fn define_func(&mut self, module_name: &str, func: &witx::InterfaceFunc) -> Result<(), Error> {
        let module_name = match self.module_name.as_ref() {
            None => module_name,
            Some(module_name) => module_name.as_str(),
        };
        let w0 = &mut self.w;
        let docs = &func.docs;
        let name = func.name.as_str();
        if !docs.is_empty() {
            Self::write_docs(w0, docs)?;
        }

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
        let return_as_type_and_comment = match return_value {
            None => (ASType::Void, "".to_string()),
            Some(x) => (x.1.clone(), format!(" /* {} */", x.0)),
        };
        w0.write_line(format!(
            "function {}(): {}",
            name, return_as_type_and_comment.0
        ))?;
        let mut w0 = w0.new_block();

        let as_params: Vec<_> = as_params
            .iter()
            .map(|(v, t)| format!("- {}: {}", v, t))
            .collect();
        let as_results: Vec<_> = as_results
            .iter()
            .map(|(v, t)| format!("- {}: {}", v, ASType::MutPtr(Box::new(t.clone()))))
            .collect();
        if !as_params.is_empty() {
            w0.write_line("- Input:")?;
            w0.new_block().write_lines(as_params.join("\n"))?;
        }
        if !as_results.is_empty() {
            w0.write_line("- Output:")?;
            w0.new_block().write_lines(as_results.join("\n"))?;
        }
        Ok(())
    }

    fn write_docs<T: Write>(w: &mut PrettyWriter<T>, docs: &str) -> Result<(), Error> {
        return Ok(());
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
            let as_leaf_type = ASType::from(leaf_type).name(param.tref.type_name());
            let (first, second) = as_leaf_type.decompose();
            match &param.tref {
                witx::TypeRef::Name(name) => {
                    as_params.push((
                        format!("{}{}", param.name.as_str(), first.1),
                        ASType::from(name.as_ref()),
                    ));
                }
                _ => {
                    as_params.push((format!("{}{}", param.name.as_str(), first.1), first.0));
                }
            }
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
