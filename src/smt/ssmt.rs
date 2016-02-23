//! Module that comtains SMTLib Backend Implementation.
//!
//! This backend outputs the constraints in standard smt-lib2 format. Hence,
//! any solver that supports this format maybe used to solve for constraints.

use std::process::{Child, Command, Stdio};
use std::collections::HashMap;
use std::io::{Read, Write};
use regex::Regex;

use smt::smt::{Logic, SMTBackend, SMTError, SMTResult, Type};

/// Enum that contains the solvers that support SMTLib2 format.
#[derive(Debug, Clone, Copy)]
pub enum Solver {
    Z3,
}

/// Trait an actual backend solver must implement in order to be compatible with SMTLib2
pub trait SMTSolver {
    /// Return the string representation of the name of the solver.
    /// This is used to distinguish between the solver and make decisions based on their varied
    /// capabilities.
    fn name(&self) -> String;
    /// Shell command to be executed in order to invoke the solver.
    /// Note that the solver must support smtlib2 format in order to be compatible.
    /// This function should return a tuple of shell command and the arguments to be passed to it.
    fn exec_str(&self) -> (String, Vec<String>);
    /// Run the solver and keep it open for further IO.
    fn exec(&self) -> Child {
        let (cmd, args) = self.exec_str();
        Command::new(cmd)
            .args(&args)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn process")
    }
}

impl SMTSolver for Solver {
    fn exec_str(&self) -> (String, Vec<String>) {
        match *self {
            Solver::Z3 => ("z3".to_owned(), vec!["-in".to_owned(), "-smt2".to_owned()]),
        }
    }

    fn name(&self) -> String {
        match *self {
            Solver::Z3 => "Z3",
        }
        .to_owned()
    }
}

/// Solver struct that wraps the spawned sub-process.
pub struct SMTLib2 {
    solver: Option<Child>,
    logic: Option<Logic>,

}

impl SMTLib2 {
    pub fn new<T: SMTSolver>(s_type: T) -> SMTLib2 {
        SMTLib2 {
            solver: Some(s_type.exec()),
            logic: None,
        }
    }

    pub fn write<T: AsRef<str>>(&mut self, s: T) {
        // TODO: Check for errors.
        // println!("Writing: {}", s);
        if let Some(ref mut stdin) = self.solver.as_mut().unwrap().stdin.as_mut() {
            stdin.write(s.as_ref().as_bytes()).expect("Write to stdin failed");
            stdin.flush().expect("Failed to flush stdin");
        }
    }

    pub fn read(&mut self) -> String {
        // XXX: This read may block indefinitely if there is nothing on the pipe to be
        // read. To prevent this we need a timeout mechanism after which we should return with
        // an error, such as: ErrTimeout.
        // Another important point to note here is that, if the data available to read is exactly
        // 2048 bytes, then this reading mechanism fails and will end up waiting to read more data
        // (when none is available) indefinitely.
        let mut bytes_read = [0; 2048];
        let mut s = String::new();
        if let Some(ref mut solver) = self.solver.as_mut() {
            if let Some(ref mut stdout) = solver.stdout.as_mut() {
                loop {
                    let n = stdout.read(&mut bytes_read).unwrap();
                    s = format!("{}{}",
                                s,
                                String::from_utf8(bytes_read[0..n].to_vec()).unwrap());
                    if n < 2048 {
                        break;
                    }
                }
            }
        }
        s
    }
}

impl SMTBackend for SMTLib2 {
    type Ident = String;
    type Assertion = String;

    fn new_var(&mut self, ident: String, typ: Type) {
        self.write(format!("(declare-fun {} () {})", ident, typ.to_string()));
    }

    fn set_logic(&mut self, logic: Logic) {
        // Set logic can only be set once in the solver and before  any declaration,
        // definitions, assert or check-sat commands. Only exit, option and info commands may
        // precede a set-logic command.
        if self.logic.is_some() { panic!() }
        self.logic = Some(logic);
        //self.write(format!("(set-logic {})\n", logic.to_string()));
    }

    fn assert(&mut self, _: Self::Ident, assert: Self::Assertion) {
        // TODO: In the future we may need to perform simplifications and optimizations
        // on the queries before sending them to the solver. But currently, in this simple
        // implementation, we will just write out the assertions to the solver and let it take care of
        // the correctness.
        // TODO 2: If the assertions result in an error, this must be parsed to a human-readable
        // form and returned from this function so that the caller may handle it.
        self.write(assert);
    }

    fn check_sat(&mut self) -> bool {
        self.write("(check-sat)\n".to_owned());
        if &self.read() == "sat\n" {
            true
        } else {
            false
        }
    }

    // TODO: Return type information along with the value.
    fn solve(&mut self) -> SMTResult<HashMap<Self::Ident, u64>> {
        let mut result = HashMap::new();
        if !self.check_sat() {
            return Err(SMTError::Unsat);
        }

        self.write("(get-model)\n".to_owned());
        // XXX: For some reason we need two reads here in order to get the result from
        // the SMT solver. Need to look into the reason for this. This might stop working in the
        // future.
        let _ = self.read();
        let read_result = self.read();

        // Example of result from the solver:
        // (model
        //  (define-fun y () Int
        //    9)
        //  (define-fun x () Int
        //    10)
        // )
        let re = Regex::new(r"\s+\(define-fun (?P<var>[0-9a-zA-Z_]+) \(\) [(]?[ _a-zA-Z0-9]+[)]?\n\s+(?P<val>([0-9]+|#x[0-9a-f]+|#b[01]+))")
                     .unwrap();
        for caps in re.captures_iter(&read_result) {
            // Here the caps.name("val") can be a hex value, or a binary value or a decimal
            // value. We need to parse the output to a u64 accordingly.
            let val_str = caps.name("val").unwrap();
            let val = if val_str.len() > 2 && &val_str[0..2] == "#x" {
                u64::from_str_radix(&val_str[2..], 16)
            } else if val_str.len() > 2 && &val_str[0..2] == "#b" {
                u64::from_str_radix(&val_str[2..], 2)
            } else {
                val_str.parse::<u64>()
            }.unwrap();
            result.insert(caps.name("var").unwrap().to_owned(), val);
        }

        Ok(result)
    }

    fn raw_write<T: AsRef<str>>(&mut self, w: T) {
        self.write(w);
    }

    fn raw_read(&mut self) -> String {
        self.read()
    }
}

/// A trait that is to be implemented on a struct that configures and spawns an SMTBackend.
pub trait SMTInit {
    type For: SMTBackend;
    fn spawn(&self) -> Option<Self::For>;
}

/// Wrapper struct that is used to spawn an instance of Z3 and wrap it into a `SMTLib2`.
///
/// This provides a nice way to configure solvers before spawning an instance of it and a chance to
/// run commands in the solver before they are used elsewhere.
///
/// __TODO__: This has to be expanded to other solvers.
pub struct SpawnZ3;
impl SpawnZ3 {
    pub fn new() -> SpawnZ3 {
        SpawnZ3
    }
}

impl SMTInit<For = SMTLib2> {
    fn spawn(&self) -> Option<SMTLib2> {
        Some(SMTLib2::new(Solver::Z3))
    }
}

#[cfg(test)]
mod test {
    use smt::smt::*;
    use super::*;

    #[test]
    fn test_z3_int() {
        let mut solver = SMTLib2::new(Solver::Z3);
        solver.new_var("x".to_owned(), Type::Int);
        solver.new_var("y".to_owned(), Type::Int);
        solver.assert("".to_owned(), "(assert (= x 10))".to_owned());
        solver.assert("".to_owned(), "(assert (> x y))".to_owned());
        let result = solver.solve().unwrap();
        assert_eq!(result["x"], 10);
        assert_eq!(result["y"], 9);
    }

    #[test]
    fn test_z3_bitvec() {
        let mut solver = SMTLib2::new(Solver::Z3);
        solver.set_logic(Logic::QF_BV);
        solver.new_var("x".to_owned(), Type::BitVector(32));
        solver.assert("".to_owned(), "(assert (= x (_ bv10 32)))".to_owned());
        let result = solver.solve().unwrap();
        assert_eq!(result["x"], 10);
    }
}
