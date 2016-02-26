use std::fmt::Debug;
use std::fmt;

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
            OpCodes::concat => "concat".to_owned(),
            OpCodes::extract(i, j) => format!("(_ extract {} {})", i, j),
            OpCodes::bvnot => "bvnot".to_owned(),
            OpCodes::bvand => "bvand".to_owned(),
            OpCodes::bvor => "bvor".to_owned(),
            OpCodes::bvneg => "bvneg".to_owned(),
            OpCodes::bvadd => "bvadd".to_owned(),
            OpCodes::bvmul => "bvmul".to_owned(),
            OpCodes::bvudiv => "bvudiv".to_owned(),
            OpCodes::bvurem => "bvurem".to_owned(),
            OpCodes::bvshl => "bvshl".to_owned(),
            OpCodes::bvlshr => "bvlshr".to_owned(),
            OpCodes::bvult => "bvult".to_owned(),
            OpCodes::bvnand => "bvnand".to_owned(),
            OpCodes::bvnor => "bvnor".to_owned(),
            OpCodes::bvxor => "bvxor".to_owned(),
            OpCodes::bvxnor => "bvxnor".to_owned(),
            OpCodes::bvcomp => "bvcomp".to_owned(),
            OpCodes::bvsub => "bvsub".to_owned(),
            OpCodes::bvsdiv => "bvsdiv".to_owned(),
            OpCodes::bvsrem => "bvsrem".to_owned(),
            OpCodes::bvsmod => "bvsmod".to_owned(),
            OpCodes::bvashr => "bvashr".to_owned(),
            OpCodes::repeat(i) => format!("(_ repeat {})", i),
            OpCodes::zero_extend(i) => format!("(_ zero_extend {})", i),
            OpCodes::sign_extend(i) => format!("(_ sign_extend {})", i),
            OpCodes::rotate_left(i) => format!("(_ rotate_left {})", i),
            OpCodes::rotate_right(i) => format!("(_ rotate_right {})", i),
            OpCodes::bvule => "bvule".to_owned(),
            OpCodes::bvugt => "bvugt".to_owned(),
            OpCodes::bvuge => "bvuge".to_owned(),
            OpCodes::bvslt => "bvslt".to_owned(),
            OpCodes::bvsle => "bvsle".to_owned(),
            OpCodes::bvsgt => "bvsgt".to_owned(),
            OpCodes::bvsge => "bvsge".to_owned(),
        };
        write!(f, "{}", s)
    }
}
