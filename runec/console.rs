//! Defines abstraction `Console`

extern crate rustyline;
use self::rustyline::error::ReadlineError;
use self::rustyline::Editor;

use std::io::{self, Read, Write};
use std::iter;

use interact::Command;

// Defining default constants for the prompt.
static PROMPT: &'static str = "\x1b[1;32m>>>\x1b[0m ";
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
    pub fn read_command(&self) -> Vec<Command> {
        let mut cmd;
        let mut repeat;

        let mut r = Editor::<()>::new();

        if let Err(_) = r.load_history("history.txt") {
            self.print_info("No history found.");
        }
        loop {
            // Add command completion
            let readline = r.readline(PROMPT);

            match readline {
                Ok(buffer) => {
                    r.add_history_entry(&buffer);
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
                },
                Err(ReadlineError::Interrupted) => {
                    cmd = Command::Invalid;
                    repeat = 1;
                },
                Err(ReadlineError::Eof) => {
                    println!("[!] CTRL-D");
                    cmd = Command::Exit;
                    repeat = 1;
                    break;
                }, 
                Err(err) => {
                    println!("[!] Error: {:?}", err);
                    cmd = Command::Invalid;
                    repeat = 1;
                    break;
                }
            }
        }
        r.save_history("history.txt").unwrap();
        iter::repeat(cmd).take(repeat as usize + 1).collect::<Vec<_>>()
    }

    pub fn readline(&self) -> io::Result<String> {
        self.print_prompt();
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer);
        Ok(buffer)
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
