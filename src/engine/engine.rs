//! Defines traits and structs that perform the actual symbolic emulation.

use context::context::Context;
use explorer::explorer::PathExplorer;

#[derive(Clone, Copy, Debug)]
pub enum EngineError {
    Undefined,
    InCorrectOperand,
}

pub type EngineResult<T> = Result<T, EngineError>;

pub trait Engine: Sized {
    fn run(&mut self) -> EngineResult<()>;
}

pub trait Configure {
    type For: Engine;
    fn configure(&mut Self::For) -> EngineResult<()> {
        Ok(())
    }
}
