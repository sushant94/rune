//! `PathExplorer` that allows interactive exploration

use std::io::{self, Read};

use rune::explorer::explorer::PathExplorer;
use rune::context::rune_ctx::RuneContext;
use rune::engine::rune::RuneControl;
use rune::context::context::{Evaluate, RegisterRead};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    FollowTrue,
    FollowFalse,
    Continue,
    Step,
    Invalid,
}

impl Command {
    pub fn is_invalid(&self) -> bool {
        *self == Command::Invalid
    }

    pub fn is_valid(&self) -> bool {
        !self.is_invalid()
    }
}

impl From<char> for Command {
    fn from(c: char) -> Command {
        match c {
            'T' => Command::FollowTrue,
            'F' => Command::FollowFalse,
            'C' => Command::Continue,
            'S' => Command::Step,
            _   => Command::Invalid,
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct InteractiveExplorer;

impl PathExplorer for InteractiveExplorer {
    type C = RuneControl;
    type Ctx = RuneContext;

    fn new() -> InteractiveExplorer {
        InteractiveExplorer { }
    }

    fn next(&mut self, _: &mut Self::Ctx) -> RuneControl {
        RuneControl::Continue
    }

    fn next_job(&mut self, ctx: &mut Self::Ctx) -> Option<RuneControl> {
        None
    }

    fn register_branch(&mut self,
                       ctx: &mut Self::Ctx,
                       condition: <Self::Ctx as RegisterRead>::VarRef) -> RuneControl {
        unimplemented!()
    }
}
