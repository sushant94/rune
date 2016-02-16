//! PathExplorer that works by exploring the CFG in Depth First Order.

use std::collections::VecDeque;

use explorer::explorer::{PathExplorer};
use engine::rune::RuneControl;
use context::context::Context;
use stream::{InstructionStream};

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
enum BranchType {
    True,
    False,
}

#[derive(Clone, Debug)]
struct SavedState<C: Context> {
    pub ctx: C,
    pub branch: BranchType,
}

impl<C: Context> SavedState<C> {
    fn new(ctx: C, b: BranchType) -> SavedState<C> {
        SavedState {
            ctx: ctx,
            branch: b
        }
    }
}

/// An explorer that traverses the program states in a depth first order.
pub struct DFSExplorer<Ctx: Context> {
    /// Depth First Queue
    queue: VecDeque<SavedState<Ctx>>,
}

impl<Ctx> PathExplorer for DFSExplorer<Ctx>
where Ctx: Context {
    type C = RuneControl;
    type Ctx = Ctx;

    fn new() -> DFSExplorer<Ctx> {
        DFSExplorer {
            queue: VecDeque::new(),
        }
    }

    // TODO: Terminate the current execution path if the depth is greater than a preset threshold.
    fn next(&mut self, _: &mut Self::Ctx) -> RuneControl {
        RuneControl::Continue
    }

    // When rune finishes its execution, pop another unexplored path for it to explore.
    fn next_job(&mut self, ctx: &mut Self::Ctx) -> Option<RuneControl> {
        if let Some(ref state) = self.queue.pop_back() {
            *ctx = state.ctx.clone();
            Some(
                match state.branch {
                    BranchType::True => RuneControl::ExploreTrue,
                    BranchType::False => RuneControl::ExploreFalse,
                })
        } else {
            None
        }
    }

    fn register_branch(&mut self, ctx: &mut Self::Ctx) -> RuneControl {
        // When a new branch is encountered, push the false branch into the queue and explore the
        // true branch. Note that this choice is arbitrary and we could have as well chosen the
        // other part without changing the nature of this explorer.
        self.queue.push_back(SavedState::new(ctx.clone(), BranchType::False));
        RuneControl::ExploreTrue
    }
}
