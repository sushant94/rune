//! `DirectedExplorer`, an implementation of a `PathExplorer` which allows directed symbolic
//! execution

use explorer::explorer::PathExplorer;
use context::ssa_ctx::SSAContext;
use engine::rune::RuneControl;
use context::context::{Context, Evaluate, RegisterRead};

use libsmt::theories::{core};
use libsmt::backends::z3;

use std::collections::HashMap;

// I know it is code repitition, but whatevs
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BranchType {
    True,
    False,
    Invalid,
}

// Allows us to maybe provide a function for the user
// to choose which branch to take based on the character.
impl From<char> for BranchType {
    fn from(c: char) -> BranchType {
        match c {
            'T' => BranchType::True,
            'F' => BranchType::False,
            _ => BranchType::Invalid,
        }
    }
}

// For now, let us assume that we have the directed path exploration
// information taken from the user.
// Consider the map of the form Address: BranchType.
// This will allow us to choose the ideal branch to be taken at each 
// instance of branching into a different path
#[derive(Debug, Clone, Default)]
pub struct DirectedExplorer {
    pub d_map: HashMap<u64, BranchType>,
    pub break_addr: u64,
    // Ideally we should give control back to the user based on
    // his choice to stop/start execution somewhere.
}

impl PathExplorer for DirectedExplorer
{
    type C = RuneControl;
    type Ctx = SSAContext;

   fn new() -> Self {
       DirectedExplorer {
           d_map: HashMap::new(),
           break_addr: 0x0000,
       }
   }

   
    fn next(&mut self, ctx: &mut Self::Ctx) -> RuneControl {
        // Automated continuous exploration my bois
        if ctx.ip() == self.break_addr {
            let mut z3: z3::Z3 = Default::default();
            println!("{:?}", ctx.solver.generate_asserts());
            println!("Attempting to solve constraints:");
            let result = ctx.solve(&mut z3);
            println!("SAT. Solutions: ");

            for (_, val) in result {
                println!("{}", val);
            }
            return RuneControl::Halt
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
            // println!("{:#x}", ctx.ip());
            if self.d_map.contains_key(&ctx.ip()) {
                let direction = self.d_map.get(&ctx.ip()).unwrap();
                match *direction {
                    BranchType::True => {
                        let one = ctx.define_const(1, 64);
                        ctx.eval(core::OpCodes::Cmp, &[condition, one]);
                        RuneControl::ExploreTrue
                    }
                    BranchType::False => {
                        let zero = ctx.define_const(0, 64);
                        ctx.eval(core::OpCodes::Cmp, &[condition, zero]);
                        RuneControl::ExploreFalse
                    }
                    _ => panic!("Invalid branch type found!"),
                }
            } else {
                panic!("Do not know which branch to take. Set a default!");
            }
    }
}

impl DirectedExplorer {
    pub fn set_decisions(&mut self, decision_list: Vec<(u64, char)>) {
        let mut d_map: HashMap<u64, BranchType> = HashMap::new();
        
        for tup in decision_list {
            d_map.entry(tup.0).or_insert(BranchType::from(tup.1));
        }

        self.d_map = d_map;
    }
}

#[cfg(test)]
mod test {
    use engine::rune::Rune; 
    use engine::engine::Engine;
    use context::ssa_ctx;
    use std::collections::HashMap;
    use r2pipe::r2::R2;
    use context::context::ContextAPI; 
    use super::*;

    #[test]
    fn check_decision_map_generation() {
        let mut temp: Vec<(u64, BranchType)> = Vec::new();
        temp.push((0x1234, BranchType::from('T')));

        // let a = DirectedExplorer::new(temp);
        // TODO: add assertion
    }

    #[test] 
    fn directed_test() {
        let mut stream = R2::new(Some("./test_files/test")).expect("Unable to spawn r2");
        stream.init();

        let mut var_map: HashMap<String, u64> = HashMap::new();
        var_map.insert("rbp".to_owned(), 0x9000);
        var_map.insert("rsp".to_owned(), 512);
        var_map.insert("of".to_owned(), 0);
        var_map.insert("cf".to_owned(), 0);
        var_map.insert("zf".to_owned(), 0);
        var_map.insert("pf".to_owned(), 0);
        var_map.insert("sf".to_owned(), 0);
        var_map.insert("rax".to_owned(), 0);
        var_map.insert("rdx".to_owned(), 0);
        var_map.insert("rsi".to_owned(), 0);
        
        let ctx = ssa_ctx::new_ssa_ctx(Some(0x0040060a), Some(Vec::new()), Some(var_map.clone()));
        let mut explorer = DirectedExplorer::new();
        
        let mut v: Vec<(u64, char)> = Vec::new();
        v.push((0x0040061b, 'T'));

        explorer.set_decisions(v);

        let mut rune = Rune::new(ctx, explorer, stream);
        rune.run().expect("Rune Error");
    }
}
