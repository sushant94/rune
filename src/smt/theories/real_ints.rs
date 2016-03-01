//! Defines basic operations defined under Real_Ints theory in SMTLIB2.

use std::fmt;
use smt::smt::SMTNode;

#[macro_use]
use smt::theories::utils;

#[derive(Clone, Debug)]
pub enum OpCodes {
    Neg,
    Sub,
    Add,
    Mul,
    Div,
    Lte,
    Lt,
    Gte,
    Gt,
    To_real,
    To_int,
    Is_int,
    ConstInt(u64),
    ConstReal(f64),
    FreeVar(String),
}

impl fmt::Display for OpCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            OpCodes::Neg => "-".to_owned(),
            OpCodes::Sub => "-".to_owned(),
            OpCodes::Add => "+".to_owned(),
            OpCodes::Mul => "*".to_owned(),
            OpCodes::Div => "/".to_owned(),
            OpCodes::Lte => "<=".to_owned(),
            OpCodes::Lt => "<".to_owned(),
            OpCodes::Gte => ">=".to_owned(),
            OpCodes::Gt => ">".to_owned(),
            OpCodes::To_real => "to_real".to_owned(),
            OpCodes::To_int => "to_int".to_owned(),
            OpCodes::Is_int => "is_int".to_owned(),
            OpCodes::ConstInt(ref val) => format!("{}", val),
            OpCodes::ConstReal(ref val) => format!("{}", val),
            OpCodes::FreeVar(ref name) => format!("{}", name),
        };
        write!(f, "{}", s)
    }
}

impl_smt_node!(OpCodes)

#[derive(Clone,Debug)]
pub enum Sorts {
    Real,
    Int
}

impl fmt::Display for Sorts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Sorts::Real => "Real",
            Sorts::Int => "Int"
        };
        write!(f, "{}", "s")
    }
}
