use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ASType {
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
    UnionMember,
    Struct(Option<String>),
    WasiStringPtr,
    Handle,
    WasiString,
    Union(Box<ASType>),
    Array(Box<ASType>),
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
            ASType::UnionMember => write!(f, "union_member"),
            ASType::Struct(None) => write!(f, "untyped_struct"),
            ASType::Struct(Some(name)) => write!(f, "struct<{}>", name),
            ASType::WasiStringPtr => write!(f, "wasi_string_ptr"),
            ASType::Handle => write!(f, "handle"),
            ASType::WasiString => write!(f, "WasiString"),
            ASType::Union(_) => write!(f, "WasiUnion"),
            ASType::Array(_) => write!(f, "WasiArray"),
        }
    }
}

impl ASType {
    pub fn is_nullable(&self) -> bool {
        match self {
            ASType::Ptr(_)
            | ASType::MutPtr(_)
            | ASType::Struct(None)
            | ASType::Struct(Some(_))
            | ASType::WasiStringPtr
            | ASType::WasiString
            | ASType::Union(_)
            | ASType::Array(_) => true,
            _ => false,
        }
    }

    pub fn decompose(&self) -> ((ASType, &'static str), Option<(ASType, &'static str)>) {
        let first = match self {
            ASType::WasiString => (ASType::WasiStringPtr, "_ptr"),
            ASType::Union(tag_type) => (tag_type.as_ref().clone(), "_tag"),
            ASType::Array(element_type) => (ASType::Ptr(element_type.clone()), "_ptr"),
            t @ _ => (t.clone(), ""),
        };
        let second = match self {
            ASType::WasiString => Some((ASType::Usize, "_len")),
            ASType::Union(_tag_type) => Some((ASType::UnionMember, "_member")),
            ASType::Array(_element_type) => Some((ASType::Usize, "_count")),
            _ => None,
        };
        (first, second)
    }

    pub fn name(self, name: String) -> Self {
        match self {
            ASType::Struct(_) => ASType::Struct(Some(name)),
            x @ _ => x,
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

impl From<&witx::TypeRef> for ASType {
    fn from(witx: &witx::TypeRef) -> Self {
        witx.type_().as_ref().into()
    }
}

impl From<&witx::Type> for ASType {
    fn from(witx: &witx::Type) -> Self {
        match witx {
            witx::Type::Builtin(x) => x.into(),
            witx::Type::ConstPointer(x) => ASType::Ptr(Box::new(x.into())),
            witx::Type::Pointer(x) => ASType::MutPtr(Box::new(x.into())),
            witx::Type::Enum(x) => x.into(),
            witx::Type::Flags(x) => x.into(),
            witx::Type::Handle(x) => x.into(),
            witx::Type::Int(x) => x.into(),
            witx::Type::Struct(_) => ASType::Struct(None),
            witx::Type::Union(x) => ASType::Union(Box::new(x.tag.as_ref().into())),
            witx::Type::Array(x) => ASType::Array(Box::new(x.into())),
        }
    }
}
