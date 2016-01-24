//! Defines traits and structs that perform the actual symbolic emulation.

use context::Context;
use explorer::PathExplorer;

#[derive(Clone, Copy, Debug)]
pub enum EngineError {
    Undefined,
}

pub type EngineResult<T> = Result<T, EngineError>;

pub trait Engine: Sized {
    type C: Context;
    type P: PathExplorer;

    fn new<T: Configure<For=Self>>() -> Self;
    fn run(&mut self) -> EngineResult<()>;
}

pub trait Configure {
    type For: Engine;
    fn configure(&mut Self::For) -> EngineResult<()> {
        Ok(())
    }
}
