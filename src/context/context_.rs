//! Traits that define a `Context`.

use std::fmt::Debug;
use std::hash::Hash;
use libsmt::smt::{SMT};
use libsmt::ssmt::{SMTInit};

pub trait Context: Sized {
    fn load_ctx(&[u8]) -> Self;
    fn dump_ctx(&self) -> &[u8];
}

pub trait RegisterRead: Sized {
    type VarRef: Clone + Debug + Hash + Eq;
    fn read_register<T: AsRef<str>>(&mut self, T) -> Self::VarRef;
}

pub trait RegisterWrite: Sized {
    type VarRef: Clone + Debug + Hash + Eq;
    fn write_register(&mut self, Self::VarRef, Self::VarRef);
}

pub trait MemoryRead: Sized + SMT {
    type VarRef: Clone + Debug + Hash + Eq + ToConcrete<Self>;
    fn read_mem<T: SMTInit>(&mut self, Self::VarRef) -> Self::VarRef;
}

pub trait MemoryWrite: Sized + SMT {
    type VarRef: Clone + Debug + Hash + Eq + ToConcrete<Self>;
    fn write_mem<T: SMTInit>(&mut self, Self::VarRef, Self::VarRef);
}

pub trait ToConcrete<U: SMT> {
    fn concrete<B: SMTInit>(&self, &mut U) -> u64;
}
