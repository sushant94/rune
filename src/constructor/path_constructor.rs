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
use radeco_lib::middle::ssa::cfg_traits::CFG;

use radeco_lib::middle::ssa::ssa_traits::SSA;

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

        // The path would look something like this ->
        // [ Start block (added here) ]
        //          |
        //          v
        //  [ Main block representing the path we want to explore ]
        //          |
        //          v
        //  [ end block ]

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
        
        // Add "mem" type variable
        pc.add_variables(vec![ValueType::Integer { width: 0 }]);

        // Emulating init_blocks
        let start_address = MAddress::new(0, 0);
        let start_block = pc.ssa.add_block(start_address);

        pc.blocks.insert(start_address, start_block);
        pc.ssa.mark_start_node(&start_block);

        let r1 = pc.regfile.regfile.clone();

         for (i, reg) in r1.iter().enumerate() {
            let (reg_name, reg_entry) = reg; 
            let vt = ValueType::Integer { width: reg_entry.get_width() as u16 };
            let reg_comment = pc.add_comment(start_address, vt, reg_name.to_owned());
            pc.write_variable(start_address, i, reg_comment);
        }
        
        {
            let reglen = pc.regfile.regfile.len();
            pc.set_mem_id(reglen);
            let mem_comment = pc.add_comment(start_address, ValueType::Integer { width: 0 }, "mem".to_owned());
            pc.write_variable(start_address, reglen, mem_comment);
        }

        // Emulating sync_register_state
        
        
        // Emulate add_dynamic to add exit node and mark it

        pc
    }

    pub fn sync_register_state(&mut self, block: NodeIndex) {
        let rs = self.ssa.registers_at(&block);
        for val in 0..self.variable_types.len() {
            let mut addr = self.addr_of(block);
            // let val = self.read_variable(&mut addr, var);
            // self.ssa.op_use(rs, var as u8, val);
        }
    }

    fn addr_of(&self, block: NodeIndex) -> MAddress {
        self.ssa.address(&block).unwrap()
    }

    // TODO: READ_VARIABLE

    pub fn write_variable(&mut self, address: MAddress, variable: VarId, value: NodeIndex) {
        self.current_def[variable].insert(address, value);
        self.outputs.insert(value, variable);
    }

    pub fn add_comment(&mut self, address: MAddress, vt: ValueType, msg: String) -> NodeIndex {
        let node = self.ssa.add_comment(vt, msg);
        self.index_to_addr.insert(node, address);
        node
    }

    pub fn set_mem_id(&mut self, id: usize) {
        assert_eq!(self.mem_id, 0);
        self.mem_id = id;
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

