// Rather than the context having all of that information, let's move the construction logic out
use std::fmt::Debug;

use petgraph::graph::NodeIndex;
use libsmt::backends::smtlib2::{SMTLib2, SMTProc};
use libsmt::backends::backend::SMTBackend;
use libsmt::logics::qf_abv;
use libsmt::theories::{array_ex, bitvec, core};

use context::context::{Context, ContextAPI, Evaluate, MemoryRead, MemoryWrite, RegisterRead,
                       RegisterWrite};
use context::structs::{RuneRegFile, RuneMemory};
use engine::rune::RuneControl;
use radeco_lib::middle::ssa::ssastorage::SSAStorage;
use radeco_lib::frontend::ssaconstructor::SSAConstruct;
use radeco_lib::middle::ir::{MAddress, MOpcode};
use radeco_lib::middle::ssa::ssa_traits::{ValueType};
use radeco_lib::middle::ssa::cfg_traits::CFGMod;
use radeco_lib::middle::ssa::ssa_traits::{SSAExtra, SSAMod};
use radeco_lib::middle::phiplacement::PhiPlacer;
use esil::parser;
use esil::lexer::{Token};
use std::collections::{BTreeMap, HashMap};

use context::utils::{Key, to_key};
use explorer::directed_explorer::BranchType;
use radeco_lib::frontend::ssaconstructor::VarId;

use r2pipe::r2::R2;
use r2pipe::structs::LRegInfo;

#[derive(Clone, Debug)]
pub struct PathConstructor
{
    ssa: SSAStorage,
    pub variable_types: Vec<ValueType>,
    current_def: Vec<BTreeMap<MAddress, NodeIndex>>,
    // There is a bit of duplication here in that there is a regfile which is a member of the
    // Context and there there is a regfile which is a member of the Constructor which is again a
    // part of the context.
    regfile: RuneRegFile,
    alias_info: HashMap<String, String>,
    constants: HashMap<u64, NodeIndex>,
    ident_map: HashMap<String, u64>,
    // We won't need nesting here since ITE is resolved
    instruction_offset: u64,
    // needs_new_block is not needed since that case will not arise.
    mem_id: usize,
    pub blocks: BTreeMap<MAddress, NodeIndex>,
    pub index_to_addr: HashMap<NodeIndex, MAddress>,
    addr_to_index: BTreeMap<MAddress, NodeIndex>,
    outputs: HashMap<NodeIndex, VarId>,
}

impl PathConstructor {
    pub fn new(ssa: SSAStorage, regfile: RuneRegFile) -> PathConstructor {
        let mut pc = PathConstructor {
            ssa: ssa,
            variable_types: Vec::new(),
            current_def: Vec::new(),
            regfile: regfile.clone(),
            alias_info: HashMap::new(),
            constants: HashMap::new(),
            ident_map: HashMap::new(),
            instruction_offset: 0,
            mem_id: 0,
            blocks: BTreeMap::new(),
            index_to_addr: HashMap::new(),
            addr_to_index: BTreeMap::new(),
            outputs: HashMap::new(),
        };

        // Assuming we dont need alias_info and we will be retrieving that information from the
        // RuneRegFile based alias_info. Doesn't really matter from where it comes from.
        {
            let r1 = regfile.regfile.clone();
            let identmap = &mut pc.ident_map;
            for (r_name, entry) in r1 {
                let reg_width = entry.get_width();
                identmap.insert(r_name, reg_width as u64);
            }
        } 

        {
            let r2 = regfile.regfile.clone();
            let mut whole: Vec<ValueType> = Vec::new();
            for (r_name, entry) in r2 {
                let reg_width = entry.get_width();
                whole.push(ValueType::Integer { width: reg_width as u16 });
            }
            
            pc.add_variables(whole);
        }

        // Add "mem" type variable
        pc.add_variables(vec![ValueType::Integer { width: 0 }]);

        pc
    }

    pub fn add_to_path<A, B>(&mut self, smt_fn: A, operands: B)
        where A: Into<qf_abv::QF_ABV_Fn> + Clone,
              B: AsRef<[NodeIndex]>
    {
    }

    pub fn add_variables(&mut self, variable_types: Vec<ValueType>) {
        for _ in &variable_types {
            self.current_def.push(BTreeMap::new());
        }
        self.variable_types.extend(variable_types);
    }
}

