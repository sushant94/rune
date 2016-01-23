//! Traits and implementation to provide context for symbolic emulation
//!
//! This module defines the `Context` trait which is needed to maintain and operate on state.
//! Context broadly refers to the register and memory state that the symbolic emulator is currently
//! operating on/under. In short, the methods and structs defined here are used to keep track of
//! registers and memory in the symbolic emulator.
//!
//! Custom memory and register profile implementations for the symbolic emulator must implement the
//! `Context` trait.
//!
//! TODO: More details about this module, including its working, philosophy, use etc.
//! TODO: Add an example usage.

use std::io::Write;
use bv::BitVector;

/// Ri - Register Index. Used if context implementation uses Indexes to reference registers.
/// Mi - Memory Index. Used if the underlying context implementation uses indexes for reference.
pub enum RefType<Ri, Mi> 
where Ri: Clone,
      Mi: Clone {
    // Name of the register in human readable format.
    RegisterIdent(String),
    RegisterIndex(Ri),
    Mem(Mi),
    // Reference memory by address.
    MemAddr(u64),
}

pub enum ContextError {
}

pub type ContextResult<T> = Result<T, ContextError>;

pub trait Context: Clone {
    type T: BitVector;
    type I: Clone;

    fn new() -> Self;

    fn write(&mut self, &Self::I, Self::T);
    fn read(&mut self, &Self::I) -> ContextResult<Self::T>;

    fn load_file<S: AsRef<str>>(S) -> ContextResult<Self>;
    fn load(&[u8]) -> ContextResult<Self>;
    fn dump<W: Write>(&self, &mut W);

    fn mark_symbolic(&mut self, &Self::I);

    fn solve(&self, &Self::I) -> ContextResult<Self::T>;
    fn solve_all(&self, &Self::I) -> ContextResult<Vec<Self::T>>;
}

#[cfg(test)]
mod test {

}
