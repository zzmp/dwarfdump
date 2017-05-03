extern crate fallible_iterator;
extern crate gimli;

pub use self::gimli::LittleEndian as LittleEndian;
pub use self::gimli::BigEndian as BigEndian;

use self::fallible_iterator::FallibleIterator;
use self::gimli::CompilationUnitHeader as Unit;
use self::gimli::DebuggingInformationEntry as DIE;

use super::*;

use std::collections::{ HashMap, HashSet };

pub struct Parser<'file, Endian: 'file + gimli::Endianity> {
    str: gimli::DebugStr<'file, Endian>,
    unit: Unit<'file, Endian>,
    abbreviations: gimli::Abbreviations,
    offsets: HashSet<usize>,
    typeds: HashMap<usize, Typed>
}

pub fn parse<Endian: gimli::Endianity>(file: object::File) -> Symbols {
    // read file sections
    let debug_abbrev = file.get_section(".debug_abbrev").unwrap_or(&[]);
    let debug_str = file.get_section(".debug_str").unwrap_or(&[]);
    let debug_info = file.get_section(".debug_info").unwrap_or(&[]);

    // prepare file state
    let abbrev = gimli::DebugAbbrev::<Endian>::new(debug_abbrev);
    let str = gimli::DebugStr::<Endian>::new(debug_str);

    gimli::DebugInfo::<Endian>::new(debug_info).units().fold(Symbols::new(), |mut symbols, unit| {
        let abbreviations = unit.abbreviations(abbrev).expect("parsing abbrev");

        // prepare unit parser
        let mut parser = Parser {
            str: str,
            unit: unit,
            abbreviations: abbreviations.clone(),
            offsets: HashSet::new(),
            typeds: HashMap::new()
        };

        // parse entries
        let mut entries = unit.entries(&abbreviations);
        while let Some((_, entry)) = entries.next_dfs().expect("setting cursor") {
            if entry.tag() == gimli::DW_TAG_subprogram {
                if entry.attr(gimli::DW_AT_external).expect("reading external").is_some() &&
                    entry.attr(gimli::DW_AT_prototyped).expect("reading prototyped").is_some() {
                    let function = parser.parse_function(&entry);
                    let symbol = function.name.as_ref().expect("reading name").clone();
                    symbols.functions.insert(symbol, function);
                }
            }
        }

        symbols
    }).expect("parsing units")
}

impl<'file, Endian: gimli::Endianity> Parser<'file, Endian> {
    /// Parse a gimli::DW_TAG_subprogram entry
    fn parse_function(&mut self, entry: &DIE<Endian>) -> Function {
        Function {
            name: self.parse_name(entry),
            typed: self.parse_typed(entry),
            parameters: self.parse_parameters(entry)
        }
    }

    /// Parse a gimli::DW_TAG_subprogram entry's Parameters
    fn parse_parameters(&mut self, entry: &DIE<Endian>) -> Parameters {
        self.parse_children(entry, gimli::DW_TAG_formal_parameter)
    }

    /// Parse a gimli::DW_TAG_formal_parameter entry
    fn parse_parameter(&mut self, entry: &DIE<Endian>) -> Parameter {
        Parameter {
            name: self.parse_name(entry),
            typed: self.parse_typed(entry)
        }
    }

    /// Parse a gimli::DW_TAG_structure_type or union_type's Members
    fn parse_members(&mut self, entry: &DIE<Endian>) -> Members {
        self.parse_children(entry, gimli::DW_TAG_member)
    }

    /// Parse nested entry's of given tag
    fn parse_children(&mut self, entry: &DIE<Endian>, tag: gimli::DwTag) -> Parameters {
        let mut children = Parameters::new();

        if entry.has_children() {
            let unit = self.unit.clone();
            let abbreviations = self.abbreviations.clone();

            let mut cursor = unit.entries_at_offset(&abbreviations, entry.offset()).expect("setting cursor");
            let _ = cursor.next_dfs();

            {
                let (_, child) = cursor.next_dfs().expect("setting cursor").expect("setting DIE");
                if child.tag() == tag {
                    children.push(self.parse_parameter(child));
                }
            }

            while let Some(child) = cursor.next_sibling().expect("setting cursor") {
                if child.tag() == tag {
                    children.push(self.parse_parameter(child));
                }
            }
        }

        children 
    }

    fn parse_dimensionality(&mut self, entry: &DIE<Endian>) -> u16 {
        let mut array_size = 1;

        let unit = self.unit.clone();
        let abbreviations = self.abbreviations.clone();

        let mut cursor = unit.entries_at_offset(&abbreviations, entry.offset()).expect("setting cursor");
        let _ = cursor.next_dfs();

        {
            let (_, child) = cursor.next_dfs().expect("setting cursor").expect("setting DIE");
            assert!(child.tag() == gimli::DW_TAG_subrange_type);
            match child.attr(gimli::DW_AT_count).expect("reading count") {
                Some(count) => array_size *= count.u16_value().expect("getting count"),
                None => {
                    let lbound = match child.attr(gimli::DW_AT_lower_bound).expect("reading lbound") {
                        Some(lbound) => lbound.u16_value().expect("getting lbound"),
                        None => 0
                    };
                    let ubound = child.attr(gimli::DW_AT_upper_bound).expect("reading ubound");
                    if let Some(ubound) = ubound {
                        array_size *= (ubound.u16_value().expect("getting ubound") - lbound) + 1;
                    }
                }
            }
        }

        while let Some(child) = cursor.next_sibling().expect("setting cursor") {
            if child.tag() == gimli::DW_TAG_subrange_type {
                match child.attr(gimli::DW_AT_count).expect("reading count") {
                    Some(count) => array_size *= count.u16_value().expect("getting count"),
                    None => {
                        let lbound = match child.attr(gimli::DW_AT_lower_bound).expect("reading lbound") {
                            Some(lbound) => lbound.u16_value().expect("getting lbound"),
                            None => 0
                        };
                        let ubound = child.attr(gimli::DW_AT_upper_bound).expect("reading ubound");
                        if let Some(ubound) = ubound {
                            array_size *= (ubound.u16_value().expect("getting ubound") - lbound) + 1;
                        }
                    }
                }
            } else {
                break;
            }
        }

        array_size
    }

    /// Parse an entry's name
    fn parse_name(&mut self, entry: &DIE<Endian>) -> Option<String> {
        entry.attr(gimli::DW_AT_name).expect("reading name").map(|name| {
            name.string_value(&self.str).expect("reading str")
                .to_str().expect("validating str")
                .to_string()
        })
    }

    /// Parse a gimli::DW_TAG_subprogram, subroutine_type, formal_parameter, or member's Typed
    fn parse_typed(&mut self, entry: &DIE<Endian>) -> Typed {
        debug_assert!(match entry.tag() {
            gimli::DW_TAG_subprogram | gimli::DW_TAG_subroutine_type |
            gimli::DW_TAG_formal_parameter | gimli::DW_TAG_member => true,
            _ => false
        });

        let offset = entry.attr(gimli::DW_AT_type).expect("reading type")
            .map(|attr| match attr.value() {
                gimli::AttributeValue::UnitRef(offset) => offset,
                _ => unreachable!()
            });

        self.parse_typed_helper(offset)
    }

    /// Parse a type entry's Typed
    fn parse_typed_helper(&mut self, offset: Option<gimli::UnitOffset>) -> Typed {
        let mut name = String::from("void");
        let mut value = TypedValue::Base;
        let mut modifiers = Vec::new();

        if offset.is_none() {
            return Typed {
                name: name,
                value: value,
                modifiers: modifiers
            }
        }

        if let Some(typed) = self.typeds.get(&offset.unwrap().0) {
            return typed.clone();
        }

        let unit = self.unit.clone();
        let abbreviations = self.abbreviations.clone();

        let mut type_offset = offset;
        while let Some(offset) = type_offset {
            let mut cursor = unit.entries_at_offset(&abbreviations, offset).expect("setting DIE");
            let _ = cursor.next_dfs();
            let entry = cursor.current().expect("getting DIE");

            type_offset = entry.attr(gimli::DW_AT_type).expect("reading type").map(|attr| match attr.value() {
                gimli::AttributeValue::UnitRef(offset) => offset,
                _ => unreachable!()
            });

            // modifiers
            if match entry.tag() {
                gimli::DW_TAG_pointer_type => Some(modifiers.push(Modifier::Pointer)),
                gimli::DW_TAG_reference_type => Some(modifiers.push(Modifier::Reference)),
                gimli::DW_TAG_const_type => Some(modifiers.push(Modifier::Const)),
                gimli::DW_TAG_volatile_type => Some(modifiers.push(Modifier::Volatile)),
                gimli::DW_TAG_restrict_type => Some(modifiers.push(Modifier::Restrict)),
                _ => None
            }.is_some() {
                continue;
            }

            // check for recursive types
            if self.offsets.contains(&offset.0) {
                value = TypedValue::Circular;
                break;
            }

            // extended types
            value = match entry.tag() {
                gimli::DW_TAG_base_type => TypedValue::Base,
                gimli::DW_TAG_enumeration_type => TypedValue::Enum,
                // potentially recursive types
                gimli::DW_TAG_typedef | gimli::DW_TAG_subroutine_type |
                gimli::DW_TAG_structure_type | gimli::DW_TAG_union_type |
                gimli::DW_TAG_array_type => {
                    self.offsets.insert(offset.0);
                    match entry.tag() {
                        gimli::DW_TAG_typedef => TypedValue::Typedef(Box::new(self.parse_typed_helper(type_offset))),
                        gimli::DW_TAG_subroutine_type => TypedValue::Function(Box::new(self.parse_function(entry))),
                        gimli::DW_TAG_structure_type => TypedValue::Struct(self.parse_members(entry)),
                        gimli::DW_TAG_union_type => TypedValue::Union(self.parse_members(entry)),
                        gimli::DW_TAG_array_type => TypedValue::Array(
                            Box::new(self.parse_typed_helper(type_offset)), self.parse_dimensionality(entry)),
                        _ => unreachable!()
                    }
                }
                _ => unreachable!()
            };

            name = self.parse_name(entry).unwrap_or(String::from("void"));
            break;
        }

        let typed = Typed {
            name: name,
            value: value,
            modifiers: modifiers
        };
        self.typeds.insert(offset.unwrap().0, typed.clone());
        typed
    }
}
