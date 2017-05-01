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

            let mut has_ptr = false;
            if let Tag::Modifier(ref modifier) = tag {
                has_ptr = match modifier {
                    &Modifier::Pointer => true,
                    _ => has_ptr
                };
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
                Tag::Array | Tag::Struct | Tag::Union => {
                    println!("ZZMP {}", has_ptr);
                    if has_ptr {
                        value = TypeValue::Pointer;
                    } else {
                        match tag {
                            Tag::Array => {
                                println!("ARRAY {}", self.parse_name(entry).unwrap_or(String::from("huh")));
                                value = TypeValue::Array(self.parse_members(entry));
                            },
                            Tag::Struct => {
                                println!("STRUCT {}", self.parse_name(entry).unwrap_or(String::from("huh")));
                                value = TypeValue::Struct(self.parse_members(entry));
                            },
                            Tag::Union => {
                                println!("UNION {}", self.parse_name(entry).unwrap_or(String::from("huh")));
                                value = TypeValue::Union(self.parse_members(entry));
                            },
                            _ => unreachable!()
                        }
                    }
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
