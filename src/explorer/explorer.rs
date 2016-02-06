//! Defines traits that guides the symbolic emulator

use context::Context;
use std::fmt::Debug;

pub trait PathExplorer {
    type C: Clone + Debug;
    type Ctx: Context;

    fn new() -> Self;
    fn next(&mut self, &mut Self::Ctx) -> Self::C;
    fn next_job(&mut self, &mut Self::Ctx) -> Option<Self::C>;

    fn register_branch(&mut self, &mut Self::Ctx) -> Self::C;
}
