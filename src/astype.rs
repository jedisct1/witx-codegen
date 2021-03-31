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
    Record(Option<String>),
    Handle,
    Variant(Option<String>),
    List(Box<ASType>),
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
            ASType::Record(None) => write!(f, "usize /* struct */"),
            ASType::Record(Some(name)) => write!(f, "struct<{}>", name),
            ASType::Handle => write!(f, "handle"),
            ASType::Variant(None) => write!(f, "usize /* union */"),
            ASType::Variant(Some(name)) => write!(f, "union<{}>", name),
            ASType::List(_) => write!(f, "usize /* array */"),
        }
    }
}

impl ASType {
    pub fn is_nullable(&self) -> bool {
        match self {
            ASType::Ptr(_)
            | ASType::MutPtr(_)
            | ASType::Record(_)
            | ASType::Variant(_)
            | ASType::List(_) => true,
            _ => false,
        }
    }

    pub fn decompose(&self) -> ((ASType, &'static str), Option<(ASType, &'static str)>) {
        let first = match self {
            ASType::List(element_type) => (ASType::Ptr(element_type.clone()), "_ptr"),
            t => (t.clone(), ""),
        };
        let second = match self {
            ASType::List(_element_type) => Some((ASType::Usize, "_count")),
            _ => None,
        };
        (first, second)
    }

    pub fn name(self, name: String) -> Self {
        match self {
            ASType::Record(_) => ASType::Record(Some(name)),
            x => x,
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
            // Assembly script has no C char type
            witx::BuiltinType::U8 { lang_c_char: _ }  => ASType::U8,
            witx::BuiltinType::U16 => ASType::U16,
            witx::BuiltinType::U32 { lang_ptr_size: true } => ASType::Usize,
            witx::BuiltinType::U32 { lang_ptr_size: false } => ASType::U32,
            witx::BuiltinType::U64 => ASType::U64,
            witx::BuiltinType::S8 => ASType::I8,
            witx::BuiltinType::S16 => ASType::I16,
            witx::BuiltinType::S32 => ASType::I32,
            witx::BuiltinType::S64 => ASType::I64,
            witx::BuiltinType::F32 => ASType::F32,
            witx::BuiltinType::F64 => ASType::F64,
            witx::BuiltinType::Char => ASType::Char
        }
    }
}

impl From<&witx::Variant> for ASType {
    fn from(_witx: &witx::Variant) -> Self {
        ASType::Variant(None)
    }
}

impl From<&witx::IntRepr> for ASType {
    fn from(witx: &witx::IntRepr) -> Self {
        match witx {
            witx::IntRepr::U8 => ASType::U8,
            witx::IntRepr::U16 => ASType::U16,
            witx::IntRepr::U32 => ASType::U32,
            witx::IntRepr::U64 => ASType::U64
        } 
    }
}

impl From<&witx::Constant> for ASType {
    fn from(witx: &witx::Constant) -> Self {
        ASType::Alias(witx.ty.as_str().to_string())
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

impl From<&witx::TypeRef> for ASType {
    fn from(witx: &witx::TypeRef) -> Self {
        let type_name = witx.type_name();
        match witx.type_().as_ref() {
            witx::Type::Builtin(x) => ASType::from(x).name(type_name),
            x @ witx::Type::List(_)
            | x @ witx::Type::Pointer(_)
            | x @ witx::Type::ConstPointer(_) => ASType::from(x).name(type_name),
            _ => ASType::Alias(type_name),
        }
    }
}

impl From<&witx::Type> for ASType {
    fn from(witx: &witx::Type) -> Self {
        match witx {
            witx::Type::Builtin(x) => x.into(),
            witx::Type::ConstPointer(x) => ASType::Ptr(Box::new(x.into())),
            witx::Type::Pointer(x) => ASType::MutPtr(Box::new(x.into())),
            witx::Type::Handle(x) => x.into(),
            witx::Type::List(x) => ASType::List(Box::new(x.into())),
            witx::Type::Variant(x) => x.into(),
            witx::Type::Record(_) => ASType::Record(None),
        }
    }
}
