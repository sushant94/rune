//! Module that describes the ArrayEx Theory

use std::fmt::Debug;
use std::fmt;
use smt::smt::SMTNode;

#[macro_use]
use smt::theories::utils;

#[derive(Clone, Debug)]
pub enum OpCodes {
    Select,
    Store,
    FreeVar(String),
}

impl fmt::Display for OpCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            OpCodes::Select => "select".to_owned(),
            OpCodes::Store => "store".to_owned(),
            OpCodes::FreeVar(ref name) => format!("{}", name),
        };
        write!(f, "{}", s)
    }
}

impl_smt_node!(OpCodes);

#[derive(Clone, Debug)]
pub enum Sorts {
    Array(String, String),
}

impl fmt::Display for Sorts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Sorts::Array(ref x, ref y) => format!("(Array {} {})", x, y),
        };
        write!(f, "{}", s)
    }
}
