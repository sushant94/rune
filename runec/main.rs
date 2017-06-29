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
use rune::context::rune_ctx::{RuneContext, RInitialState};
use console::Console;
use rune::explorer::command::Command;

static USAGE: &'static str = "
runec. Interactive console for rune.

Usage:
  runec FILE
  runec (-h | --help)

Options:
  -h --help                              Show this screen.
";

#[derive(Debug, Clone, RustcDecodable)]
struct Args {
    flag_help: bool,
    arg_FILE: Option<String>,
}

fn main() {
    let args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.parse())
                      .unwrap_or_else(|e| e.exit());

    if args.get_bool("-h") {
        println!("{}", USAGE);
        exit(0);
    }

    let mut stream = R2::new(Some(args.get_str("FILE"))).expect("Unable to spawn r2");
    stream.init();

    let mut c: Console = Default::default();
    let mut is: RInitialState = RInitialState::new(); 

    loop {
        match c.read_command()[0] {
            Command::SetContext((ref key, ref val)) => {
                if val.is_symbolic() {
                    is.add_sym(key.clone());
                }
            },
            _ => break,
        }
    }

    /* let ctx = utils::new_ctx(args.flag_start, Some(sym_vars), Some(const_vars));
    let mut explorer = InteractiveExplorer::new();
    explorer.bp = breakpoints;

    let mut rune = Rune::new(ctx, explorer, stream);
    rune.run().expect("Rune Error:");
    */
}
