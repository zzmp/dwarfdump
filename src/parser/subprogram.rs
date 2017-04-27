use super::*;

impl<'file, Endian: gimli::Endianity> Parser<'file, Endian> {
    pub fn parse_subprogram(&self, entry: &DIE<Endian>) -> Subprogram {
        Subprogram {
            declarator: self.parse_parameter(entry),
            parameters: self.parse_parameters(entry)
        }
    }

    fn parse_parameters(&self, entry: &DIE<Endian>) -> Parameters {
        let mut parameters = Parameters::new();

        if entry.has_children() {
            let mut children = self.unit.entries_at_offset(self.abbrev, entry.offset()).expect("setting DIE");
            let _ = children.next_dfs();

            {
                let (_, child) = children.next_dfs().expect("starting DIE").expect("checking DIE");
                if child.tag() == gimli::DW_TAG_formal_parameter {
                    parameters.push(self.parse_parameter(child));
                }
            }

            while let Some(child) = children.next_sibling().expect("advancing DIE") {
                if child.tag() == gimli::DW_TAG_formal_parameter {
                    parameters.push(self.parse_parameter(child));
                }
            }
        }

        parameters
    }

    fn parse_name(&self, entry: &DIE<Endian>) -> Option<String> {
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

    fn parse_parameter(&self, entry: &DIE<Endian>) -> Parameter {
        let declarator = self.parse_name(entry);
        let mut specifier = None;
        let mut modifiers = Modifiers::new();
        let mut type_offset = self.parse_type_offset(entry);

        while let Some(offset) = type_offset {
            let mut cursor = self.unit.entries_at_offset(self.abbrev, offset).expect("setting DIE");
            let _ = cursor.next_dfs();
            let entry = cursor.current().expect("checking DIE");
            type_offset = self.parse_type_offset(entry);

            match self.parse_tag(entry) {
                Tag::Subroutine => {
                    modifiers.pop(); // all subroutines are pointers
                    let subprogram = self.parse_subprogram(entry);
                    modifiers.push(Modifier::Subroutine(subprogram));
                },
                Tag::Modifier(modifier) => { modifiers.push(modifier); },
                // if it's not a modifier, it's a named type
                _ => {
                    specifier = self.parse_name(entry);
                    type_offset = Some(offset); // reset the offset to point to this
                    break; // don't descend - that is for type parsing
                }
            }
        }

        Parameter {
            declarator: declarator,
            specifier: specifier.unwrap_or(String::from("void")),
            modifiers: modifiers,
            offset: type_offset.map(|o| o.0)
        }
    }

    fn parse_type_offset(&self, entry: &DIE<Endian>) -> Option<gimli::UnitOffset> {
        match entry.attr(gimli::DW_AT_type).expect("parsing type") {
            Some(attr) => match attr.value() {
                gimli::AttributeValue::UnitRef(offset) => Some(offset),
                _ => unreachable!()
            },
            None => None
        }
    }

    fn parse_tag(&self, entry: &DIE<Endian>) -> Tag {
        match entry.tag() {
            gimli::DW_TAG_base_type => Tag::BaseType,
            gimli::DW_TAG_pointer_type => Tag::Modifier(Modifier::Pointer),
            gimli::DW_TAG_reference_type => Tag::Modifier(Modifier::Reference),
            gimli::DW_TAG_const_type => Tag::Modifier(Modifier::Const),
            gimli::DW_TAG_volatile_type => Tag::Modifier(Modifier::Volatile),
            gimli::DW_TAG_restrict_type => Tag::Modifier(Modifier::Restrict),
            gimli::DW_TAG_subroutine_type => Tag::Subroutine,
            tag => Tag::Other(tag)
        }
    }
}
