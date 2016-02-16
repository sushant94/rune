//! Implements RuneContext

use context::context_::{Context, MemoryRead, MemoryWrite, RegisterRead, RegisterWrite, ToConcrete};
use smt::smt::{SMTBackend, SMT, SMTResult};
use smt::ssmt::{Solver, SMTInit};

pub type VarRef = usize;

pub struct RuneContext {
    regs: RuneRegfile,
    mem: RuneMem,
}

pub struct RuneRegfile;
pub struct RuneMem;

impl SMT for RuneContext {
    type Idx = VarRef;

    fn solve_for<B: SMTInit>(idx: &VarRef, solver: &mut B) -> SMTResult<u64> {
        unimplemented!()
    }
    
    fn solve_all_for<B: SMTInit>(idx: &VarRef, solver: &mut B) -> SMTResult<Vec<u64>> {
        unimplemented!()
    }

    fn check_sat<B: SMTInit>(&mut self, solver: &mut B) -> SMTResult<bool> {
        unimplemented!()
    }
}

impl ToConcrete<RuneContext> for VarRef {
    fn concrete<B: SMTInit>(&self, ctx: &mut RuneContext) -> u64 {
        unimplemented!()
    }
}

impl RegisterRead for RuneContext {
    type VarRef = VarRef;
    fn read_register<T: AsRef<str>>(&mut self, reg: T) -> Self::VarRef {
        unimplemented!()
    }
}

impl RegisterWrite for RuneContext {
    type VarRef = VarRef;
    fn write_register(&mut self, reg: Self::VarRef, val: Self::VarRef) {
        unimplemented!()
    }
}

impl MemoryRead for RuneContext {
    type VarRef = VarRef;
    fn read_mem<T: SMTInit>(&mut self, loc: VarRef) -> Self::VarRef {
        unimplemented!()
    }
}

impl MemoryWrite for RuneContext {
    type VarRef = VarRef;
    fn write_mem<T: SMTInit>(&mut self, loc: Self::VarRef, info: Self::VarRef) {
        unimplemented!()
    }
}
