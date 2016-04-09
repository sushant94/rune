//! Defines abstraction `Console`
//!
//! TODO:
//!  - Add colors to prompt
//!  - Use readline for command history and other goodies

use std::io::{self, Read, Write};
use std::iter;

use interact::Command;

// Defining default constants for the prompt.
static PROMPT: &'static str = ">>> ";
static OUTPUT: &'static str = "< ";

#[derive(Clone, Debug)]
pub struct Console {
    prompt: String,
    out_prompt: String,
}

impl Default for Console {
    fn default() -> Console {
        Console {
            prompt: PROMPT.to_owned(),
            out_prompt: OUTPUT.to_owned(),
        }
    }
}

impl Console {
    pub fn readline(&self) -> io::Result<String> {
        self.print_prompt();
        let mut buffer = String::new();
        let res = io::stdin().read_line(&mut buffer);
        Ok(buffer)
    }

    pub fn read_command(&self) -> Vec<Command> {
        let mut cmd;
        let mut repeat;

        loop {
            let buffer = self.readline().expect("Read failed!");
            cmd = if let Some(ref c) = buffer.chars().nth(0) {
                From::from(*c)
            } else {
                Command::Invalid
            };

            repeat = buffer.trim().chars().skip(1).fold(0, |acc, c: char| {
                if c == ' ' {
                    acc
                } else {
                    (acc * 10) + c.to_digit(10).unwrap()
                }
            });

            if cmd.is_valid() {
                break;
            }
        }
        iter::repeat(cmd).take(repeat as usize + 1).collect::<Vec<_>>()
    }

    pub fn print_prompt(&self) {
        print!("{}", self.prompt);
        io::stdout().flush().ok().expect("Could not flush stdout");
    }

    pub fn print(&self) {
        println!("{}", self.out_prompt);
    }

    pub fn print_success(&self, s: &str) {
        println!("[$] {}", s);
    }

    pub fn print_error(&self, s: &str) {
        println!("[!] {}", s);
    }

    pub fn print_info(&self, s: &str) {
        println!("[*] {}", s);
    }
}
