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
            let mut children = self.cursor_at_offset(entry.offset());

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

    fn parse_parameter(&self, entry: &DIE<Endian>) -> Parameter {
        let declarator = self.parse_name(entry);
        let mut specifier = None;
        let mut modifiers = Modifiers::new();
        let mut type_offset = self.parse_type_offset(entry);

        while let Some(offset) = type_offset {
            let cursor = self.cursor_at_offset(offset);
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
            unit_offset: self.unit.offset().0,
            type_offset: type_offset.map(|o| o.0),
        }
    }
}
