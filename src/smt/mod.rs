//! Traits, method and structs that represent theories and logics for SMTLIB2
//!
//! All notations are directly taken from the SMTLIB2 Standard and mean the same in this context.

pub mod smt;
pub mod ssmt;

pub mod theories {
    pub mod bitvec;
    pub mod integer;
    pub mod core;
}
