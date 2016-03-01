//! Defines basic operation defined under Core theory in SMTLIB2.

use std::fmt;
use std::fmt::Debug;

use smt::smt::SMTNode;

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
    Const(u64, usize),
    FreeVar(String)
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
            OpCodes::FreeVar(ref s) => s.clone(),
            OpCodes::Const(_,_) => panic!(),
        };
        write!(f, "{}", s)
    }
}

impl_smt_node!(OpCodes);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sorts {
    Bool
}

impl fmt::Display for Sorts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", "bool")
    }
}
