mod pretty_writer;

use pretty_writer::PrettyWriter;
use std::io::Write;
use std::path::Path;
use std::{self, fmt};
use witx::WitxError;

#[derive(Debug)]
pub enum Error {
    Witx(WitxError),
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<WitxError> for Error {
    fn from(e: WitxError) -> Self {
        Self::Witx(e)
    }
}

struct Generator<W: Write> {
    w: PrettyWriter<W>,
}

impl<W: Write> Generator<W> {
    fn new(writer: W) -> Self {
        let w = PrettyWriter::new(writer, "    ");
        let generator = Generator { w };
        generator
    }

    fn generate<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
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
            .write_line("type ptr<T> = usize; // all pointers are usize'd")?
            .write_line("type mut_ptr<T> = usize; // all pointers are usize'd")?
            .write_line("type untyped_ptr = usize; // all pointers are usize'd")?
            .write_line("type union_member = usize; // all pointers are usize'd")?
            .write_line("type struct<T> = T;  // structs are references already in AS)")?
            .write_line("type wasi_string = ptr<char>;")?
            .eob()?;
        Ok(())
    }

    fn define_type(&mut self, type_: &witx::NamedType) -> Result<(), Error> {
        let w0 = &mut self.w;
        let docs = &type_.docs;
        let name = type_.name.as_str();
        if docs.is_empty() {
            w0.write_line(format!("/** {} */", name))?;
        } else {
            w0.write_line("/**")?;
            for docs_line in docs.lines() {
                w0.write_line(format!(" * {}", docs_line))?;
            }
            w0.write_line(" */")?;
        }
        let tref = &type_.tref;
        match tref {
            witx::TypeRef::Name(another_type) => {
                w0.write_line(format!(
                    "export type {} = {};",
                    name,
                    another_type.name.as_str()
                ))?;
            }
            witx::TypeRef::Value(type_def) => match type_def.as_ref() {
                witx::Type::Enum(enum_data_type) => {
                    let as_type = match enum_data_type.repr {
                        witx::IntRepr::U8 => "u8",
                        witx::IntRepr::U16 => "u16",
                        witx::IntRepr::U32 => "u32",
                        witx::IntRepr::U64 => "u64",
                    };
                    w0.write_line(format!("export namespace {} {{", name))?;
                    let mut w = w0.new_block();
                    for (i, variant) in enum_data_type.variants.iter().enumerate() {
                        let docs = &variant.docs;
                        if !docs.is_empty() {
                            if i > 0 {
                                w.eob()?;
                            }
                            w.write_line("/**")?;
                            for docs_line in docs.lines() {
                                w.write_line(format!(" * {}", docs_line))?;
                            }
                            w.write_line(" */")?;
                        }
                        w.write_line("// @ts-ignore: decorator")?
                            .write_line("@inline")?
                            .write_line(format!(
                                "export const {}: {} = {};",
                                variant.name.as_str().to_uppercase(),
                                name,
                                i
                            ))?;
                    }
                    w0.write_line("}")?
                        .write_line(format!("export type {} = {};", name, as_type))?
                        .eob()?;
                }
                witx::Type::Handle(_handle) => {
                    w0.write_line(format!("export type {} = handle;", name))?;
                }
                witx::Type::Int(int) => {
                    let as_type = match int.repr {
                        witx::IntRepr::U8 => "u8",
                        witx::IntRepr::U16 => "u16",
                        witx::IntRepr::U32 => "u32",
                        witx::IntRepr::U64 => "u64",
                    };
                    w0.write_line(format!("export namespace {} {{", name))?;
                    let mut w = w0.new_block();
                    for (i, variant) in int.consts.iter().enumerate() {
                        let docs = &variant.docs;
                        if !docs.is_empty() {
                            if i > 0 {
                                w.eob()?;
                            }
                            w.write_line("/**")?;
                            for docs_line in docs.lines() {
                                w.write_line(format!(" * {}", docs_line))?;
                            }
                            w.write_line(" */")?;
                        }
                        w.write_line("// @ts-ignore: decorator")?
                            .write_line("@inline")?
                            .write_line(format!(
                                "export const {}: {} = {};",
                                variant.name.as_str().to_uppercase(),
                                name,
                                i
                            ))?;
                    }
                    w0.write_line("}")?
                        .write_line(format!("export type {} = {};", name, as_type))?
                        .eob()?;
                }
                witx::Type::Flags(flags) => {
                    let as_type = match flags.repr {
                        witx::IntRepr::U8 => "u8",
                        witx::IntRepr::U16 => "u16",
                        witx::IntRepr::U32 => "u32",
                        witx::IntRepr::U64 => "u64",
                    };
                    w0.write_line(format!("export namespace {} {{", name))?;
                    let mut w = w0.new_block();
                    for (i, variant) in flags.flags.iter().enumerate() {
                        let docs = &variant.docs;
                        if !docs.is_empty() {
                            if i > 0 {
                                w.eob()?;
                            }
                            w.write_line("/**")?;
                            for docs_line in docs.lines() {
                                w.write_line(format!(" * {}", docs_line))?;
                            }
                            w.write_line(" */")?;
                        }
                        w.write_line("// @ts-ignore: decorator")?
                            .write_line("@inline")?
                            .write_line(format!(
                                "export const {}: {} = {};",
                                variant.name.as_str().to_uppercase(),
                                name,
                                i
                            ))?;
                    }
                    w0.write_line("}")?
                        .write_line(format!("export type {} = {};", name, as_type))?
                        .eob()?;
                }
                witx::Type::Builtin(builtin) => {
                    let as_type: &str = match builtin {
                        witx::BuiltinType::U8 => "u8",
                        witx::BuiltinType::U16 => "u16",
                        witx::BuiltinType::U32 => "u32",
                        witx::BuiltinType::U64 => "u64",
                        witx::BuiltinType::S8 => "i8",
                        witx::BuiltinType::S16 => "i16",
                        witx::BuiltinType::S32 => "i32",
                        witx::BuiltinType::S64 => "i64",
                        witx::BuiltinType::Char8 => "char",
                        witx::BuiltinType::USize => "usize",
                        witx::BuiltinType::F32 => "f32",
                        witx::BuiltinType::F64 => "f64",
                        witx::BuiltinType::String => unimplemented!(),
                    };
                    w0.write_line(format!("export type {} = {};", name, as_type))?;
                }
                witx::Type::Union(union) => {
                    let tag = union.tag.as_ref();
                    let variants = &union.variants;
                    w0.write_line("// @ts-ignore: decorator")?
                        .write_line("@unmanaged")?
                        .write_line(format!("export class {} {{", name))?;
                    let mut w = w0.new_block();
                    w.write_line("/** union tag */")?
                        .write_line(format!("tag: {};", tag.name.as_str()))?
                        .eob()?
                        .write_line("// @ts-ignore: decorator")?
                        .write_line("@inline")?
                        .write_line(format!("constructor(tag: {}) {{", tag.name.as_str()))?;
                    {
                        w.new_block().write_line("this.tag = tag;")?;
                    }
                    w.write_line("}")?;
                    w.eob()?;
                    for (i, variant) in variants.iter().enumerate() {
                        match variant.tref.as_ref() {
                            None => {
                                w.write_line(format!("{}: void; // if tag={}", name, i))?;
                            }
                            Some(witx::TypeRef::Name(another_type)) => {
                                w.write_line(format!(
                                    "{}: {}; // if tag={}",
                                    variant.name.as_str(),
                                    another_type.name.as_str(),
                                    i
                                ))?;
                            }
                            Some(witx::TypeRef::Value(_type_ref)) => match type_def.as_ref() {
                                witx::Type::Enum(enum_data_type) => {
                                    let as_type = match enum_data_type.repr {
                                        witx::IntRepr::U8 => "u8",
                                        witx::IntRepr::U16 => "u16",
                                        witx::IntRepr::U32 => "u32",
                                        witx::IntRepr::U64 => "u64",
                                    };
                                    w.write_line(format!(
                                        "{}: {}; // if tag={}",
                                        variant.name.as_str(),
                                        as_type,
                                        i
                                    ))?;
                                }
                                _ => unimplemented!(),
                            },
                        }
                        match variant.tref.as_ref() {
                            None => {
                                w.write_line("// @ts-ignore: decorator")?
                                    .write_line("@inline")?
                                    .write_line(format!(
                                        "new_{}(): {} {{",
                                        variant.name.as_str(),
                                        name
                                    ))?
                                    .indent()?
                                    .write_line(format!("this.tag = {};", i))?
                                    .write_line("}")?
                                    .eob()?
                                    .write_line("// @ts-ignore: decorator")?
                                    .write_line("@inline")?
                                    .write_line(format!(
                                        "set_{}(): void {{",
                                        variant.name.as_str()
                                    ))?
                                    .indent()?
                                    .write_line(format!("this.tag = {};", i))?
                                    .write_line("}")?
                                    .eob()?
                                    .write_line("// @ts-ignore: decorator")?
                                    .write_line("@inline")?
                                    .write_line(format!("is_{}(): bool {{", variant.name.as_str()))?
                                    .indent()?
                                    .write_line(format!("return this.tag = {};", i))?
                                    .write_line("}")?;
                            }
                            Some(witx::TypeRef::Name(another_type)) => {
                                w.write_line("// @ts-ignore: decorator")?
                                    .write_line("@inline")?
                                    .write_line(format!(
                                        "new_{}(val: {}): {} {{",
                                        variant.name.as_str(),
                                        another_type.name.as_str(),
                                        name
                                    ))?;
                                {
                                    w.new_block()
                                        .write_line(format!("this.tag = {};", i))?
                                        .write_line(format!(
                                            "this.{} = val;",
                                            variant.name.as_str()
                                        ))?;
                                }
                                w.write_line("}")?.eob()?;

                                w.write_line("// @ts-ignore: decorator")?
                                    .write_line("@inline")?
                                    .write_line(format!(
                                        "set_{}(val: {}): void {{",
                                        variant.name.as_str(),
                                        another_type.name.as_str()
                                    ))?;
                                {
                                    w.new_block()
                                        .write_line(format!("this.tag = {};", i))?
                                        .write_line(format!(
                                            "this.{} = val;",
                                            variant.name.as_str()
                                        ))?;
                                }
                                w.write_line("}")?.eob()?;

                                w.write_line("// @ts-ignore: decorator")?
                                    .write_line("@inline")?
                                    .write_line(format!(
                                        "get_{}(): {} | null {{",
                                        variant.name.as_str(),
                                        another_type.name.as_str()
                                    ))?;

                                {
                                    let mut w = w.new_block();
                                    w.write_line(format!("if (this.tag !== {}) {{", i))?;
                                    {
                                        w.new_block().write_line("return null;")?;
                                    }
                                    w.write_line("}")?.write_line(format!(
                                        "return this.{};",
                                        variant.name.as_str()
                                    ))?;
                                }
                                w.write_line("}")?;
                            }
                            _ => unimplemented!(),
                        }
                        w.eob()?;
                    }
                    w.write_line("}")?;
                }
                e => {
                    dbg!(e);
                    unimplemented!();
                }
            },
        };
        w0.eob()?;
        Ok(())
    }

    fn define_module(&mut self, module: &witx::Module) -> Result<(), Error> {
        let w0 = &mut self.w.clone();
        w0.eob()?.write_line(format!(
            "// ----------------------[{}]----------------------",
            module.name.as_str()
        ))?;
        for func in module.funcs() {
            self.define_func(module.name.as_str(), func.as_ref())?;
        }
        w0.eob()?;
        Ok(())
    }

    fn define_func(&mut self, module_name: &str, func: &witx::InterfaceFunc) -> Result<(), Error> {
        let w0 = &mut self.w;
        let docs = &func.docs;
        let name = func.name.as_str();
        if docs.is_empty() {
            w0.write_line(format!("\n/** {} */", name))?;
        } else {
            w0.eob()?.write_line("/**")?;
            for docs_line in docs.lines() {
                w0.write_line(format!(" * {}", docs_line))?;
            }
            w0.write_line(" */")?;
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

        //
        let params = &func.params;
        let as_params = params_to_as(params);
        let results = &func.results;
        let as_results = params_to_as(results);
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
            .map(|(v, t)| format!("{}_ptr: mut_ptr<{}>", v, t))
            .collect();
        if !as_params.is_empty() {
            if !as_results.is_empty() {
                w0.continuation()?
                    .write_line(format!("{},", as_params.join(", ")))?;
            } else {
                w0.continuation()?
                    .write_line(format!("{}", as_params.join(", ")))?;
            }
        }
        let return_as_type_and_comment = match return_value {
            None => ("void".to_string(), "".to_string()),
            Some(x) => (x.1.clone(), format!(" /* {} */", x.0)),
        };
        if !as_results.is_empty() {
            w0.continuation()?
                .write_line(format!("{}", as_results.join(", ")))?;
        }
        w0.write_line(format!(
            "): {}{};",
            return_as_type_and_comment.0, return_as_type_and_comment.1
        ))?;
        Ok(())
    }
}

fn wasm_atom_type_to_as(atom_type: witx::AtomType) -> &'static str {
    match atom_type {
        witx::AtomType::I32 => "i32",
        witx::AtomType::I64 => "i64",
        witx::AtomType::F32 => "f32",
        witx::AtomType::F64 => "f64",
    }
}

fn leaf_type(type_ref: &witx::TypeRef) -> &witx::Type {
    match type_ref {
        witx::TypeRef::Name(other_type) => {
            let x = other_type.as_ref();
            return leaf_type(&x.tref);
        }
        witx::TypeRef::Value(type_) => type_.as_ref(),
    }
}

fn params_to_as(params: &[witx::InterfaceFuncParam]) -> Vec<(String, String)> {
    let mut as_params = vec![];
    for param in params {
        let leaf_type = leaf_type(&param.tref);
        let second_part = match leaf_type {
            witx::Type::Array(_) => Some(("size", "untyped_ptr")),
            witx::Type::Builtin(witx::BuiltinType::String) => Some(("size", "usize")),
            witx::Type::Builtin(_) => None,
            witx::Type::Enum(_) => None,
            witx::Type::Flags(_) => None,
            witx::Type::Handle(_) => None,
            witx::Type::Int(_) => None,
            witx::Type::Pointer(_) | witx::Type::ConstPointer(_) => None,
            witx::Type::Struct(_) => None,
            witx::Type::Union(_) => Some(("member", "union_member")),
        };
        let first_part = match &param.tref {
            witx::TypeRef::Name(other_type) => other_type.name.as_str().to_string(),
            witx::TypeRef::Value(type_) => match type_.as_ref() {
                witx::Type::Array(_) => "untyped_ptr".to_string(),
                witx::Type::Builtin(builtin) => match builtin {
                    witx::BuiltinType::U8 => "u8",
                    witx::BuiltinType::U16 => "u16",
                    witx::BuiltinType::U32 => "u32",
                    witx::BuiltinType::U64 => "u64",
                    witx::BuiltinType::S8 => "i8",
                    witx::BuiltinType::S16 => "i16",
                    witx::BuiltinType::S32 => "i32",
                    witx::BuiltinType::S64 => "i64",
                    witx::BuiltinType::Char8 => "char",
                    witx::BuiltinType::USize => "usize",
                    witx::BuiltinType::F32 => "f32",
                    witx::BuiltinType::F64 => "f64",
                    witx::BuiltinType::String => "wasi_string",
                }
                .to_string(),
                witx::Type::Pointer(type_ref) | witx::Type::ConstPointer(type_ref) => {
                    match type_ref {
                        witx::TypeRef::Name(other_type) => {
                            format!("ptr<{}>", other_type.as_ref().name.as_str())
                        }
                        witx::TypeRef::Value(type_) => match type_.as_ref() {
                            witx::Type::Builtin(witx::BuiltinType::String) => {
                                "ptr<wasi_string>".to_string()
                            }
                            witx::Type::Builtin(builtin_type) => {
                                let as_builtin = match builtin_type {
                                    witx::BuiltinType::U8 => "u8",
                                    witx::BuiltinType::U16 => "u16",
                                    witx::BuiltinType::U32 => "u32",
                                    witx::BuiltinType::U64 => "u64",
                                    witx::BuiltinType::S8 => "i8",
                                    witx::BuiltinType::S16 => "i16",
                                    witx::BuiltinType::S32 => "i32",
                                    witx::BuiltinType::S64 => "i64",
                                    witx::BuiltinType::Char8 => "char",
                                    witx::BuiltinType::USize => "usize",
                                    witx::BuiltinType::F32 => "f32",
                                    witx::BuiltinType::F64 => "f64",
                                    witx::BuiltinType::String => "wasi_string",
                                };
                                format!("ptr<{}>", as_builtin)
                            }
                            _ => "untyped_ptr".to_string(),
                        },
                    }
                }
                witx::Type::Enum(enum_data_type) => match enum_data_type.repr {
                    witx::IntRepr::U8 => "u8",
                    witx::IntRepr::U16 => "u16",
                    witx::IntRepr::U32 => "u32",
                    witx::IntRepr::U64 => "u64",
                }
                .to_string(),
                witx::Type::Flags(flags) => match flags.repr {
                    witx::IntRepr::U8 => "u8",
                    witx::IntRepr::U16 => "u16",
                    witx::IntRepr::U32 => "u32",
                    witx::IntRepr::U64 => "u64",
                }
                .to_string(),
                witx::Type::Handle(_) => "handle".to_string(),
                witx::Type::Int(int) => match int.repr {
                    witx::IntRepr::U8 => "u8",
                    witx::IntRepr::U16 => "u16",
                    witx::IntRepr::U32 => "u32",
                    witx::IntRepr::U64 => "u64",
                }
                .to_string(),
                witx::Type::Struct(_) => "untyped_ptr".to_string(),
                witx::Type::Union(u) => u.tag.as_ref().name.as_str().to_string(),
            },
        };
        as_params.push((param.name.as_str().to_string(), first_part));
        if let Some(second_part) = second_part {
            as_params.push((
                format!("{}_{}", param.name.as_str(), second_part.0),
                second_part.1.to_string(),
            ))
        }
    }
    as_params
}

fn main() {
    let mut generator = Generator::new(std::io::stdout());
    generator.generate("/tmp/xqd.witx").unwrap();
}
