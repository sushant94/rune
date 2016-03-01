//! Traits, method and structs that represent theories and logics for SMTLIB2
//!
//! All notations are directly taken from the SMTLIB2 Standard and mean the same in this context.

pub mod smt;
pub mod ssmt;

pub mod theories {
    #[macro_use]
    pub mod utils;
    pub mod bitvec;
    pub mod integer;
    pub mod core;
    pub mod array_ex;
    pub mod real;
    pub mod real_ints;
}

pub mod logics {
    #[macro_use]
    pub mod utils;
    pub mod qf_bv;
    pub mod qf_aufbv;
    pub mod qf_abv;
}
