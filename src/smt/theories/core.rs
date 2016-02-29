//! Defines basic operation defined under Core theory in SMTLIB2.

use std::fmt;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug)]
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
}

impl fmt::Display for OpCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            OpCodes::Not => "not",
            OpCodes::Imply => "=>",
            OpCodes::And => "and",
            OpCodes::Or => "or",
            OpCodes::Xor => "xor",
            OpCodes::Cmp => "=",
            OpCodes::Distinct => "distinct",
            OpCodes::ITE => "ite",
            OpCodes::True => "true",
            OpCodes::False => "false",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sorts {
    Bool
}

impl fmt::Display for Sorts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", "bool")
    }
}
