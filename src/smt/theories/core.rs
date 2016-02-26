//! Defines basic operation defined under Core theory in SMTLIB2.

use std::fmt;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug)]
pub enum OpCodes {
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
        };
        write!(f, "{}", s)
    }
}
