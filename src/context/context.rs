//!  Defines `Context` trait to be used by symbolic emulator

use std::fmt::Debug;
use std::hash::Hash;

use libsmt::backends::smtlib2::SMTProc;

pub trait Context: Clone + Debug
                   + RegisterRead
                   + RegisterWrite<VarRef=<Self as RegisterRead>::VarRef>
                   + MemoryRead<VarRef=<Self as RegisterRead>::VarRef>
                   + MemoryWrite<VarRef=<Self as RegisterRead>::VarRef>
                   + Evaluate<VarRef=<Self as RegisterRead>::VarRef>
{
    fn ip(&self) -> u64;
    fn is_symbolic(&self) -> bool {
        true
    }
    fn is_concrete(&self) -> bool {
        !self.is_symbolic()
    }
    fn increment_ip(&mut self, u64);
    fn set_ip(&mut self, u64);
    fn define_const(&mut self, u64, usize) -> <Self as RegisterRead>::VarRef;
    fn alias_of(&self, String) -> Option<String>;
}

pub trait RegisterRead: Sized {
    type VarRef: Clone + Debug + Hash + Eq;
    fn reg_read<T: AsRef<str>>(&mut self, T) -> Self::VarRef;
}

pub trait RegisterWrite: Sized {
    type VarRef: Clone + Debug + Hash + Eq;
    fn reg_write<T: AsRef<str>>(&mut self, T, Self::VarRef);
}

pub trait MemoryRead: Sized {
    type VarRef: Clone + Debug + Hash + Eq;
    fn mem_read(&mut self, Self::VarRef, u64) -> Self::VarRef;
}

pub trait MemoryWrite: Sized {
    type VarRef: Clone + Debug + Hash + Eq;
    fn mem_write(&mut self, Self::VarRef, Self::VarRef, u64);
}

pub trait Evaluate {
    type VarRef: Clone + Debug + Hash + Eq;
    type IFn: Clone + Debug;

    fn eval<T, Q>(&mut self, T, Q) -> Self::VarRef
        where T: Into<Self::IFn>,
              Q: AsRef<[Self::VarRef]>;
}
