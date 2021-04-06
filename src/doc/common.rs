use crate::astype::*;

pub trait Normalize {
    fn as_str(&self) -> &str;

    fn as_link(&self) -> String {
        let s = self.as_str().trim().to_lowercase();
        let s = s.trim_matches(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-'));
        format!("#{}", s)
    }

    fn as_type(&self) -> String {
        format!("_[`{}`]({})_", self.as_str(), self.as_link())
    }

    fn as_fn(&self) -> String {
        format!("[`{}()`]({})", self.as_str(), self.as_link())
    }

    fn as_fn_suffix(&self) -> String {
        format!("{}", self.as_str())
    }

    fn as_var(&self) -> String {
        format!("**`{}`**", self.as_str())
    }

    fn as_const(&self) -> String {
        format!("**`{}`**", self.as_str())
    }

    fn as_namespace(&self) -> String {
        format!("**[`{}`]({})**", self.as_str(), self.as_link())
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
            ASType::Bool => "`bool`".to_string(),
            ASType::Char32 => "`char32`".to_string(),
            ASType::Char8 => "`char8`".to_string(),
            ASType::F32 => "`f32`".to_string(),
            ASType::F64 => "`f64`".to_string(),
            ASType::Handle(_resource_name) => "`handle`".to_string(),
            ASType::ConstPtr(pointee) => format!("`{}` pointer", pointee.to_string()),
            ASType::MutPtr(pointee) => format!("`{}` mutable pointer", pointee.to_string()),
            ASType::Option(_) => todo!(),
            ASType::Result(_) => todo!(),
            ASType::S8 => "`i8`".to_string(),
            ASType::S16 => "`i16`".to_string(),
            ASType::S32 => "`i32`".to_string(),
            ASType::S64 => "`i64`".to_string(),
            ASType::U8 => "`u8`".to_string(),
            ASType::U16 => "`u16`".to_string(),
            ASType::U32 => "`u32`".to_string(),
            ASType::U64 => "`u64`".to_string(),
            ASType::USize => "`usize`".to_string(),
            ASType::Void => "_(empty)_".to_string(),
            ASType::Constants(_) => unimplemented!(),
            ASType::Enum(enum_) => {
                format!("`{}` enumeration", enum_.repr.as_ref().as_lang())
            }
            ASType::Struct(_) => unimplemented!(),
            ASType::Tuple(tuple_members) => {
                let tuple_types: Vec<_> =
                    tuple_members.iter().map(|x| x.type_.to_string()).collect();
                format!("`({})`", tuple_types.join(", "))
            }
            ASType::Union(_) => unimplemented!(),
            ASType::Slice(element_type) => format!("`{}` mutable slice", element_type.as_lang()),
            ASType::String(_) => "`string`".to_string(),
            ASType::ReadBuffer(element_type) => format!("`{}` slice", element_type.as_lang()),
            ASType::WriteBuffer(element_type) => {
                format!("`{}`ß mutable slice", element_type.to_string())
            }
        }
    }
}

impl ToLanguageRepresentation for ASType {
    fn as_astype(&self) -> &ASType {
        &self
    }
}
