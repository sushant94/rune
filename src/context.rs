//! Traits and implementation to provide context for symbolic emulation
//!
//! This module defines the `Context` trait which maintains state.
//! Context broadly refers to the register and memory state that the symbolic
//! emulator is currently
//! operating on/under. In short, the methods and structs defined here are used
//! to keep track of
//! registers and memory in the symbolic emulator.
//!
//! Custom memory and register profile implementations for the symbolic
//! emulator must implement the
//! `Context` trait.
//!
//! TODO: More details about this module, including its working, philosophy,
//! use etc.
//! TODO: Add an example usage.

use std::io::Write;
use std::hash::Hash;
use std::fmt::Debug;

use bv::BitVector;

/// Ri - Register Index. Used if context implementation uses Indexes to reference registers.
/// Mi - Memory Index. Used if the underlying context implementation uses indexes for reference.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RefType<Ri, Mi>
    where Ri: Clone + Debug,
          Mi: Clone + Debug
{
    // Name of the register in human readable format.
    RegisterIdent(String),
    RegisterIndex(Ri),
    Mem(Mi),
    // Reference memory by address.
    MemAddr(u64),
}

#[derive(Clone, Copy, Debug)]
pub enum ContextError {
}

pub type ContextResult<T> = Result<T, ContextError>;

pub trait Context: Clone {
    type BV: BitVector;
    type Idx: Clone + Hash + Eq;

    fn new() -> Self;

    fn write(&mut self, &Self::Idx, Self::BV);
    fn read(&mut self, Self::Idx) -> ContextResult<Self::BV>;
    fn read_mem(&mut self, Self::Idx) -> ContextResult<Self::BV>;
    fn write_mem(&mut self, Self::Idx, &Self::BV) -> ContextResult<()>;
    /// Equivalent to write but intended to be used when BV is available rather than
    /// a reference to it.
    fn update_bv(&mut self, &Self::BV, Self::BV) -> ContextResult<()>;

    fn load_file<S: AsRef<str>>(S) -> ContextResult<Self>;
    fn load(&[u8]) -> ContextResult<Self>;
    fn dump<W: Write>(&self, &mut W);

    fn mark_symbolic(&mut self, &Self::Idx);

    fn solve(&self, &Self::Idx) -> ContextResult<Self::BV>;
    fn solve_all(&self, &Self::Idx) -> ContextResult<Vec<Self::BV>>;

    fn new_value(&mut self, u64) -> Self::BV;
    fn new_symbol(&mut self) -> Self::BV;

    fn ip(&self) -> u64;
    fn increment_ip(&mut self, &u64);
}

#[cfg(test)]
mod test {

}
