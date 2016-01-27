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
pub struct R2Stream;
impl InstructionStream for R2Stream {
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
    type C: Clone + Debug;

    fn new() -> Self;
    fn next<C: Context>(&mut self, &mut C) -> Option<<Self::I as InstructionStream>::Output>;

    fn register_branch<C: Context>(&mut self, C::BV, &mut C) -> Self::C;
}
