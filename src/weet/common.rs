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
        self.as_str().to_case(Case::Kebab)
    }

    fn as_fn(&self) -> String {
        self.as_str().to_case(Case::Kebab)
    }

    fn as_fn_suffix(&self) -> String {
        self.as_str().to_case(Case::Kebab)
    }

    fn as_var(&self) -> String {
        escape_reserved_word(&self.as_str().to_case(Case::Kebab))
    }

    fn as_const(&self) -> String {
        self.as_str().to_case(Case::Kebab)
    }

    fn as_namespace(&self) -> String {
        self.as_str().to_string().to_case(Case::Kebab)
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
            ASType::Char32 => "char".to_string(),
            ASType::Char8 => "u8".to_string(),
            ASType::F32 => "f32".to_string(),
            ASType::F64 => "f64".to_string(),
            ASType::Handle(_resource_name) => "wasi-handle".to_string(),
            ASType::ConstPtr(pointee) => format!("{}", pointee.to_string()),
            ASType::MutPtr(pointee) => format!("borrow<{}>", pointee.to_string()),
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
            ASType::USize => "u32".to_string(),
            ASType::Void => "_".to_string(),
            ASType::Constants(_) => unimplemented!(),
            ASType::Enum(enum_) => enum_.repr.as_ref().as_lang(),
            ASType::Struct(_) => unimplemented!(),
            ASType::Tuple(tuple_members) => Tuple::name_for(tuple_members),
            ASType::Union(_) => unimplemented!(),
            ASType::Slice(element_type) => format!("list<{}>", element_type.as_lang()),
            ASType::String(_) => "string".to_string(),
            ASType::ReadBuffer(element_type) => format!("list<{}>", element_type.as_lang()),
            ASType::WriteBuffer(element_type) => {
                format!("borrow<list<{}>>", element_type.to_string())
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
/// If the given word conflicts with a keyword, a suffix will be
/// appended.
pub fn escape_reserved_word(word: &str) -> String {
    if RESERVED.iter().any(|k| *k == word) {
        // If the camel-cased string matched any strict or reserved keywords, then
        // prepend a prefix to the identifier we generate.
        format!("x-{}", word)
    } else {
        word.to_string() // Otherwise, use the string as is.
    }
}

/// Reserved Keywords.
///
const RESERVED: &[&str] = &[
    "list",
    "option",
    "result",
    "resource",
    "tuple",
    "record",
    "variant",
    "enum",
    "flags",
    "types",
    "func",
    "interface",
    "world",
    "import",
    "export",
    "package",
    "char",
    "string",
    "borrow",
];
