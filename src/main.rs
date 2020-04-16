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
    print!("export declare function {}(", name);

    //
    if false {
        for param in &func.params {
            println!("- {}: ", param.name.as_str());
            match &param.tref {
                witx::TypeRef::Name(other_type) => {
                    println!("    > {}", other_type.name.as_str());
                }
                witx::TypeRef::Value(type_) => match type_.as_ref() {
                    witx::Type::Handle(_) => println!("    > handle"),
                    _ => {}
                },
            }
        }
    }
    //

    let core_type = func.core_type();
    let mut first = true;
    for (i, core_type) in core_type.args.iter().enumerate() {
        if !first {
            print!(", ");
        }
        print!("a{}: {}", i, wasm_atom_type_to_as(core_type.repr()));
        first = false;
    }
    match core_type.ret {
        None => println!("): void;"),
        Some(core_type) => {
            let as_ret_type = wasm_atom_type_to_as(core_type.repr());
            println!("): {};", as_ret_type);
        }
    }
}

fn header() {
    println!("type handle = i32;");
    println!("type char = u8;");
    println!("type ptr<T> = usize; // all pointers are usize'd");
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
