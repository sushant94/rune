//! Defines abstraction `Console`
//!
//! TODO: 
//!  - Add colors to prompt
//!  - Use readline for command history and other goodies

use std::io::{self, Read};
use std::collections::VecDeque;
use std::iter;

use interact::Command;

// Defining default constants for the prompt.
static PROMPT: &'static str = ">>> ";
static OUTPUT: &'static str = "< ";

#[derive(Clone, Debug)]
pub struct Console {
    cmd_q: VecDeque<Command>,
    prompt: String,
    out_prompt: String,
}

impl Default for Console {
    fn default() -> Console {
        Console {
            cmd_q: VecDeque::new(),
            prompt: PROMPT.to_owned(),
            out_prompt: OUTPUT.to_owned(),
        }
    }
}

impl Console {
    pub fn readline(&self) -> io::Result<String> {
        self.print_prompt();
        let mut buffer = String::new();
        let res = io::stdin().read_to_string(&mut buffer);
        res.map(|_| buffer)
    }

    pub fn read_command(&mut self) -> Command {
        let mut cmd = self.cmd_q.pop_front().unwrap_or(Command::Invalid);
        let mut repeat;

        if cmd.is_valid() {
            return cmd;
        }

        loop {
            let buffer = self.readline().expect("Read failed!");
            cmd = if let Some(ref c) = buffer.chars().nth(0) {
                From::from(*c)
            } else {
                Command::Invalid
            };

            repeat = buffer.chars().skip(1).fold(0, |acc, c: char| {
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

        if repeat > 0 {
            self.cmd_q = iter::repeat(cmd).take(repeat as usize).collect();
        }
        cmd
    }

    pub fn print_prompt(&self) {
        println!("{}", self.prompt);
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
