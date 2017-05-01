use super::*;

impl<'file, Endian: gimli::Endianity> Parser<'file, Endian> {
    pub fn parse_subprogram(&self, entry: &DIE<Endian>) -> Subprogram {
        Subprogram {
            declarator: self.parse_parameter(entry),
            parameters: self.parse_parameters(entry)
        }
    }

    pub fn parse_parameter(&self, entry: &DIE<Endian>) -> Parameter {
        Parameter {
            declarator: self.parse_name(entry),
            specifier: self.parse_type(self.parse_type_offset(entry)) 
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
}
