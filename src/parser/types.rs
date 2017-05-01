use super::*;

impl<'file, Endian: gimli::Endianity> Parser<'file, Endian> {
    pub fn parse_type(&self, mut type_offset: Option<gimli::UnitOffset>) -> Type {
        let mut name = String::from("void");
        let mut modifiers = Vec::new();
        let mut value = TypeValue::Base;

        if type_offset.is_none() {
            return Type { name: name, modifiers: modifiers, value: value };
        }

        if self.types.contains_key(&type_offset.unwrap().0) {
            return (*self.types.get(&type_offset.unwrap().0).unwrap()).clone();
        }

        while let Some(offset) = type_offset {
            let cursor = self.cursor_at_offset(offset);
            let entry = cursor.current().expect("getting entry");
            let tag = self.parse_tag(entry);

            if let Tag::Modifier(ref modifier) = tag {
                modifiers.push(modifier.clone());
                type_offset = self.parse_type_offset(entry);
                continue;
            }

            match tag {
                Tag::Modifier(_) => unreachable!(),
                Tag::Subroutine => {
                    modifiers.pop(); // all subroutines are pointers
                    let subprogram = self.parse_subprogram(entry);
                    value = TypeValue::Subroutine(Box::new(subprogram));
                },
                Tag::BaseType => {
                    value = TypeValue::Base;
                },
                Tag::TypeDef => {
                    let type_offset = self.parse_type_offset(entry);
                    let typedef = self.parse_type(type_offset);
                    value = TypeValue::TypeDef(Box::new(typedef));
                },
                Tag::Enum => {
                    value = TypeValue::Enum;
                },
                Tag::Array => {
                    name = self.parse_name(entry).unwrap_or(String::from("void"));
                    println!("ARRAY {:?}", name);
                    value = TypeValue::Array(self.parse_members(entry));
                },
                Tag::Struct => {
                    name = self.parse_name(entry).unwrap_or(String::from("void"));
                    println!("STRUCT {:?}", name);
                    value = TypeValue::Struct(self.parse_members(entry));
                },
                Tag::Union => {
                    name = self.parse_name(entry).unwrap_or(String::from("void"));
                    println!("UNION {:?}", name);
                    value = TypeValue::Union(self.parse_members(entry));
                },
                _ => unreachable!()
            }
            name = self.parse_name(entry).unwrap_or(String::from("void"));
            println!("{:?} {:?}", name, value);
            break;
        }

        Type {
            name: name,
            modifiers: modifiers,
            value: value
        }
    }
}
