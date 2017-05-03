extern crate object;

use object::Object;

mod parser;
use parser::{ parse, LittleEndian, BigEndian };

use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write;

pub struct Symbols {
    pub functions: BTreeMap<String, Function>
}

#[derive(Clone)]
pub struct Function {
    pub name: Option<String>,
    pub typed: Typed,
    pub parameters: Parameters
}

#[derive(Clone)]
pub struct Parameter {
    pub name: Option<String>,
    pub typed: Typed
}
pub type Parameters = Vec<Parameter>;
pub type Member = Parameter;
pub type Members = Vec<Member>;

#[derive(Clone)]
pub struct Typed {
    pub name: String,
    pub value: TypedValue,
    pub modifiers: Modifiers
}

#[derive(Clone)]
#[derive(Debug)]
pub enum TypedValue {
    Base,
    Enum,
    Typedef(Box<Typed>),
    Function(Box<Function>),
    Struct(Members),
    Union(Members),
    Array(Box<Typed>, u16),
    Circular
}

#[derive(Clone)]
pub enum Modifier {
    Pointer,
    Reference,
    Const,
    Volatile,
    Restrict
}
pub type Modifiers = Vec<Modifier>;

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
            functions: BTreeMap::new()
        }
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let typeds = self.parameters.iter().fold(format!("\n{:?}", self.typed), |mut s, p| {
            let _ = write!(s, "{:?}", p);
            s
        });
        write!(f, "{}\n---{}\n\n", self, typeds)
    }
}

impl fmt::Display for Function {
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

        match self.name {
            Some(ref name) => write!(f, "{} {}({})", name, self.typed, parameters),
            None => write!(f, "{}({})", self.typed, parameters)
        }
    }
}

impl fmt::Debug for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self.name {
            Some(ref name) => format!("name: {}\t", name),
            None => String::new()
        };
        write!(f, "\n{}{:?}", name, self.typed)
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.typed.value {
            TypedValue::Function(ref function) => {
                let mut function_ptr = (*function).clone();
                function_ptr.name = Some(match self.name {
                    Some(ref name) => format!("(*{})", name),
                    None => String::from("(*)")
                });
                write!(f, "{}", function_ptr)
            },
            _ => {
                match self.name {
                    Some(ref name) => write!(f, "{} {}", self.typed, name),
                    None => write!(f, "{}", self.typed)
                }
            }
        }
    }
}

impl fmt::Debug for Typed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:#?}", self, self.value)
    }
}

impl fmt::Display for Typed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let specifier = self.modifiers.iter().fold(self.name.clone(), |s, m| {
            match m {
                &Modifier::Pointer => s + "*",
                &Modifier::Reference => s + "&",
                &Modifier::Const => s + " const",
                &Modifier::Volatile => s + " volatile",
                &Modifier::Restrict => s + " restrict"
            }
        });
        write!(f, "{}", specifier)
    }
}
