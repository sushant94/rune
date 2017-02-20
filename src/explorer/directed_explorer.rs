//! `DirectedExplorer`, an implementation of a `PathExplorer` which allows directed symbolic
//! execution

use explorer::explorer::PathExplorer;
use context::ssa_ctx::SSAContext;
use engine::rune::RuneControl;
use petgraph::dot::{Dot, Config};
use context::context::{Context, Evaluate, MemoryRead, RegisterRead};

use libsmt::theories::{bitvec, core};
use libsmt::logics::qf_abv::QF_ABV_Fn;
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

impl PathExplorer for DirectedExplorer {
    type C = RuneControl;
    type Ctx = SSAContext;

   fn new() -> Self {
       DirectedExplorer {
           d_map: HashMap::new(),
           break_addr: 0x0
       }
   }

   
    fn next(&mut self, ctx: &mut Self::Ctx) -> RuneControl {
        // Automated continuous exploration my bois
        // TODO: Add tree construction logic?
        // What should the logic contain?
        // self.ssa_form is SSAStorage which should be modified?
        // This has to be modified based on the current context which is ctx
        // The tree here is a path which should be in the form available for 
        // IR optimization.
        // println!("{:#x}\n*************", ctx.ip());

        if ctx.ip() == self.break_addr {
            let mut z3: z3::Z3 = Default::default();
            // println!("{:?}", ctx.solver);
            // println!("{:?}", Dot::with_config(&ctx.solver.gr, &[Config::EdgeNoLabel]));
            let res = ctx.solve(&mut z3);
            // println!("{:?}", ctx.solver());
            // println!("YAYAYA");
            for (key, val) in &res {
                println!("{}", val);
            }

            // println!("{:?}", ctx);
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

    pub fn set_break(&mut self, addr: u64) {
        self.break_addr = addr;
    }
}

#[cfg(test)]
mod test {
    use engine::rune::Rune; 
    use engine::engine::Engine;
    use context::ssa_ctx;
    use std::collections::HashMap;
    use r2pipe::r2::R2;
    use explorer::explorer::PathExplorer;
    use context::context::ContextAPI;
    use context::context::Context;
    
    use super::*;

    #[test]
    fn check_decision_map_generation() {
        let mut temp: Vec<(u64, BranchType)> = Vec::new();
        temp.push((0x1234, BranchType::from('T')));

        // let a = DirectedExplorer::new(temp);
        // TODO: add assertion
    }

    #[test]
    fn directed_explorer_test() {
        let mut stream = R2::new(Some("./test_files/test")).expect("Unable to spawn r2");
        stream.init();

        let mut var_map: HashMap<String, u64> = HashMap::new();
        var_map.insert("rbp".to_owned(), 256);
        var_map.insert("rsp".to_owned(), 512);
        var_map.insert("of".to_owned(), 0);
        var_map.insert("cf".to_owned(), 0);
        var_map.insert("zf".to_owned(), 0);
        var_map.insert("pf".to_owned(), 0);
        var_map.insert("sf".to_owned(), 0);
        var_map.insert("rax".to_owned(), 0);
        var_map.insert("rdx".to_owned(), 0);
        var_map.insert("rsi".to_owned(), 0);
        
        let ctx = ssa_ctx::new_ssa_ctx(Some(0x0040050a), Some(Vec::new()), Some(var_map.clone()));
        let mut explorer = DirectedExplorer::new();
        
        let mut v: Vec<(u64, char)> = Vec::new();
        v.push((0x00400526, 'F'));

        explorer.set_decisions(v);

        let mut rune = Rune::new(ctx, explorer, stream);
        rune.run().expect("Rune Error");
    }

    #[test]
    fn crackme_test() {
        let mut stream = R2::new(Some("./test_files/crackme2")).expect("Unable to spawn r2");
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
        var_map.insert("rdi".to_owned(), 0);
        var_map.insert("rcx".to_owned(), 0);

        
        let mut ctx = ssa_ctx::new_ssa_ctx(Some(0x100000e71), Some(Vec::new()), Some(var_map.clone()));
        let mut explorer = DirectedExplorer::new();
        
        let mut v: Vec<(u64, char)> = Vec::new();
        // pie
        // v.push((0x0040057b, 'T'));
        // v.push((0x004005c3, 'F'));
        // v.push((0x004005e8, 'F'));
        // v.push((0x004005fb, 'T'));
        // v.push((0x00400625, 'F'));
        // v.push((0x0040062f, 'F'));
        // nopie
        v.push((0x100000e85, 'F'));
        v.push((0x100000ea0, 'F'));
        v.push((0x100000ebb, 'T'));
        v.push((0x100000ed9, 'F'));

        explorer.set_decisions(v);

        let break_addr = 0x100000ea0;
        explorer.set_break(break_addr);

        let mut vec: Vec<u64> = Vec::new();
        vec.push(0x8fd8);
        vec.push(0x8fd0);
        // vec.push(0x8fc8);
        vec.push(0x8fc0);

        ctx.set_mem_as_const(0x8fec, 64, 64);

        for addr in vec {
            ctx.set_mem_as_sym(addr as usize, 64);
        }

        let mut rune = Rune::new(ctx, explorer, stream);
        rune.run().expect("Rune Error");
    }
}
