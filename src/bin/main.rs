extern crate getopts;
extern crate memmap;
extern crate object;
extern crate dwarfdump;

use dwarfdump::Symbols;
use object::Object;

use std::env;
use std::io::Write;
use std::io;
use std::fs;
use std::process;

struct Flags { }

fn print_usage(opts: &getopts::Options) -> ! {
    let brief = format!("Usage: {} <options> <file>...", env::args().next().unwrap());
    write!(&mut io::stderr(), "{}", opts.usage(&brief)).ok();
    process::exit(1);
}

fn main() {
    let opts = getopts::Options::new();

    let matches = match opts.parse(env::args().skip(1)) {
        Ok(m) => m,
        Err(e) => {
            writeln!(&mut io::stderr(), "{:?}\n", e).ok();
            print_usage(&opts);
        }
    };
    if matches.free.is_empty() {
        print_usage(&opts);
    }

    let _ = Flags{
    };

    let mut first_file = true;
    for file_path in &matches.free {
        if matches.free.len() != 1 {
            if !first_file {
                println!("");
            }
            println!("{}:", file_path);

            if first_file {
                first_file = false;
            }
        }

        let file = fs::File::open(&file_path).expect("opening file");
        let file = memmap::Mmap::open(&file, memmap::Protection::Read).expect("mmapping file");
        let file = object::File::parse(unsafe { file.as_slice() }).expect("parsing file");

        let symbols = Symbols::from(file);

        symbols.functions.iter().fold((), |_, (_, v)| {
            println!("{:?}", v);
        });
    }
}
