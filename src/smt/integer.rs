//! Defines basic operation defined under QF_UF in SMTLIB2.

use std::fmt;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug)]
pub enum OpCodes {
    Cmp,
    Lt,
    Gt,
    Lte,
    Gte,
    Mod,
    Div,
    Abs,
    Mul,
    Add,
    Sub,
    Neg,
}


impl fmt::Display for OpCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            OpCodes::Cmp => "=",
            OpCodes::Lt => "<",
            OpCodes::Gt => ">",
            OpCodes::Lte => "<=",
            OpCodes::Gte => ">=",
            OpCodes::Mod => "mod",
            OpCodes::Div => "div",
            OpCodes::Abs => "abs",
            OpCodes::Mul => "*",
            OpCodes::Add => "+",
            OpCodes::Sub => "-",
            OpCodes::Neg => "-",
        };
        write!(f, "{}", s)
    }
}
