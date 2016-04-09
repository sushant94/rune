//! runec - Rune Console.
//!
//! Interactive shell that uses rune for user guided symbolic execution and
//! binary reasoning.

extern crate rune;
extern crate libsmt;
extern crate docopt;
extern crate rustc_serialize;

mod interact;
mod console;

use std::process::exit;
use docopt::Docopt;

static USAGE: &'static str = "
runec. Interactive console for rune.
Usage:
  runec [options] [<file>]

Options:
  -e --end=<end_addr>                    Address to end emulation at.
  -s --start=<start_addr>                Address to start emulation at.
  --const=<const_vars>                   Key:Value pairs.
                                         Example: --const=rbp:0x1000,rsp:0x1100
  --sym=<sym_vars>                       Registers/Memory address to be set as symbolic.
                                         Example: --sym=rsi,rdi,0x1000
  -bp --break=<bp_list>                  Set breakpoints at addresses.
  -h --help                              Show this screen.
";

#[derive(Debug, Clone, RustcDecodable)]
struct Args {
    flag_help: bool,
    flag_break: Option<Vec<u64>>,
    flag_sym: Option<Vec<String>>,
    flag_const: Option<Vec<String>>,
    flag_start: Option<u64>,
    flag_end: Option<u64>,
    arg_file: String,
}
    
fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode())
                                       .unwrap_or_else(|e| e.exit());

    if args.flag_help {
        println!("{}", USAGE);
        exit(0);
    }
}
