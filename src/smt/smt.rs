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
    type Idx: Debug + Clone;
    type Logic: Logic;

    fn set_logic(&mut self);
    //fn declare_fun<T: AsRef<str>>(&mut self, Option<T>, Option<Vec<Type>>, Type) -> Self::Idx;

    fn new_var<T, P>(&mut self, Option<T>, P) -> Self::Idx
        where T: AsRef<str>,
              P: Into<<<Self as SMTBackend>::Logic as Logic>::Sorts>;

    fn assert<T: Into<<<Self as SMTBackend>::Logic as Logic>::Fns>>(&mut self, T, &[Self::Idx]) -> Self::Idx;
    fn check_sat(&mut self) -> bool;
    fn solve(&mut self) -> SMTResult<HashMap<Self::Idx, u64>>;

    fn raw_write<T: AsRef<str>>(&mut self, T);
    fn raw_read(&mut self) -> String;
}

pub trait Logic: fmt::Display + Clone + Copy {
    type Fns: SMTNode + fmt::Display + Debug + Clone;
    type Sorts: fmt::Display + Debug + Clone;
    
    fn free_var<T: AsRef<str>>(T, Self::Sorts) -> Self::Fns;
}

pub trait SMTNode: fmt::Display {
    /// Returns true if the node is a symbolic variable
    fn is_var(&self) -> bool;
    /// Returns true if the node is a constant value
    fn is_const(&self) -> bool;
    /// Returns true if the node is a function
    fn is_fn(&self) -> bool {
        !self.is_var() && !self.is_const()
    }
}
