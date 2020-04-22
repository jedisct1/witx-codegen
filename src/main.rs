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
        let as_type = ASType::Alias(type_.name.as_str().to_string());
        let docs = &type_.docs;
        if docs.is_empty() {
            w0.write_line(format!("/** {} */", as_type))?;
        } else {
            write_docs(w0, &type_.docs)?;
        }
        let tref = &type_.tref;
        match tref {
            witx::TypeRef::Name(another_type) => {
                w0.write_line(format!(
                    "export type {} = {};",
                    as_type,
                    ASType::Alias(another_type.name.as_str().to_string())
                ))?;
            }
            witx::TypeRef::Value(type_def) => match type_def.as_ref() {
                witx::Type::Enum(enum_data_type) => {
                    let actual_as_type = ASType::from(enum_data_type.repr);
                    w0.write_line(format!("export namespace {} {{", as_type))?;
                    let mut w = w0.new_block();
                    for (i, variant) in enum_data_type.variants.iter().enumerate() {
                        write_docs(&mut w, &variant.docs)?;
                        w.write_line("// @ts-ignore: decorator")?
                            .write_line("@inline")?
                            .write_line(format!(
                                "export const {}: {} = {};",
                                variant.name.as_str().to_uppercase(),
                                as_type,
                                i
                            ))?;
                    }
                    w0.write_line("}")?
                        .write_line(format!("export type {} = {};", as_type, actual_as_type))?
                        .eob()?;
                }
                witx::Type::Handle(_handle) => {
                    w0.write_line(format!("export type {} = {};", as_type, ASType::Handle))?;
                }
                witx::Type::Int(int) => {
                    let actual_as_type = ASType::from(int);
                    w0.write_line(format!("export namespace {} {{", as_type))?;
                    let mut w = w0.new_block();
                    for (i, variant) in int.consts.iter().enumerate() {
                        write_docs(&mut w, &variant.docs)?;
                        w.write_line("// @ts-ignore: decorator")?
                            .write_line("@inline")?
                            .write_line(format!(
                                "export const {}: {} = {};",
                                variant.name.as_str().to_uppercase(),
                                as_type,
                                i
                            ))?;
                    }
                    w0.write_line("}")?
                        .write_line(format!("export type {} = {};", as_type, actual_as_type))?
                        .eob()?;
                }
                witx::Type::Flags(flags) => {
                    let actual_as_type = ASType::from(flags);
                    w0.write_line(format!("export namespace {} {{", as_type))?;
                    let mut w = w0.new_block();
                    for (i, variant) in flags.flags.iter().enumerate() {
                        write_docs(&mut w, &variant.docs)?;
                        w.write_line("// @ts-ignore: decorator")?
                            .write_line("@inline")?
                            .write_line(format!(
                                "export const {}: {} = {};",
                                variant.name.as_str().to_uppercase(),
                                as_type,
                                1u64 << i
                            ))?;
                    }
                    w0.write_line("}")?
                        .write_line(format!("export type {} = {};", as_type, actual_as_type))?
                        .eob()?;
                }
                witx::Type::Builtin(builtin) => {
                    let actual_as_type = ASType::from(builtin);
                    w0.write_line(format!("export type {} = {};", as_type, actual_as_type))?;
                }
                witx::Type::Union(union) => {
                    let as_tag = ASType::from(union.tag.as_ref());
                    let variants = &union.variants;
                    w0.write_line("// @ts-ignore: decorator")?
                        .write_line("@unmanaged")?
                        .write_line(format!("export class {} {{", as_type))?;
                    let mut w = w0.new_block();
                    w.write_line(format!("tag: {};", as_tag))?
                        .eob()?
                        .write_line("// @ts-ignore: decorator")?
                        .write_line("@inline")?
                        .write_line(format!("constructor(tag: {}) {{", as_tag))?;
                    {
                        w.new_block().write_line("this.tag = tag;")?;
                    }
                    w.write_line("}")?;
                    w.eob()?;
                    for (i, variant) in variants.iter().enumerate() {
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
                            Some(witx::TypeRef::Value(_type_ref)) => match type_def.as_ref() {
                                witx::Type::Enum(enum_data_type) => {
                                    let as_type = ASType::from(enum_data_type);
                                    w.write_line(format!(
                                        "{}: {}; // if tag={}",
                                        variant_name, as_type, i
                                    ))?;
                                }
                                _ => unimplemented!(),
                            },
                        }
                        w.eob()?;
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
                                    .write_line(format!("return this.tag = {};", i))?
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
                        w.eob()?;
                    }
                    w0.write_line("}")?;
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
            write_docs(w0, docs)?;
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
            .map(|(v, t)| format!("{}_ptr: {}", v, ASType::MutPtr(Box::new(t.clone()))))
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
            None => (ASType::Void, "".to_string()),
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

#[derive(Clone, Debug, Eq, PartialEq)]
enum ASType {
    Void,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Char,
    Usize,
    F32,
    F64,
    Alias(String),
    Ptr(Box<ASType>),
    MutPtr(Box<ASType>),
    UntypedPtr,
    UnionMember,
    Struct(String),
    WasiString,
    Handle,
}

impl fmt::Display for ASType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ASType::Void => write!(f, "void"),
            ASType::U8 => write!(f, "u8"),
            ASType::U16 => write!(f, "u16"),
            ASType::U32 => write!(f, "u32"),
            ASType::U64 => write!(f, "u64"),
            ASType::I8 => write!(f, "i8"),
            ASType::I16 => write!(f, "i16"),
            ASType::I32 => write!(f, "i32"),
            ASType::I64 => write!(f, "i64"),
            ASType::Char => write!(f, "char"),
            ASType::Usize => write!(f, "usize"),
            ASType::F32 => write!(f, "f32"),
            ASType::F64 => write!(f, "f64"),
            ASType::Alias(to) => write!(f, "{}", to),
            ASType::Ptr(other_type) => write!(f, "ptr<{}>", other_type),
            ASType::MutPtr(other_type) => write!(f, "mut_ptr<{}>", other_type),
            ASType::UntypedPtr => write!(f, "untyped_ptr"),
            ASType::UnionMember => write!(f, "union_member"),
            ASType::Struct(name) => write!(f, "untyped_ptr /* {} struct */", name),
            ASType::WasiString => write!(f, "wasi_string"),
            ASType::Handle => write!(f, "handle"),
        }
    }
}

impl From<witx::IntRepr> for ASType {
    fn from(witx: witx::IntRepr) -> Self {
        match witx {
            witx::IntRepr::U8 => ASType::U8,
            witx::IntRepr::U16 => ASType::U16,
            witx::IntRepr::U32 => ASType::U32,
            witx::IntRepr::U64 => ASType::U64,
        }
    }
}

impl From<&witx::BuiltinType> for ASType {
    fn from(witx: &witx::BuiltinType) -> Self {
        match witx {
            witx::BuiltinType::U8 => ASType::U8,
            witx::BuiltinType::U16 => ASType::U16,
            witx::BuiltinType::U32 => ASType::U32,
            witx::BuiltinType::U64 => ASType::U64,
            witx::BuiltinType::S8 => ASType::I8,
            witx::BuiltinType::S16 => ASType::I16,
            witx::BuiltinType::S32 => ASType::I32,
            witx::BuiltinType::S64 => ASType::I64,
            witx::BuiltinType::String => ASType::WasiString,
            witx::BuiltinType::USize => ASType::Usize,
            witx::BuiltinType::F32 => ASType::F32,
            witx::BuiltinType::F64 => ASType::F64,
            witx::BuiltinType::Char8 => ASType::Char,
        }
    }
}

impl From<&witx::EnumDatatype> for ASType {
    fn from(witx: &witx::EnumDatatype) -> Self {
        witx.repr.into()
    }
}

impl From<&witx::FlagsDatatype> for ASType {
    fn from(witx: &witx::FlagsDatatype) -> Self {
        witx.repr.into()
    }
}

impl From<&witx::IntDatatype> for ASType {
    fn from(witx: &witx::IntDatatype) -> Self {
        witx.repr.into()
    }
}

impl From<&witx::HandleDatatype> for ASType {
    fn from(_witx: &witx::HandleDatatype) -> Self {
        ASType::Handle
    }
}

impl From<&witx::NamedType> for ASType {
    fn from(witx: &witx::NamedType) -> Self {
        ASType::Alias(witx.name.as_str().to_string())
    }
}

impl From<&witx::UnionDatatype> for ASType {
    fn from(witx: &witx::UnionDatatype) -> Self {
        witx.tag.as_ref().into()
    }
}

impl From<&witx::Type> for ASType {
    fn from(witx: &witx::Type) -> Self {
        match witx {
            witx::Type::Builtin(x) => x.into(),
            witx::Type::ConstPointer(_x) => ASType::UntypedPtr,
            witx::Type::Pointer(_x) => ASType::UntypedPtr,
            witx::Type::Enum(x) => x.into(),
            witx::Type::Flags(x) => x.into(),
            witx::Type::Handle(x) => x.into(),
            witx::Type::Int(x) => x.into(),
            witx::Type::Struct(_) => unimplemented!(),
            witx::Type::Union(_) => unimplemented!(),
            witx::Type::Array(_) => unimplemented!(),
        }
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

fn params_to_as(params: &[witx::InterfaceFuncParam]) -> Vec<(String, ASType)> {
    let mut as_params = vec![];
    for param in params {
        let leaf_type = leaf_type(&param.tref);
        let second_part = match leaf_type {
            witx::Type::Array(_) => Some(("size", ASType::UntypedPtr)),
            witx::Type::Builtin(witx::BuiltinType::String) => Some(("size", ASType::Usize)),
            witx::Type::Builtin(_) => None,
            witx::Type::Enum(_) => None,
            witx::Type::Flags(_) => None,
            witx::Type::Handle(_) => None,
            witx::Type::Int(_) => None,
            witx::Type::Pointer(_) | witx::Type::ConstPointer(_) => None,
            witx::Type::Struct(_) => None,
            witx::Type::Union(_) => Some(("member", ASType::UnionMember)),
        };
        let first_part = match &param.tref {
            witx::TypeRef::Name(other_type) => ASType::Alias(other_type.name.as_str().to_string()),
            witx::TypeRef::Value(type_) => match type_.as_ref() {
                witx::Type::Array(_) => ASType::UntypedPtr,
                witx::Type::Builtin(builtin) => ASType::from(builtin),
                witx::Type::Pointer(type_ref) | witx::Type::ConstPointer(type_ref) => {
                    match type_ref {
                        witx::TypeRef::Name(other_type) => ASType::Ptr(Box::new(ASType::Alias(
                            other_type.as_ref().name.as_str().to_string(),
                        ))),
                        witx::TypeRef::Value(type_) => match type_.as_ref() {
                            witx::Type::Builtin(witx::BuiltinType::String) => {
                                ASType::Ptr(Box::new(ASType::WasiString))
                            }
                            witx::Type::Builtin(builtin_type) => {
                                ASType::Ptr(Box::new(ASType::from(builtin_type)))
                            }
                            _ => ASType::UntypedPtr,
                        },
                    }
                }
                witx::Type::Enum(enum_data_type) => ASType::from(enum_data_type),
                witx::Type::Flags(flags) => ASType::from(flags),
                witx::Type::Handle(_) => ASType::Handle,
                witx::Type::Int(int) => ASType::from(int),
                witx::Type::Struct(_) => ASType::Struct(param.name.as_str().to_string()),
                witx::Type::Union(u) => ASType::from(u),
            },
        };
        as_params.push((param.name.as_str().to_string(), first_part));
        if let Some(second_part) = second_part {
            as_params.push((
                format!("{}_{}", param.name.as_str(), second_part.0),
                second_part.1,
            ))
        }
    }
    as_params
}

fn main() {
    let mut generator = Generator::new(std::io::stdout());
    generator
        .generate("/tmp/wasi_ephemeral_crypto.witx")
        .unwrap();
}
