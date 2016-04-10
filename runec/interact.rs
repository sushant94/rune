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
    Debug,
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
            'D' => Command::Debug,
            _ => Command::Invalid,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InteractiveExplorer {
    console: Console,
    cmd_q: Vec<Command>,
    single_step: bool,
    // TODO: Remove this breakpointing feature once BPs are implemented.
    pub bp: Vec<u64>,
}

impl InteractiveExplorer {
    pub fn print_debug(&self, ctx: &RuneContext) {
        self.console.print_info("DEBUG");
        self.console.print_info(&format!("Constraints:\n{}", ctx.solver.generate_asserts()));
    }
}

impl PathExplorer for InteractiveExplorer {
    type C = RuneControl;
    type Ctx = RuneContext;

    fn new() -> InteractiveExplorer {
        InteractiveExplorer {
            cmd_q: Vec::new(),
            console: Default::default(),
            single_step: false,
            bp: Vec::new(),
        }
    }

    fn next(&mut self, ctx: &mut Self::Ctx) -> RuneControl {
        if self.single_step || self.bp.contains(&ctx.ip()) {
            self.console.print_info(&format!("Halted at {:#x}", ctx.ip()));
            loop{
                self.single_step = match self.console.read_command()[0] {
                    Command::Step => true,
                    Command::Continue => false,
                    Command::Debug => {
                        self.print_debug(ctx);
                        continue;
                    },
                    _ => {
                        continue;
                    }
                };
                break;
            }
        }
        RuneControl::Continue
    }

    fn next_job(&mut self, _ctx: &mut Self::Ctx) -> Option<RuneControl> {
        None
    }

    fn register_branch(&mut self,
                       ctx: &mut Self::Ctx,
                       condition: <Self::Ctx as RegisterRead>::VarRef)
                       -> RuneControl {
        if self.cmd_q.is_empty() {
            self.console.print_info(&format!("Encountered Branch At {:#x}", ctx.ip()));
            self.cmd_q = self.console.read_command();
        }

        if let Some(cmd) = self.cmd_q.pop() {
            match cmd {
                Command::FollowTrue => {
                    let one = ctx.define_const(1, 1);
                    ctx.eval(core::OpCodes::Cmp, &[condition, one]);
                    RuneControl::ExploreTrue
                }
                Command::FollowFalse => {
                    let zero = ctx.define_const(0, 1);
                    ctx.eval(core::OpCodes::Cmp, &[condition, zero]);
                    RuneControl::ExploreFalse
                }
                _ => panic!("Incompatible command"),
            }
        } else {
            unreachable!()
        }
    }
}
