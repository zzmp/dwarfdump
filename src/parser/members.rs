use super::*;

impl<'file, Endian: gimli::Endianity> Parser<'file, Endian> {
    pub fn parse_members(&self, entry: &DIE<Endian>) -> Parameters {
        let mut members = Parameters::new();

        if entry.has_children() {
            let mut children = self.cursor_at_offset(entry.offset());

            {
                let (_, child) = children.next_dfs().expect("starting DIE").expect("checking DIE");
                if child.tag() == gimli::DW_TAG_member {
                    println!("MEMBER {:?} {:?} {:?}", child.offset(), self.parse_type_offset(child), entry.offset());
                    let member = if self.parse_type_offset(child) == Some(entry.offset()) {
                        Parameter::circular()
                    } else {
                        self.parse_parameter(child)
                    };
                    members.push(member);
                }
            }

            while let Some(child) = children.next_sibling().expect("advancing DIE") {
                if child.tag() == gimli::DW_TAG_member {
                    println!("MEMBER {:?} {:?} {:?}", child.offset(), self.parse_type_offset(child), entry.offset());
                    let member = if self.parse_type_offset(child) == Some(entry.offset()) {
                        Parameter::circular()
                    } else {
                        self.parse_parameter(child)
                    };
                    members.push(member);
                }
            }
        }

        members 
    }
}
