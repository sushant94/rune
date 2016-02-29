//! Module that describes the ArrayEx Theory

use std::fmt;

pub enum OpCodes {
    Select,
    Store,
}

impl fmt::Display for OpCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            OpCodes::Select => "select",
            OpCodes::Store => "store",
        };
        write!(f, "{}", s)
    }
}
