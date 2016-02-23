use std::fmt::Debug;
use smt::fmt

#[derive(Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
pub enum OpCodes {
    concat,
    extract(u64, u64),
    bvnot,
    bvand,
    bvor,
    bvneg,
    bvadd,
    bvmul,
    bvudiv,
    bvurem,
    bvshl,
    bvlshr,
    bvult,
    bvnand,
    bvnor,
    bvxor,
    bvxnor,
    bvcomp,
    bvsub,
    bvsdiv,
    bvsrem,
    bvsmod,
    bvashr,
    // parameterized functions
    repeat(u64),
    zero_extend(u64),
    sign_extend(u64),
    rotate_left(u64),
    rotate_right(u64),
    // logical functions
    bvule,
    bvugt,
    bvuge,
    bvslt,
    bvsle,
    bvsgt,
    bvsge,
}

impl fmt::Display for OpCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            OpCodes::concat => "concat",
            OpCodes::extract(i, j) => format!("_ extract {} {}", i, j),
            OpCodes::bvnot => "bvnot",
            OpCodes::bvand => "bvand",
            OpCodes::bvor => "bvor",
            OpCodes::bvneg => "bvneg",
            OpCodes::bvadd => "bvadd",
            OpCodes::bvmul => "bvmul",
            OpCodes::bvudiv => "bvudiv",
            OpCodes::bvurem => "bvurem",
            OpCodes::bvshl => "bvshl",
            OpCodes::bvlshr => "bvlshr",
            OpCodes::bvult => "bvult",
            OpCodes::bvnand => "bvnand",
            OpCodes::bvnor => "bvnor",
            OpCodes::bvxor => "bvxor",
            OpCodes::bvxnor => "bvxnor",
            OpCodes::bvcomp => "bvcomp",
            OpCodes::bvsub => "bvsub",
            OpCodes::bvsdiv => "bvsdiv",
            OpCodes::bvsrem => "bvsrem",
            OpCodes::bvsmod => "bvsmod",
            OpCodes::bvashr => "bvashr",
            OpCodes::repeat(i) => format!("_ repeat {}", i),
            OpCodes::zero_extend(i) => format!("_ zero_extend {}", i),
            OpCodes::sign_extend(i) => format!("_ sign_extend {}", i),
            OpCodes::rotate_left(i) => format!("_ rotate_left {}", i),
            OpCodes::rotate_right(i) => format!("_ rotate_right {}", i),
            OpCodes::bvule => "bvule",
            OpCodes::bvugt => "bvugt",
            OpCodes::bvuge => "bvuge",
            OpCodes::bvslt => "bvslt",
            OpCodes::bvsle => "bvsle",
            OpCodes::bvsgt => "bvsgt",
            OpCodes::bvsge => "bvsge",
        };
        write!(f, "{}", s)        
    }
}
