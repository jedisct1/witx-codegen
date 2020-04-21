use std::{self, fmt};
use witx::WitxError;

#[derive(Debug)]
enum Error {
    Witx(WitxError),
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

fn doit() -> Result<(), Error> {
    let document = witx::load(&["/tmp/wasi_ephemeral_crypto.witx"])?;
    header();
    for type_ in document.typenames() {
        define_type(type_.as_ref());
    }
    for module in document.modules() {
        define_module(module.as_ref());
    }
    Ok(())
}

fn define_module(module: &witx::Module) {
    println!();
    println!();
    println!(
        "// ----------------------[{}]----------------------",
        module.name.as_str()
    );
    for func in module.funcs() {
        define_func(module.name.as_str(), func.as_ref());
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

fn define_func(module_name: &str, func: &witx::InterfaceFunc) {
    let docs = &func.docs;
    let name = func.name.as_str();
    if docs.is_empty() {
        println!("\n/** {} */", name);
    } else {
        println!("\n/**");
        for docs_line in docs.lines() {
            println!(" * {}", docs_line);
        }
        println!(" */");
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
    println!("/**");
    println!(" * in:  {}", s_in.join(", "));
    println!(" * out: {}", s_out.join(", "));
    println!(" */");

    println!("// @ts-ignore: decorator");
    println!("@external(\"{}\", \"{}\")", module_name, name);
    println!("export declare function {}(", name);

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
    if as_results.is_empty() {
        println!("    {}", as_params.join(", "));
    } else {
        println!("    {},", as_params.join(", "));
    }
    let return_as_type_and_comment = match return_value {
        None => ("void".to_string(), "".to_string()),
        Some(x) => (x.1.clone(), format!(" /* {} */", x.0)),
    };
    if !as_results.is_empty() {
        println!("    {}", as_results.join(", "));
    }
    println!(
        "): {}{};",
        return_as_type_and_comment.0, return_as_type_and_comment.1
    );
}

fn header() {
    println!("type handle = i32;");
    println!("type char = u8;");
    println!("type ptr<T> = usize; // all pointers are usize'd");
    println!("type mut_ptr<T> = usize; // all pointers are usize'd");
    println!("type untyped_ptr = usize; // all pointers are usize'd");
    println!("type union_member = usize; // all pointers are usize'd");
    println!("type struct<T> = T;  // structs are references already in AS)");
    println!("type wasi_string = ptr<char>;");
    println!("");
}

fn define_type(type_: &witx::NamedType) {
    let docs = &type_.docs;
    let name = type_.name.as_str();
    if docs.is_empty() {
        println!("\n/** {} */", name);
    } else {
        println!("\n/**");
        for docs_line in docs.lines() {
            println!(" * {}", docs_line);
        }
        println!(" */");
    }
    let tref = &type_.tref;
    match tref {
        witx::TypeRef::Name(another_type) => {
            println!("export type {} = {};", name, another_type.name.as_str())
        }
        witx::TypeRef::Value(type_def) => match type_def.as_ref() {
            witx::Type::Enum(enum_data_type) => {
                let as_type = match enum_data_type.repr {
                    witx::IntRepr::U8 => "u8",
                    witx::IntRepr::U16 => "u16",
                    witx::IntRepr::U32 => "u32",
                    witx::IntRepr::U64 => "u64",
                };
                println!("export namespace {} {{", name);
                for (i, variant) in enum_data_type.variants.iter().enumerate() {
                    let docs = &variant.docs;
                    if !docs.is_empty() {
                        if i > 0 {
                            println!();
                        }
                        println!("  /**");
                        for docs_line in docs.lines() {
                            println!("   * {}", docs_line);
                        }
                        println!("   */");
                    }
                    println!("  // @ts-ignore: decorator");
                    println!("  @inline");
                    println!(
                        "  export const {}: {} = {};",
                        variant.name.as_str().to_uppercase(),
                        name,
                        i
                    );
                }
                println!("}}");
                println!("export type {} = {};", name, as_type);
                println!();
            }
            witx::Type::Handle(_handle) => {
                println!("export type {} = handle;", name);
            }
            witx::Type::Int(int) => {
                let as_type = match int.repr {
                    witx::IntRepr::U8 => "u8",
                    witx::IntRepr::U16 => "u16",
                    witx::IntRepr::U32 => "u32",
                    witx::IntRepr::U64 => "u64",
                };
                println!("export namespace {} {{", name);
                for (i, variant) in int.consts.iter().enumerate() {
                    let docs = &variant.docs;
                    if !docs.is_empty() {
                        if i > 0 {
                            println!();
                        }
                        println!("  /**");
                        for docs_line in docs.lines() {
                            println!("   * {}", docs_line);
                        }
                        println!("   */");
                    }
                    println!("  // @ts-ignore: decorator");
                    println!("  @inline");
                    println!(
                        "  export const {}: {} = {};",
                        variant.name.as_str().to_uppercase(),
                        name,
                        i
                    );
                }
                println!("}}");
                println!("export type {} = {};", name, as_type);
                println!();
            }
            witx::Type::Flags(flags) => {
                let as_type = match flags.repr {
                    witx::IntRepr::U8 => "u8",
                    witx::IntRepr::U16 => "u16",
                    witx::IntRepr::U32 => "u32",
                    witx::IntRepr::U64 => "u64",
                };
                println!("export namespace {} {{", name);
                for (i, variant) in flags.flags.iter().enumerate() {
                    let docs = &variant.docs;
                    if !docs.is_empty() {
                        if i > 0 {
                            println!();
                        }
                        println!("  /**");
                        for docs_line in docs.lines() {
                            println!("   * {}", docs_line);
                        }
                        println!("   */");
                    }
                    println!("  // @ts-ignore: decorator");
                    println!("  @inline");
                    println!(
                        "  export const {}: {} = {};",
                        variant.name.as_str().to_uppercase(),
                        name,
                        i
                    );
                }
                println!("}}");
                println!("export type {} = {};", name, as_type);
                println!();
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
                println!("export type {} = {};", name, as_type);
            }
            witx::Type::Union(union) => {
                let tag = union.tag.as_ref();
                let variants = &union.variants;
                println!("// @ts-ignore: decorator");
                println!("@unmanaged");
                println!("export class {} {{", name);
                println!("  /** union tag */");
                println!("  tag: {};", tag.name.as_str());
                println!();
                println!("  // @ts-ignore: decorator");
                println!("  @inline");
                println!("  constructor(tag: {}) {{", tag.name.as_str());
                println!("    this.tag = tag;");
                println!("  }}");
                println!();
                for (i, variant) in variants.iter().enumerate() {
                    match variant.tref.as_ref() {
                        None => println!("  {}: void; // if tag={}", name, i),
                        Some(witx::TypeRef::Name(another_type)) => println!(
                            "  {}: {}; // if tag={}",
                            variant.name.as_str(),
                            another_type.name.as_str(),
                            i
                        ),
                        Some(witx::TypeRef::Value(_type_ref)) => match type_def.as_ref() {
                            witx::Type::Enum(enum_data_type) => {
                                let as_type = match enum_data_type.repr {
                                    witx::IntRepr::U8 => "u8",
                                    witx::IntRepr::U16 => "u16",
                                    witx::IntRepr::U32 => "u32",
                                    witx::IntRepr::U64 => "u64",
                                };
                                println!(
                                    "  {}: {}; // if tag={}",
                                    variant.name.as_str(),
                                    as_type,
                                    i
                                );
                            }
                            _ => unimplemented!(),
                        },
                    }
                    match variant.tref.as_ref() {
                        None => {
                            println!("  // @ts-ignore: decorator");
                            println!("  @inline");
                            println!("  new_{}(): {} {{", variant.name.as_str(), name);
                            println!("    this.tag = {};", i);
                            println!("  }}");
                            println!();
                            println!("  // @ts-ignore: decorator");
                            println!("  @inline");
                            println!("  set_{}(): void {{", variant.name.as_str(),);
                            println!("    this.tag = {};", i);
                            println!("  }}");
                            println!();
                            println!("  // @ts-ignore: decorator");
                            println!("  @inline");
                            println!("  is_{}(): bool {{", variant.name.as_str(),);
                            println!("    return this.tag = {};", i);
                            println!("  }}");
                        }
                        Some(witx::TypeRef::Name(another_type)) => {
                            println!("  // @ts-ignore: decorator");
                            println!("  @inline");
                            println!(
                                "  new_{}(val: {}): {} {{",
                                variant.name.as_str(),
                                another_type.name.as_str(),
                                name
                            );
                            println!("    this.tag = {};", i);
                            println!("    this.{} = val;", variant.name.as_str());
                            println!("  }}");
                            println!();
                            println!("  // @ts-ignore: decorator");
                            println!("  @inline");
                            println!(
                                "  set_{}(val: {}): void {{",
                                variant.name.as_str(),
                                another_type.name.as_str()
                            );
                            println!("    this.tag = {};", i);
                            println!("    this.{} = val;", variant.name.as_str());
                            println!("  }}");
                            println!();
                            println!("  // @ts-ignore: decorator");
                            println!("  @inline");
                            println!(
                                "  get_{}(): {} | null {{",
                                variant.name.as_str(),
                                another_type.name.as_str()
                            );
                            println!("    if (this.tag !== {}) {{", i);
                            println!("      return null;");
                            println!("    }}");
                            println!("    return this.{};", variant.name.as_str());
                            println!("  }}");
                        }
                        _ => unimplemented!(),
                    }
                    println!();
                }
                println!("}}");
            }
            e => {
                dbg!(e);
                unimplemented!();
            }
        },
    }
}

fn main() {
    doit().unwrap();
}
