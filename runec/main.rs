//! runec - Rune Console.
//!
//! Interactive shell that uses rune for user guided symbolic execution and
//! binary reasoning.

extern crate rune;
extern crate libsmt;
extern crate docopt;
extern crate rustc_serialize;
extern crate r2pipe;

mod interact;
mod console;

use std::process::exit;
use docopt::Docopt;
use std::collections::HashMap;
use rune::context::utils;
use rune::explorer::explorer::PathExplorer;
use rune::engine::rune::Rune;
use rune::engine::engine::Engine;
use interact::InteractiveExplorer;
use r2pipe::r2::R2;
use rune::stream::InstructionStream;

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
  -b --break=<bp_list>                   Set breakpoints at addresses.
  -h --help                              Show this screen.
";

#[derive(Debug, Clone, RustcDecodable)]
struct Args {
    flag_help: bool,
    flag_break: Option<String>,
    flag_sym: Option<String>,
    flag_const: Option<String>,
    flag_start: Option<u64>,
    flag_end: Option<u64>,
    arg_file: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

    if args.flag_help {
        println!("{}", USAGE);
        exit(0);
    }

    let sym_vars = args.flag_sym
                       .unwrap_or_default()
                       .split(',')
                       .map(|x| x.to_owned())
                       .collect::<Vec<String>>();

    let const_vars = args.flag_const
                         .unwrap_or_default()
                         .split(',')
                         .map(|x_| {
                             let x = x_.to_owned();
                             let mut substr = x.split(':').take(2);
                             (substr.next().unwrap().to_owned(),
                              {
                                 let v_str = substr.next().unwrap().to_owned();
                                 if v_str.starts_with("0x") {
                                     u64::from_str_radix(&v_str[2..], 16)
                                         .expect("Invalid integer in base16")
                                 } else {
                                     u64::from_str_radix(&v_str, 10)
                                         .expect("Invalid integer in base10")
                                 }
                             })
                         })
                         .collect::<HashMap<_, _>>();

    let mut breakpoints = args.flag_break
                          .unwrap_or_default()
                          .split(',')
                          .map(|x| {
                              let b = x.to_owned();
                              if b.starts_with("0x") {
                                  u64::from_str_radix(&b[2..], 16).expect("Invalid base16 integer")
                              } else {
                                  u64::from_str_radix(&b, 10).expect("Invalid base10 integer")
                              }
                          })
                          .collect::<Vec<_>>();

    if let Some(addr) = args.flag_end {
        breakpoints.push(addr);
    }

    let mut ctx = utils::new_ctx(args.flag_start, Some(sym_vars), Some(const_vars));
    let mut explorer = InteractiveExplorer::new();
    explorer.bp = breakpoints;
    let mut stream = R2::new(Some(args.arg_file)).expect("Unable to spawn r2");
    stream.init();

    let mut rune = Rune::new(ctx, explorer, stream);
    rune.run().expect("Rune Error:");
}
