extern crate fallible_iterator;
extern crate gimli;

mod shared;
mod subprogram;
mod types;
mod members;

use std::collections::HashMap;
use std::option::Option;

use object::Object;

use self::fallible_iterator::FallibleIterator;
use self::gimli::CompilationUnitHeader as Unit;
use self::gimli::DebuggingInformationEntry as DIE;
pub use self::gimli::LittleEndian as LittleEndian;
pub use self::gimli::BigEndian as BigEndian;

use super::*;
use self::shared::*;

struct Parser<'file, Endian: 'file + gimli::Endianity> {
    str: &'file gimli::DebugStr<'file, Endian>,
    unit: &'file Unit<'file, Endian>,
    abbrev: &'file gimli::Abbreviations,
    types: HashMap<usize, Type>
}

pub fn parse<Endian: gimli::Endianity>(file: object::File) -> Symbols {
    // read file sections
    let debug_abbrev = file.get_section(".debug_abbrev").unwrap_or(&[]);
    let debug_str = file.get_section(".debug_str").unwrap_or(&[]);
    let debug_info = file.get_section(".debug_info").unwrap_or(&[]);

    // prepare file state
    let abbrev = gimli::DebugAbbrev::<Endian>::new(debug_abbrev);
    let str = gimli::DebugStr::<Endian>::new(debug_str);

    gimli::DebugInfo::<Endian>::new(debug_info).units()
        .fold(Symbols::new(), |mut symbols, unit| {
            // prepare unit state
            let abbrev = unit.abbreviations(abbrev).expect("parsing abbrev");
            let parser = Parser {
                str: &str,
                abbrev: &abbrev,
                unit: &unit,
                types: HashMap::new()
            };

            // parse subprograms
            let mut entries = unit.entries(&abbrev);
            while let Some((_, entry)) = entries.next_dfs().expect("advancing DIE") {
                if entry.tag() == gimli::DW_TAG_subprogram {
                    if entry.attr(gimli::DW_AT_external).expect("reading external").is_some() &&
                        entry.attr(gimli::DW_AT_prototyped).expect("reading prototyped").is_some() {
                        let subprogram = parser.parse_subprogram(&entry);
                        let symbol = subprogram.declarator.declarator.as_ref().expect("reading declarator").clone();
                        println!("ZZMP {}", symbol);
                        symbols.subprograms.insert(symbol, subprogram);
                    }
                }
            }

            symbols
        })
        .unwrap_or(Symbols::new())
}
