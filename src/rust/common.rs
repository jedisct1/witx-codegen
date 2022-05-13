use convert_case::{Case, Casing};

use super::tuple::Tuple;
use crate::astype::*;

pub trait IsNullable {
    fn is_nullable(&self) -> bool;
}

impl IsNullable for ASType {
    fn is_nullable(&self) -> bool {
        matches!(
            self,
            ASType::ConstPtr(_)
                | ASType::MutPtr(_)
                | ASType::ReadBuffer(_)
                | ASType::WriteBuffer(_)
                | ASType::Enum(_)
                | ASType::Struct(_)
                | ASType::Tuple(_)
                | ASType::Union(_)
        )
    }
}

pub trait Normalize {
    fn as_str(&self) -> &str;

    fn as_type(&self) -> String {
        self.as_str().to_case(Case::Pascal)
    }

    fn as_fn(&self) -> String {
        escape_reserved_word(&self.as_str().to_case(Case::Snake))
    }

    fn as_fn_suffix(&self) -> String {
        escape_reserved_word(&self.as_str().to_case(Case::Snake))
    }

    fn as_var(&self) -> String {
        escape_reserved_word(&self.as_str().to_case(Case::Snake))
    }

    fn as_const(&self) -> String {
        self.as_str().to_case(Case::UpperSnake)
    }

    fn as_namespace(&self) -> String {
        self.as_str().to_string().to_case(Case::UpperSnake)
    }
}

impl<T: AsRef<str>> Normalize for T {
    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

pub trait ToLanguageRepresentation {
    fn as_astype(&self) -> &ASType;

    fn to_string(&self) -> String {
        self.as_lang()
    }

    fn as_lang(&self) -> String {
        match self.as_astype() {
            ASType::Alias(alias) => alias.name.as_type(),
            ASType::Bool => "bool".to_string(),
            ASType::Char32 => "Char32".to_string(),
            ASType::Char8 => "Char8".to_string(),
            ASType::F32 => "f32".to_string(),
            ASType::F64 => "f64".to_string(),
            ASType::Handle(_resource_name) => "WasiHandle".to_string(),
            ASType::ConstPtr(pointee) => format!("WasiPtr<{}>", pointee.to_string()),
            ASType::MutPtr(pointee) => format!("WasiMutPtr<{}>", pointee.to_string()),
            ASType::Option(_) => todo!(),
            ASType::Result(_) => todo!(),
            ASType::S8 => "i8".to_string(),
            ASType::S16 => "i16".to_string(),
            ASType::S32 => "i32".to_string(),
            ASType::S64 => "i64".to_string(),
            ASType::U8 => "u8".to_string(),
            ASType::U16 => "u16".to_string(),
            ASType::U32 => "u32".to_string(),
            ASType::U64 => "u64".to_string(),
            ASType::USize => "usize".to_string(),
            ASType::Void => "()".to_string(),
            ASType::Constants(_) => unimplemented!(),
            ASType::Enum(enum_) => {
                format!("{} /* Enum */", enum_.repr.as_ref().as_lang())
            }
            ASType::Struct(_) => unimplemented!(),
            ASType::Tuple(tuple_members) => Tuple::name_for(tuple_members).as_type(),
            ASType::Union(_) => unimplemented!(),
            ASType::Slice(element_type) => format!("WasiMutSlice<{}>", element_type.as_lang()),
            ASType::String(_) => "WasiString".to_string(),
            ASType::ReadBuffer(element_type) => format!("WasiSlice<{}>", element_type.as_lang()),
            ASType::WriteBuffer(element_type) => {
                format!("WasiMutSlice<{}>", element_type.to_string())
            }
        }
    }
}

impl ToLanguageRepresentation for ASType {
    fn as_astype(&self) -> &ASType {
        self
    }
}

/// Checks the given word against a list of reserved keywords.
/// If the given word conflicts with a keyword, a trailing underscore will be
/// appended.
///
/// Adapted from [wiggle](https://docs.rs/wiggle/latest/wiggle/index.html)
pub fn escape_reserved_word(word: &str) -> String {
    if STRICT.iter().chain(RESERVED).any(|k| *k == word) {
        // If the camel-cased string matched any strict or reserved keywords, then
        // append a trailing underscore to the identifier we generate.
        format!("{}_", word)
    } else {
        word.to_string() // Otherwise, use the string as is.
    }
}

/// Strict keywords.
///
/// Source: [The Rust Reference][https://doc.rust-lang.org/reference/keywords.html#strict-keywords]
const STRICT: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

/// Reserved keywords.
///
/// These keywords aren't used yet, but they are reserved for future use. They
/// have the same restrictions as strict keywords. The reasoning behind this is
/// to make current programs forward compatible with future versions of Rust by
/// forbidding them to use these keywords.
///
/// Source: [The Rust Reference](https://doc.rust-lang.org/reference/keywords.html#reserved-keywords)
const RESERVED: &[&str] = &[
    "abstract", "become", "box", "do", "final", "macro", "override", "priv", "try", "typeof",
    "unsized", "virtual", "yield",
];
