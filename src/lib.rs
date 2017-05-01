extern crate object;

mod parser;
use parser::*;

use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write;

use object::Object;

#[derive(Debug)]
#[derive(Clone)]
pub enum Modifier {
    Pointer,
    Reference,
    Const,
    Volatile,
    Restrict
}
pub type Modifiers = Vec<Modifier>;

#[derive(Debug)]
#[derive(Clone)]
pub enum TypeValue {
    Base,
    Enum,
    Subroutine(Box<Subprogram>),
    TypeDef(Box<Type>),
    Array(Parameters),
    Union(Parameters),
    Struct(Parameters),
    Circular
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Type {
    pub name: String,
    pub modifiers: Modifiers,
    pub value: TypeValue
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Parameter {
    pub declarator: Option<String>,
    pub specifier: Type
}
pub type Parameters = Vec<Parameter>;

#[derive(Debug)]
#[derive(Clone)]
pub struct Subprogram {
    pub declarator: Parameter,
    pub parameters: Parameters
}

pub struct Symbols {
    pub subprograms: BTreeMap<String, Subprogram>
}

impl Type {
    fn format(&self, f: &mut fmt::Formatter, declarator: Option<&str>) -> fmt::Result {
       match self.value {
            TypeValue::Subroutine(ref subprogram) => {
                let mut subprogram = subprogram.clone();
                subprogram.declarator.declarator = Some(match declarator {
                    Some(ref declarator) => format!("(*{})", declarator),
                    None => format!("(*)")
                });
                write!(f, "{}", subprogram)
            },
            _ => {
                let typevalue = match self.value {
                    TypeValue::Base => "base",
                    TypeValue::Enum => "enum",
                    TypeValue::Subroutine(_) => unreachable!(),
                    TypeValue::TypeDef(_) => "typedef",
                    TypeValue::Array(_) => "array",
                    TypeValue::Union(_) => "union",
                    TypeValue::Struct(_) => "struct",
                    TypeValue::Circular => unreachable!()
                };

                let specifier = self.modifiers.iter().fold(self.name.clone(), |mut s, m| {
                    match m {
                        &Modifier::Pointer => { s += "*"; }
                        &Modifier::Reference => { s += "&"; },
                        &Modifier::Const => { s += " const"; },
                        &Modifier::Volatile => { s += " volatile"; },
                        &Modifier::Restrict => { s += " restrict"; }
                    }
                    s
                });
                match declarator {
                    Some(ref declarator) => write!(f, "{} {} {}", typevalue, specifier, declarator),
                    None => write!(f, "{} {}", typevalue, specifier)
                }
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.format(f, None)
    }
}

impl Parameter {
    fn circular() -> Self {
        Parameter {
            declarator: None,
            specifier: Type {
                name: String::from("circular"),
                modifiers: Vec::new(),
                value: TypeValue::Circular
            }
        }
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.specifier.format(f, self.declarator.as_ref().map(|s| s.as_str()))
    }
}

impl fmt::Display for Subprogram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Vec cannot impl fmt::Display, so it is done here
        let parameters = self.parameters.iter().fold(String::new(), |mut s, p| {
            let _ = if s.is_empty() {
                write!(s, "{}", p)
            } else {
                write!(s, ", {}", p)
            };
            s
        });

        write!(f, "{}({})", self.declarator, parameters)
    }
}

impl Symbols {
    pub fn from(file: object::File) -> Symbols {
        if file.is_little_endian() {
            parse::<LittleEndian>(file)
        } else {
            parse::<BigEndian>(file)
        }
    }

    fn new() -> Symbols {
        Symbols {
            subprograms: BTreeMap::new()
        }
    }
}
