extern crate object;

mod parser;
use parser::*;

use std::collections::{BTreeMap, HashMap};
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
    Restrict,
    Subroutine(Subprogram)
}
pub type Modifiers = Vec<Modifier>;

pub struct Type {
    pub name: String
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Parameter {
    pub modifiers: Modifiers,
    pub specifier: String,
    pub declarator: Option<String>,
    unit_offset: usize,
    type_offset: Option<usize>
}
pub type Parameters = Vec<Parameter>;

#[derive(Debug)]
#[derive(Clone)]
pub struct Subprogram {
    pub declarator: Parameter,
    pub parameters: Parameters
}

pub struct Symbols {
    pub subprograms: BTreeMap<String, Subprogram>,
    types: HashMap<usize, HashMap<usize, Type>>
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.modifiers.first() {
            Some(&Modifier::Subroutine(ref subroutine)) => {
                let mut subroutine = subroutine.clone();
                subroutine.declarator.declarator = Some(match self.declarator {
                    Some(ref declarator) => format!("(*{})", declarator),
                    None => format!("(*)")
                });
                write!(f, "{}", subroutine)
            },
            _ => {
                let specifier = self.modifiers.iter().fold(self.specifier.clone(), |mut s, m| {
                    match m {
                        &Modifier::Pointer => { s += "*"; }
                        &Modifier::Reference => { s += "&"; },
                        &Modifier::Const => { s += " const"; },
                        &Modifier::Volatile => { s += " volatile"; },
                        &Modifier::Restrict => { s += " restrict"; }
                        &Modifier::Subroutine(_) => unreachable!()
                    }
                    s
                });
                match self.declarator {
                    Some(ref declarator) => write!(f, "{} {}", specifier, declarator),
                    None => write!(f, "{}", self.specifier)
                }
            }
        }
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

impl Subprogram {
    fn type_offsets(&self) -> Vec<Option<usize>> {
        self.parameters.iter().fold(vec![self.declarator.type_offset], |mut o, p| {
            o.push(p.type_offset);
            o
        })
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
            subprograms: BTreeMap::new(),
            types: HashMap::new()
        }
    }
}
