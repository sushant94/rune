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
use rune::context::utils::{Key, ValType, SAssignment};
use rune::explorer::explorer::PathExplorer;
use rune::engine::rune::Rune;
use rune::engine::engine::Engine;
use interact::InteractiveExplorer;
use r2pipe::r2::R2;
use rune::stream::InstructionStream;
use rune::context::rune_ctx::{RuneContext, RInitialState};
use console::Console;
use rune::explorer::interactive::Command;

static USAGE: &'static str = "
runec. Interactive console for rune.

Usage:
  runec [-p PATH] FILE
  runec (-h | --help)

Options:
  -h --help                              Show this screen.
  -p                                     Load a previous configuration of state
";

#[derive(Debug, Clone, RustcDecodable)]
struct Args {
    flag_help: bool,
    flag_project: bool,
    arg_PATH: String,
    arg_FILE: String,
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

    if args.get_bool("-p") {
        let path = args.get_str("PATH");
        is = RInitialState::import_from_json(path);
    }

    loop {
        match c.read_command()[0] {
            Command::Run => {
                // Flexible to use any explorer here.
                let mut explorer = InteractiveExplorer::new();
                explorer.bp = is.get_breakpoints();

                let mut ctx = is.create_context();

                let mut rune = Rune::new(ctx, explorer, stream);
                rune.run().expect("Rune Error!");
                break;
            },
            Command::Help => {
                c.print_help();
                continue;
            },
            Command::Save => {
                is.write_to_json();
                continue;
            },
            Command::Debug => {
                // TODO: Pretty print this shit?
                c.print_info(&is.get_string());
                continue;
            },
            Command::SetContext(SAssignment { lvalue: Key::Mem(val),
                                              rvalue: ValType::Break }) => {
                is.add_breakpoint(val as u64);
            },
            Command::SetContext(SAssignment { lvalue: ref val, 
                                              rvalue: ValType::Symbolic }) => {
                is.add_sym(val.clone());
            },
            Command::SetContext(SAssignment { lvalue: ref key,
                                              rvalue: ValType::Concrete(val) }) => {
                if *key == Key::Reg("rip".to_owned()) {
                    is.set_start_addr(val as u64);
                } else {
                    is.add_const((key.clone(), val as u64));
                }
            },
            Command::Exit => {
                c.print_info("Thanks for using rune!");
                exit(0);
                break;
            },
            _ => break,
        }
    }
}
