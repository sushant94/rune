//! Module that describes the ArrayEx Theory

use std::fmt;

use smt::smt::SMTNode;

#[derive(Clone, Debug)]
pub enum OpCodes {
    Select,
    Store,
    FreeVar(String),
    Const(u64),
}

impl fmt::Display for OpCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            OpCodes::Select => "select".to_owned(),
            OpCodes::Store => "store".to_owned(),
            OpCodes::FreeVar(ref s) => s.clone(),
            OpCodes::Const(_) => unreachable!(),
        };
        write!(f, "{}", s)
    }
}

impl_smt_node!(OpCodes, define consts [OpCodes::Const(_)]);

#[derive(Clone, Debug)]
pub enum Sorts<X, Y>
    where X: fmt::Display + fmt::Debug + Clone,
          Y: fmt::Display + fmt::Debug + Clone
{
    Array(Box<X>, Box<Y>),
}

impl<X, Y> fmt::Display for Sorts<X, Y>
    where X: fmt::Display + fmt::Debug + Clone,
          Y: fmt::Display + fmt::Debug + Clone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Sorts::Array(ref x, ref y) => format!("(Array {} {})", x, y),
        };
        write!(f, "{}", s)
    }
}
