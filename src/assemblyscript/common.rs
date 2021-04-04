use super::tuple::Tuple;
use crate::astype::*;
use convert_case::{Case, Casing};
use std::fmt;

pub trait IsNullable {
    fn is_nullable(&self) -> bool;
}

impl IsNullable for ASType {
    fn is_nullable(&self) -> bool {
        match self {
            ASType::ConstPtr(_)
            | ASType::MutPtr(_)
            | ASType::ReadBuffer(_)
            | ASType::WriteBuffer(_)
            | ASType::Enum(_)
            | ASType::Struct(_)
            | ASType::Tuple(_)
            | ASType::Union(_) => true,
            _ => false,
        }
    }
}

pub trait Normalize {
    fn as_str(&self) -> &str;

    fn as_type(&self) -> String {
        self.as_str().to_case(Case::Pascal)
    }

    fn as_fn(&self) -> String {
        self.as_str().to_case(Case::Camel)
    }

    fn as_fn_suffix(&self) -> String {
        self.as_str().to_case(Case::UpperCamel)
    }

    fn as_var(&self) -> String {
        self.as_str().to_case(Case::Snake)
    }

    fn as_const(&self) -> String {
        self.as_str().to_case(Case::UpperSnake)
    }
}

impl<T: AsRef<str>> Normalize for T {
    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Display for ASType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
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
            ASType::Void => "void".to_string(),
            ASType::Constants(_) => unimplemented!(),
            ASType::Enum(enum_) => {
                format!("{} /* Enum */", enum_.repr.as_ref())
            }
            ASType::Struct(_) => unimplemented!(),
            ASType::Tuple(tuple_members) => Tuple::name_for(tuple_members).as_type(),
            ASType::Union(_) => unimplemented!(),
            ASType::Slice(element_type) => format!("WasiMutSlice<{}>", element_type),
            ASType::String(_) => "WasiString".to_string(),
            ASType::ReadBuffer(element_type) => format!("WasiSlice<{}>", element_type),
            ASType::WriteBuffer(element_type) => {
                format!("WasiMutSlice<{}>", element_type.to_string())
            }
        };
        write!(f, "{}", s)
    }
}
