use std::fmt::Debug;
use std::fmt;

#[derive(Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
pub enum OpCodes {
    Concat,
    Extract(u64, u64),
    Bvnot,
    Bvand,
    Bvor,
    Bvneg,
    Bvadd,
    Bvmul,
    Bvudiv,
    Bvurem,
    Bvshl,
    Bvlshr,
    Bvult,
    Bvnand,
    Bvnor,
    Bvxor,
    Bvxnor,
    Bvcomp,
    Bvsub,
    Bvsdiv,
    Bvsrem,
    Bvsmod,
    Bvashr,
    // parameterized functions
    Repeat(u64),
    Zero_extend(u64),
    Sign_extend(u64),
    Rotate_left(u64),
    Rotate_right(u64),
    // logical functions
    Bvule,
    Bvugt,
    Bvuge,
    Bvslt,
    Bvsle,
    Bvsgt,
    Bvsge,
    Const(u64, usize),
}

impl fmt::Display for OpCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            OpCodes::Concat => "concat".to_owned(),
            OpCodes::Extract(i, j) => format!("(_ extract {} {})", i, j),
            OpCodes::Bvnot => "bvnot".to_owned(),
            OpCodes::Bvand => "bvand".to_owned(),
            OpCodes::Bvor => "bvor".to_owned(),
            OpCodes::Bvneg => "bvneg".to_owned(),
            OpCodes::Bvadd => "bvadd".to_owned(),
            OpCodes::Bvmul => "bvmul".to_owned(),
            OpCodes::Bvudiv => "bvudiv".to_owned(),
            OpCodes::Bvurem => "bvurem".to_owned(),
            OpCodes::Bvshl => "bvshl".to_owned(),
            OpCodes::Bvlshr => "bvlshr".to_owned(),
            OpCodes::Bvult => "bvult".to_owned(),
            OpCodes::Bvnand => "bvnand".to_owned(),
            OpCodes::Bvnor => "bvnor".to_owned(),
            OpCodes::Bvxor => "bvxor".to_owned(),
            OpCodes::Bvxnor => "bvxnor".to_owned(),
            OpCodes::Bvcomp => "bvcomp".to_owned(),
            OpCodes::Bvsub => "bvsub".to_owned(),
            OpCodes::Bvsdiv => "bvsdiv".to_owned(),
            OpCodes::Bvsrem => "bvsrem".to_owned(),
            OpCodes::Bvsmod => "bvsmod".to_owned(),
            OpCodes::Bvashr => "bvashr".to_owned(),
            OpCodes::Repeat(i) => format!("(_ repeat {})", i),
            OpCodes::Zero_extend(i) => format!("(_ zero_extend {})", i),
            OpCodes::Sign_extend(i) => format!("(_ sign_extend {})", i),
            OpCodes::Rotate_left(i) => format!("(_ rotate_left {})", i),
            OpCodes::Rotate_right(i) => format!("(_ rotate_right {})", i),
            OpCodes::Bvule => "bvule".to_owned(),
            OpCodes::Bvugt => "bvugt".to_owned(),
            OpCodes::Bvuge => "bvuge".to_owned(),
            OpCodes::Bvslt => "bvslt".to_owned(),
            OpCodes::Bvsle => "bvsle".to_owned(),
            OpCodes::Bvsgt => "bvsgt".to_owned(),
            OpCodes::Bvsge => "bvsge".to_owned(),
            OpCodes::Const(val, n) => format!("(_ bv{} {})", val, n),
        };
        write!(f, "{}", s)
    }
}

pub enum Sorts {
    BitVector(usize, usize),
}
