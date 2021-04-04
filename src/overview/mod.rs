mod common;
mod function;
mod r#struct;
mod tuple;
mod union;

use super::*;
use crate::astype::*;
use crate::error::*;
use crate::pretty_writer::PrettyWriter;
use common::*;
use std::io::Write;

pub struct OverviewGenerator {
    module_name: Option<String>,
}

impl OverviewGenerator {
    pub fn new(module_name: Option<String>) -> Self {
        OverviewGenerator { module_name }
    }
}

impl<T: Write> Generator<T> for OverviewGenerator {
    fn generate(
        &self,
        writer: &mut T,
        module_witx: witx::Module,
        options: &Options,
    ) -> Result<(), Error> {
        let mut w = PrettyWriter::new(writer, "    ");
        let module_name = match &self.module_name {
            None => module_witx.name().as_str().to_string(),
            Some(module_name) => module_name.to_string(),
        };
        let module_id = module_witx.module_id();
        let skip_imports = options.skip_imports;

        if !options.skip_header {
            Self::header(&mut w)?;
        }

        let module_title_doc = format!(
            "---------------------- Module: [{}] ----------------------",
            module_name
        );
        w.eob()?;
        w.write_line(format!("{}", module_title_doc))?;
        w.eob()?;

        for type_ in module_witx.typenames() {
            if skip_imports && &type_.module != module_id {
                continue;
            }
            let constants_for_type: Vec<_> = module_witx
                .constants()
                .into_iter()
                .filter_map(|x| {
                    if x.ty == type_.name {
                        Some(ASConstant {
                            name: x.name.as_str().to_string(),
                            value: x.value,
                        })
                    } else {
                        None
                    }
                })
                .collect();
            Self::define_type(&mut w, type_.as_ref(), &constants_for_type)?;
        }

        for func in module_witx.funcs() {
            Self::define_func(&mut w, &module_name, func.as_ref())?;
        }

        Ok(())
    }
}

impl OverviewGenerator {
    fn header<T: Write>(w: &mut PrettyWriter<T>) -> Result<(), Error> {
        w.write_line("API overview")?;
        w.eob()?;
        Ok(())
    }

    fn define_as_alias<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        other_type: &ASType,
    ) -> Result<(), Error> {
        w.write_line(format!(
            "alias {} = {}",
            name.as_type(),
            other_type.as_lang()
        ))?;
        Ok(())
    }

    fn define_as_atom<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        type_: &ASType,
    ) -> Result<(), Error> {
        w.write_line(format!("[alias] {} = {}", name.as_type(), type_.as_lang()))?;
        Ok(())
    }

    fn define_as_enum<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        enum_: &ASEnum,
    ) -> Result<(), Error> {
        let repr = enum_.repr.as_ref();
        w.write_line(format!(
            "enum {}: (tag: {})",
            name.as_type(),
            repr.as_lang()
        ))?;
        {
            let mut w = w.new_block();
            for choice in &enum_.choices {
                w.write_line(format!("- {}: {}", choice.name.as_const(), name.as_type(),))?;
            }
        }
        Ok(())
    }

    fn define_as_constants<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        constants: &ASConstants,
    ) -> Result<(), Error> {
        let repr = constants.repr.as_ref();
        w.write_line(format!(
            "constants {}: (type: {})",
            name.as_type(),
            repr.as_lang()
        ))?;
        Self::define_constants_for_type(w, name, &constants.constants)?;
        Ok(())
    }

    fn define_as_type<T: Write>(
        w: &mut PrettyWriter<T>,
        name: &str,
        type_: &ASType,
    ) -> Result<(), Error> {
        match type_ {
            ASType::Alias(_)
            | ASType::Bool
            | ASType::Char8
            | ASType::Char32
            | ASType::F32
            | ASType::F64
            | ASType::U8
            | ASType::U16
            | ASType::U32
            | ASType::U64
            | ASType::S8
            | ASType::S16
            | ASType::S32
            | ASType::S64
            | ASType::USize
            | ASType::Handle(_) => Self::define_as_atom(w, name, type_)?,
            ASType::Enum(enum_) => Self::define_as_enum(w, name, enum_)?,
            ASType::Union(union_) => Self::define_as_union(w, name, union_)?,
            ASType::Constants(constants) => Self::define_as_constants(w, name, constants)?,
            ASType::Tuple(members) => Self::define_as_tuple(w, name, members)?,
            ASType::Struct(members) => Self::define_as_struct(w, name, members)?,
            _ => {
                dbg!(type_);
                unimplemented!();
            }
        }
        Ok(())
    }

    fn define_constants_for_type<T: Write>(
        w: &mut PrettyWriter<T>,
        type_name: &str,
        constants: &[ASConstant],
    ) -> Result<(), Error> {
        if constants.is_empty() {
            return Ok(());
        }
        w.write_line(format!("predefined constants for {}:", type_name))?;
        {
            let mut w = w.new_block();
            let mut hex = false;
            let mut single_bits: usize = 0;
            for constant in constants {
                if constant.value > 0xffff {
                    hex = true;
                }
                if constant.value.count_ones() == 1 {
                    single_bits += 1;
                }
            }
            if constants.len() > 2 && single_bits == constants.len() {
                hex = true;
            }
            for constant in constants {
                let value_s = if hex {
                    format!("0x{:x}", constant.value)
                } else {
                    format!("{}", constant.value)
                };
                w.write_line(format!("- {} = {}", constant.name.as_const(), value_s))?;
            }
        }
        Ok(())
    }

    fn define_type<T: Write>(
        w: &mut PrettyWriter<T>,
        type_witx: &witx::NamedType,
        constants: &[ASConstant],
    ) -> Result<(), Error> {
        let type_name = type_witx.name.as_str().clone();
        let tref = &type_witx.tref;
        match tref {
            witx::TypeRef::Name(other_type) => {
                Self::define_as_alias(w, type_name, &ASType::from(&other_type.tref))?
            }
            witx::TypeRef::Value(type_witx) => {
                let t = ASType::from(type_witx.as_ref());
                Self::define_as_type(w, type_name, &t)?
            }
        }
        w.eob()?;
        Self::define_constants_for_type(w, type_name, constants)?;
        Ok(())
    }
}
