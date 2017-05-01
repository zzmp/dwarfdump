use super::*;

pub enum Tag {
    BaseType,
    Modifier(Modifier),
    Subroutine,
    TypeDef,
    Enum,
    Struct,
    Union,
    Array,
    Subrange,
    Other(gimli::DwTag)
}

impl<'file, Endian: gimli::Endianity> Parser<'file, Endian> {
    pub fn cursor_at_offset(&self, offset: gimli::UnitOffset) -> gimli::EntriesCursor<Endian> {
        let mut cursor = self.unit.entries_at_offset(self.abbrev, offset).expect("setting DIE");
        let _ = cursor.next_dfs();
        cursor
    }

    pub fn parse_name(&self, entry: &DIE<Endian>) -> Option<String> {
        match entry.attr(gimli::DW_AT_name).expect("reading name") {
            Some(name) => {
                let name = name.string_value(&self.str).expect("looking up name")
                    .to_str().expect("validating name");
                let name = String::from(name);
                Some(name)
            },
            None => None
        }
    }

    pub fn parse_type_offset(&self, entry: &DIE<Endian>) -> Option<gimli::UnitOffset> {
        match entry.attr(gimli::DW_AT_type).expect("parsing type") {
            Some(attr) => match attr.value() {
                gimli::AttributeValue::UnitRef(offset) => Some(offset),
                _ => unreachable!()
            },
            None => None
        }
    }

    pub fn parse_tag(&self, entry: &DIE<Endian>) -> Tag {
        match entry.tag() {
            gimli::DW_TAG_base_type => Tag::BaseType,
            gimli::DW_TAG_pointer_type => Tag::Modifier(Modifier::Pointer),
            gimli::DW_TAG_reference_type => Tag::Modifier(Modifier::Reference),
            gimli::DW_TAG_const_type => Tag::Modifier(Modifier::Const),
            gimli::DW_TAG_volatile_type => Tag::Modifier(Modifier::Volatile),
            gimli::DW_TAG_restrict_type => Tag::Modifier(Modifier::Restrict),
            gimli::DW_TAG_subroutine_type => Tag::Subroutine,
            gimli::DW_TAG_typedef => Tag::TypeDef,
            gimli::DW_TAG_enumeration_type => Tag::Enum,
            gimli::DW_TAG_structure_type => Tag::Struct,
            gimli::DW_TAG_union_type => Tag::Union,
            gimli::DW_TAG_array_type => Tag::Array,
            gimli::DW_TAG_subrange_type => Tag::Subrange,
            tag => Tag::Other(tag)
        }
    }
}
