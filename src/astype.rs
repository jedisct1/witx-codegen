use std::rc::Rc;
use witx::Layout as _;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASAlias {
    pub name: String,
    pub type_: Rc<ASType>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASStructMember {
    pub name: String,
    pub offset: usize,
    pub type_: Rc<ASType>,
    pub padding: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASEnumChoice {
    pub name: String,
    pub value: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASEnum {
    pub repr: Rc<ASType>,
    pub choices: Vec<ASEnumChoice>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASUnionMember {
    pub name: String,
    pub type_: Rc<ASType>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASUnion {
    pub tag_repr: Rc<ASType>,
    pub members: Vec<ASUnionMember>,
    pub member_offset: usize,
    pub padding_after_tag: usize,
    pub max_member_size: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASTupleMember {
    pub type_: Rc<ASType>,
    pub offset: usize,
    pub padding: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASOption {
    pub tag_repr: Rc<ASType>,
    pub type_: Rc<ASType>,
    pub offset: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASResult {
    pub tag_repr: Rc<ASType>,
    pub error_type: Rc<ASType>,
    pub ok_type: Rc<ASType>,
    pub result_offset: usize,
    pub padding_after_tag: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASConstant {
    pub name: String,
    pub value: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ASConstants {
    pub repr: Rc<ASType>,
    pub constants: Vec<ASConstant>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ASType {
    Void,
    Alias(ASAlias),
    Bool,
    Char8,
    Char32,
    USize,
    F32,
    F64,
    S8,
    S16,
    S32,
    S64,
    U8,
    U16,
    U32,
    U64,
    Constants(ASConstants),
    Result(ASResult),
    Option(ASOption),
    Handle(String),
    Enum(ASEnum),
    Tuple(Vec<ASTupleMember>),
    ConstPtr(Rc<ASType>),
    MutPtr(Rc<ASType>),
    Union(ASUnion),
    Struct(Vec<ASStructMember>),
    Slice(Rc<ASType>),
    String(Rc<ASType>),
    ReadBuffer(Rc<ASType>),
    WriteBuffer(Rc<ASType>),
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
    fn from(witx_builtin: &witx::BuiltinType) -> Self {
        match witx_builtin {
            witx::BuiltinType::Char => return ASType::Char32,
            witx::BuiltinType::F32 => return ASType::F32,
            witx::BuiltinType::F64 => return ASType::F64,
            witx::BuiltinType::S8 => return ASType::S8,
            witx::BuiltinType::S16 => return ASType::S16,
            witx::BuiltinType::S32 => return ASType::S32,
            witx::BuiltinType::S64 => return ASType::S64,

            witx::BuiltinType::U8 { lang_c_char: false } => return ASType::U8,
            witx::BuiltinType::U8 { lang_c_char: true } => return ASType::Char8,
            witx::BuiltinType::U16 => return ASType::U16,
            witx::BuiltinType::U32 {
                lang_ptr_size: false,
            } => return ASType::U32,
            witx::BuiltinType::U32 {
                lang_ptr_size: true,
            } => return ASType::USize,
            witx::BuiltinType::U64 => return ASType::U64,
        }
    }
}

impl From<&witx::Type> for ASType {
    fn from(type_witx: &witx::Type) -> Self {
        match type_witx {
            witx::Type::Builtin(witx_builtin) => ASType::from(witx_builtin),
            witx::Type::ConstPointer(constptr_tref) => {
                let pointee = ASType::from(constptr_tref);
                return ASType::ConstPtr(Rc::new(pointee));
            }
            witx::Type::Pointer(constptr_tref) => {
                let pointee = ASType::from(constptr_tref);
                return ASType::MutPtr(Rc::new(pointee));
            }
            witx::Type::Handle(handle_data_type) => {
                // data type doesn't seem to be used for anything
                let resource_name = handle_data_type.resource_id.name.as_str().to_string();
                ASType::Handle(resource_name)
            }
            witx::Type::Record(record) if record.is_tuple() =>
            // Tuple
            {
                let mut tuple_members = vec![];
                let layout_witx = &record.member_layout(true);
                for member_witx in layout_witx {
                    let member_tref = &member_witx.member.tref;
                    let member_offset = member_witx.offset;
                    let member = ASTupleMember {
                        offset: member_offset,
                        type_: Rc::new(ASType::from(member_tref)),
                        padding: 0,
                    };
                    tuple_members.push(member);
                }
                // Perform a second pass to compute padding between members
                let n = if layout_witx.is_empty() {
                    0
                } else {
                    layout_witx.len() - 1
                };
                for (i, member_witx) in layout_witx.iter().enumerate().take(n) {
                    let member_tref = &member_witx.member.tref;
                    let member_size = member_tref.mem_size(true);
                    let member_padding =
                        layout_witx[i + 1].offset - member_witx.offset - member_size;
                    tuple_members[i].padding = member_padding;
                }
                return ASType::Tuple(tuple_members);
            }
            witx::Type::Record(record) if record.bitflags_repr().is_none() =>
            // Struct
            {
                let mut struct_members = vec![];
                let layout_witx = &record.member_layout(true);
                for member_witx in layout_witx {
                    let member_name = member_witx.member.name.as_str().to_string();
                    let member_tref = &member_witx.member.tref;
                    let member_offset = member_witx.offset;
                    let member = ASStructMember {
                        name: member_name,
                        offset: member_offset,
                        type_: Rc::new(ASType::from(member_tref)),
                        padding: 0,
                    };
                    struct_members.push(member);
                }
                // Perform a second pass to compute padding between members
                let n = if layout_witx.is_empty() {
                    0
                } else {
                    layout_witx.len() - 1
                };
                for (i, member_witx) in layout_witx.iter().enumerate().take(n) {
                    let member_tref = &member_witx.member.tref;
                    let member_size = member_tref.mem_size(true);
                    let member_padding =
                        layout_witx[i + 1].offset - member_witx.offset - member_size;
                    struct_members[i].padding = member_padding;
                }
                return ASType::Struct(struct_members);
            }
            witx::Type::Record(record) if record.bitflags_repr().is_some() =>
            // Constants
            {
                let mut constants = vec![];
                let constants_repr = ASType::from(record.bitflags_repr().unwrap());
                for (idx, contants_witx) in record.member_layout(true).iter().enumerate() {
                    let constant_name = contants_witx.member.name.as_str().to_string();
                    let constant = ASConstant {
                        name: constant_name,
                        value: 1u64 << idx,
                    };
                    constants.push(constant);
                }
                return ASType::Constants(ASConstants {
                    repr: Rc::new(constants_repr),
                    constants,
                });
            }
            witx::Type::Record(record) => {
                dbg!(record);
                dbg!(record.bitflags_repr());
                unreachable!()
            }
            witx::Type::Variant(variant)
                if (variant.is_enum() || variant.is_bool())
                    && variant.as_expected().is_none()
                    && variant.as_option().is_none() =>
            // Enum
            {
                let enum_repr = ASType::from(variant.tag_repr);
                let mut choices = vec![];
                for (idx, choice_witx) in variant.cases.iter().enumerate() {
                    let choice_name = choice_witx.name.as_str().to_string();
                    let choice = ASEnumChoice {
                        name: choice_name,
                        value: idx,
                    };
                    choices.push(choice);
                }
                // WITX exposes booleans as enums
                if choices.len() == 2
                    && choices[0].name == "false"
                    && choices[0].value == 0
                    && choices[1].name == "true"
                    && choices[1].value == 1
                {
                    ASType::Bool
                } else {
                    ASType::Enum(ASEnum {
                        repr: Rc::new(enum_repr),
                        choices,
                    })
                }
            }
            witx::Type::Variant(variant)
                if variant.as_expected().is_none() && variant.as_option().is_some() =>
            // Option
            {
                let tag_repr = ASType::from(variant.tag_repr);
                let option_offset = variant.payload_offset(true);
                assert_eq!(variant.cases.len(), 1);
                let option_tref = &variant.cases[0].tref;
                let option_type = match &option_tref {
                    None => ASType::Void,
                    Some(type_witx) => ASType::from(type_witx),
                };
                ASType::Option(ASOption {
                    tag_repr: Rc::new(tag_repr),
                    offset: option_offset,
                    type_: Rc::new(option_type),
                })
            }
            witx::Type::Variant(variant)
                if variant.as_expected().is_some() && variant.as_option().is_none() =>
            // Result
            {
                let tag_repr = ASType::from(variant.tag_repr);
                let result_offset = variant.payload_offset(true);
                assert_eq!(variant.cases.len(), 2);
                assert_eq!(variant.cases[0].name, "ok");
                assert_eq!(variant.cases[1].name, "err");
                let ok_tref = &variant.cases[0].tref;
                let ok_type = match &ok_tref {
                    None => ASType::Void,
                    Some(type_witx) => ASType::from(type_witx),
                };
                let error_tref = &variant.cases[1].tref;
                let error_type = match &error_tref {
                    None => ASType::Void,
                    Some(type_witx) => ASType::from(type_witx),
                };
                let full_size = variant.mem_size(true);
                let tag_size = variant.tag_repr.mem_size(true);
                let padding_after_tag = full_size - tag_size;
                ASType::Result(ASResult {
                    tag_repr: Rc::new(tag_repr),
                    result_offset: result_offset,
                    padding_after_tag,
                    error_type: Rc::new(error_type),
                    ok_type: Rc::new(ok_type),
                })
            }
            witx::Type::Variant(variant) =>
            // Tagged Union
            {
                let tag_repr = ASType::from(variant.tag_repr);
                let member_offset = variant.payload_offset(true);
                let mut members = vec![];
                for member_witx in &variant.cases {
                    let member_name = member_witx.name.as_str().to_string();
                    let member_type = match member_witx.tref.as_ref() {
                        None => ASType::Void,
                        Some(type_witx) => ASType::from(type_witx),
                    };
                    let member = ASUnionMember {
                        name: member_name,
                        type_: Rc::new(member_type),
                    };
                    members.push(member);
                }
                let full_size = variant.mem_size(true);
                let tag_size = variant.tag_repr.mem_size(true);
                let padding_after_tag = full_size - tag_size;
                let max_member_size = full_size - member_offset;
                ASType::Union(ASUnion {
                    tag_repr: Rc::new(tag_repr),
                    members,
                    member_offset,
                    padding_after_tag,
                    max_member_size,
                })
            }
            witx::Type::List(items_tref) => {
                let elements_type = ASType::from(items_tref);
                match elements_type {
                    // The "string" keyword in WITX returns a Char32, even if the actual encoding is expected to be UTF-8
                    ASType::Char32 | ASType::Char8 => ASType::String(Rc::new(ASType::Char8)),
                    _ => ASType::Slice(Rc::new(elements_type)),
                }
            }
            witx::Type::Buffer(buffer) if buffer.out => {
                let elements_type = ASType::from(&buffer.tref);
                ASType::WriteBuffer(Rc::new(elements_type))
            }
            witx::Type::Buffer(buffer) => {
                let elements_typ = ASType::from(&buffer.tref);
                ASType::ReadBuffer(Rc::new(elements_typ))
            }
        }
    }
}

impl From<&witx::TypeRef> for ASType {
    fn from(witx_tref: &witx::TypeRef) -> Self {
        match witx_tref {
            witx::TypeRef::Value(type_witx) => ASType::from(type_witx.as_ref()),
            witx::TypeRef::Name(alias_witx) => {
                let alias_witx = alias_witx.as_ref();
                let alias_name = alias_witx.name.as_str().to_string();
                let alias_target = ASType::from(&alias_witx.tref);
                ASType::Alias(ASAlias {
                    name: alias_name,
                    type_: Rc::new(alias_target),
                })
            }
        }
    }
}

pub struct ASTypeDecomposed {
    pub name: String,
    pub type_: Rc<ASType>,
}

impl ASType {
    pub fn leaf(&self) -> &ASType {
        if let ASType::Alias(alias) = self {
            alias.type_.as_ref()
        } else {
            self
        }
    }

    pub fn decompose(&self, name: &str, as_mut_pointers: bool) -> Vec<ASTypeDecomposed> {
        let leaf = self.leaf();
        let mut decomposed = match leaf {
            ASType::Void => vec![],
            ASType::ReadBuffer(elements_type)
            | ASType::WriteBuffer(elements_type)
            | ASType::Slice(elements_type)
            | ASType::String(elements_type) => {
                let ptr_name = format!("{}_ptr", name);
                let len_name = format!("{}_len", name);
                let ptr_type = if let ASType::ReadBuffer(_) = leaf {
                    ASType::ConstPtr(elements_type.clone())
                } else {
                    ASType::MutPtr(elements_type.clone())
                };
                let ptr_element = ASTypeDecomposed {
                    name: ptr_name,
                    type_: Rc::new(ptr_type),
                };
                let len_element = ASTypeDecomposed {
                    name: len_name,
                    type_: Rc::new(ASType::MutPtr(elements_type.clone())),
                };
                vec![ptr_element, len_element]
            }
            _ => {
                vec![ASTypeDecomposed {
                    name: name.to_string(),
                    type_: Rc::new(self.clone()),
                }]
            }
        };
        if as_mut_pointers {
            for part in decomposed.iter_mut() {
                let type_ = part.type_.clone();
                part.type_ = Rc::new(ASType::MutPtr(type_));
            }
        }
        decomposed
    }
}
