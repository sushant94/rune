//! Defines traits that guides the symbolic emulator

use context::Context;
use std::fmt::Debug;
use r2pipe::structs::LOpInfo;

pub trait InstructionStream {
    type Output: Debug + Clone;
    type Index: Debug + Clone;

    fn next(&self) -> Option<Self::Output>;
    fn at(Self::Index) -> Option<Self::Output>;
}

// TODO
pub struct r2stream;
impl InstructionStream for r2stream {
    type Output = LOpInfo;
    type Index = String;

    fn next(&self) -> Option<LOpInfo> {
        None
    }

    fn at(address: String) -> Option<LOpInfo> {
        None
    }
}

pub trait PathExplorer {
    type I: InstructionStream;

    fn new() -> Self;
    fn next<C: Context>(&mut self, &mut C) -> Option<<Self::I as InstructionStream>::Output>;
}
