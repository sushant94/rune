//! Module that defines traits that need to be implemented, as a prerequisite to implement
//! `Context`, that provies it SMT solver capabilities.

use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt;

use smt::theories::core;

use smt::ssmt::SMTInit;

#[derive(Clone, Debug)]
pub enum SMTError {
    Undefined,
    Unsat,
    AssertionError(String),
}

//#[derive(Clone, Copy, Debug)]
//#[allow(non_camel_case_types)]
//pub enum Logic {
    //QF_BV,
    //QF_AX,
    //QF_ABV,
    //QF_AUFB,
//}

//impl fmt::Display for Logic {
    //fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //let s = match *self {
            //Logic::QF_BV => "QF_BV",
            //Logic::QF_AX => "QF_AX",
            //Logic::QF_ABV => "QF_ABV",
            //Logic::QF_AUFB => "QF_AUFB",
        //};
        //write!(f, "{}", s)
    //}
//}

#[derive(Clone, Debug)]
pub enum Type {
    Int,
    BitVector(usize),
    Array(Box<Type>, Box<Type>),
    Float,
    Bool,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Type::Int => "Int".to_owned(),
            Type::BitVector(n) => format!("(_ BitVec {})", n),
            Type::Array(ref idx, ref ty) => format!("(Array {} {})", idx, ty),
            Type::Float => unimplemented!(),
            Type::Bool => "Bool".to_owned(),
        };
        write!(f, "{}", s)
    }
}

pub type SMTResult<T> = Result<T, SMTError>;

/// Trait to be implemented by `Context` to support SMT Solving.
pub trait SMT {
    /// Indexing mechanism that allows the outside world to refer to the variables inside the
    /// context.
    type Idx: Clone + Debug;

    /// Return one solution
    fn solve_for<B: SMTInit>(&Self::Idx, &mut B) -> SMTResult<u64>;
    /// Repeatedly query the SMT solver to obtain all possible solutions for a set of constraints.
    fn solve_all_for<B: SMTInit>(&Self::Idx, &mut B) -> SMTResult<Vec<u64>>;
    /// Check if the constraints are satisfiable.
    fn check_sat<B: SMTInit>(&mut self, &mut B) -> SMTResult<bool>;
}

/// Trait a backend should implement to support SMT solving.
///
/// This is a minimalistic API and has to be expanded in the future to support more SMT operations
/// and to grow this into a full SMTLib Crate.
///
/// All functions names are analogous in meaning to their usage in the original SMT-LIB2 sense.
/// TODO:
///  - define_fun
///  - declare_sort
///  - define_sort
///  - get_proof
///  - get_unsat_core
///  - get_value
///  - get_assignment
///  - push
///  - pop
///  - get_option
///  - set_option
///  - get_info
///  - set_info
///  - exit
pub trait SMTBackend {
    type Ident: Debug + Clone;
    type Assertion: Debug + Clone;

    //fn set_logic<T: Logic>(&mut self, T);
    fn declare_fun<T: AsRef<str>>(&mut self, Option<T>, Option<Vec<Type>>, Type) -> Self::Ident;

    fn new_var<T: AsRef<str>>(&mut self, Option<T>, Type) -> Self::Ident;
    fn assert(&mut self, Self::Assertion, &[Self::Ident]) -> Self::Ident;
    fn check_sat(&mut self) -> bool;
    fn solve(&mut self) -> SMTResult<HashMap<Self::Ident, u64>>;

    fn raw_write<T: AsRef<str>>(&mut self, T);
    fn raw_read(&mut self) -> String;
}

pub trait Logic: fmt::Display {
    type Fns;
    type Sorts;
    
    fn free_var<T: AsRef<str>>(name: T) -> Self::Fns;
}

