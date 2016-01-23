//! Defines traits that guides the symbolic emulator

use context::Context;

pub trait PathExplorer {
    type C: Context;

    fn new() -> Self;
    fn next(&mut self, &mut Self::C) -> String;
}
