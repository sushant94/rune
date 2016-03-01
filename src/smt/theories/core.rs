//! Defines basic operations defined under Core theory in SMTLIB2.

use std::fmt;
use std::fmt::Debug;
use smt::smt::SMTNode;

#[macro_use]
use smt::theories::utils;

#[derive(Clone, Debug)]
pub enum OpCodes {
    True,
    False,
    Not,
    Imply,
    And,
    Or,
    Xor,
    Cmp,
    Distinct,
    ITE,
    FreeVar(String),
}

impl fmt::Display for OpCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            OpCodes::Not => "not".to_owned(),
            OpCodes::Imply => "=>".to_owned(),
            OpCodes::And => "and".to_owned(),
            OpCodes::Or => "or".to_owned(),
            OpCodes::Xor => "xor".to_owned(),
            OpCodes::Cmp => "=".to_owned(),
            OpCodes::Distinct => "distinct".to_owned(),
            OpCodes::ITE => "ite".to_owned(),
            OpCodes::True => "true".to_owned(),
            OpCodes::False => "false".to_owned(),
            OpCodes::FreeVar(ref name) => format!("{}", name),
        };
        write!(f, "{}", s)
    }
}

impl_smt_node!(OpCodes);

#[derive(Clone, Debug)]
pub enum Sorts {
    Bool
}

impl fmt::Display for Sorts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", "Bool")
    }
}
