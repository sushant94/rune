//! `PathExplorer` that allows interactive exploration

use rune::explorer::explorer::PathExplorer;
use rune::context::rune_ctx::RuneContext;
use rune::engine::rune::RuneControl;
use rune::context::context::{Context, Evaluate, RegisterRead};

use libsmt::theories::core;
use console::Console;

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

#[derive(Debug, Clone, Default)]
pub struct InteractiveExplorer {
    console: Console,
    cmd_q: Vec<Command>,
}

impl PathExplorer for InteractiveExplorer {
    type C = RuneControl;
    type Ctx = RuneContext;

    fn new() -> InteractiveExplorer {
        InteractiveExplorer { 
            cmd_q: Vec::new(),
            console: Default::default(),
        }
    }

    fn next(&mut self, _: &mut Self::Ctx) -> RuneControl {
        RuneControl::Continue
    }

    fn next_job(&mut self, _ctx: &mut Self::Ctx) -> Option<RuneControl> {
        None
    }

    fn register_branch(&mut self,
                       ctx: &mut Self::Ctx,
                       condition: <Self::Ctx as RegisterRead>::VarRef) -> RuneControl {
        if self.cmd_q.is_empty() {
            self.cmd_q = self.console.read_command();
        }

        if let Some(cmd) = self.cmd_q.pop() {
            match cmd {
                Command::FollowTrue => {
                    let one = ctx.define_const(1, 1);
                    ctx.eval(core::OpCodes::Cmp, &[condition, one]);
                    RuneControl::ExploreTrue
                },
                Command::FollowFalse => {
                    let zero = ctx.define_const(0, 1);
                    ctx.eval(core::OpCodes::Cmp, &[condition, zero]);
                    RuneControl::ExploreFalse
                },
                _ => panic!("Incompatible command"),
            }
        } else {
            unreachable!()
        }
    }
}
