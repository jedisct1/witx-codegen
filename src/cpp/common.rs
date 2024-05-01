use super::tuple::Tuple;
use crate::astype::*;
use convert_case::{Case, Casing};

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
        self.as_str().to_case(Case::Snake)
    }

    fn as_fn_suffix(&self) -> String {
        self.as_str().to_case(Case::Snake)
    }

    fn as_var(&self) -> String {
        self.as_str().to_case(Case::Snake)
    }

    fn as_const(&self) -> String {
        self.as_str().to_case(Case::UpperSnake)
    }

    fn as_namespace(&self) -> String {
        self.as_str().to_case(Case::Pascal)
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
            ASType::Char32 => "char32_t".to_string(),
            ASType::Char8 => "unsigned char".to_string(),
            ASType::F32 => "float".to_string(),
            ASType::F64 => "double".to_string(),
            ASType::Handle(_resource_name) => "WasiHandle".to_string(),
            ASType::ConstPtr(pointee) => format!("WasiPtr<{}>", pointee.to_string()),
            ASType::MutPtr(pointee) => format!("WasiMutPtr<{}>", pointee.to_string()),
            ASType::Option(_) => todo!(),
            ASType::Result(_) => todo!(),
            ASType::S8 => "int8_t".to_string(),
            ASType::S16 => "int16_t".to_string(),
            ASType::S32 => "int32_t".to_string(),
            ASType::S64 => "int64_t".to_string(),
            ASType::U8 => "uint8_t".to_string(),
            ASType::U16 => "uint16_t".to_string(),
            ASType::U32 => "uint32_t".to_string(),
            ASType::U64 => "uint64_t".to_string(),
            ASType::USize => "size_t".to_string(),
            ASType::Void => "void".to_string(),
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
