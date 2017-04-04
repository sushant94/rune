// Rather than the context having all of that information, let's move the construction logic out
use petgraph::graph::NodeIndex;
use libsmt::logics::qf_abv;

use context::structs::{RuneRegFile};
use radeco_lib::middle::ssa::ssastorage::SSAStorage;
use radeco_lib::middle::ir::{MAddress, MOpcode};
use radeco_lib::middle::ssa::ssa_traits::{ValueType};
use radeco_lib::middle::ssa::cfg_traits::CFGMod;
use radeco_lib::middle::ssa::ssa_traits::{SSAMod};
use std::collections::{BTreeMap, HashMap};

use radeco_lib::frontend::ssaconstructor::VarId;
use radeco_lib::middle::ssa::cfg_traits::CFG;
use radeco_lib::middle::dot;

use std::io::prelude::*;
use std::fs::File;

use radeco_lib::middle::ssa::ssa_traits::SSA;

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
    pub fn new(ssa: SSAStorage, regfile: RuneRegFile, ip: u64) -> PathConstructor {
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

        // Add variable to the HashMap
        {
            let r2 = regfile.regfile.clone();
            let mut whole = Vec::new();
            let mut reg_width: usize;
            for (_, r_entry) in r2 {
                reg_width = r_entry.get_width();
                whole.push(ValueType::Integer { width: reg_width as u16 });
            }
            pc.add_variables(whole);
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
        pc.sync_register_state(start_block);

        // The exit block needs a predecessor block,
        // else we need to add a phi node to specify an incomplete CFG
        // Let's defer the addition of the exit block for after the 
        // creation of the CFG.
        //
        // Emulate add_dynamic to add exit node and mark it
        // let exit_block = pc.add_dynamic();
        
        // Mark the exit node.
        // pc.ssa.mark_exit_node(&exit_block);
        
        // Let's add the main block we are going to work on.
        pc.instruction_offset = 0;
        let next_address = MAddress::new(ip, pc.instruction_offset);
        pc.add_block(next_address, Some(start_address), None);

        // Test creation of the graph
        let tmp = dot::emit_dot(&pc.ssa);
        let mut f = File::create("yay.dot").unwrap();
        f.write_all(tmp.as_bytes()).expect("Write failed.");
        
        pc
    }

    pub fn new_block(&mut self, bb: MAddress) -> NodeIndex {
        if let Some(b) = self.blocks.get(&bb) {
            *b
        } else {
            let block = self.ssa.add_block(bb);
            block
        }
    }
    
    pub fn add_block(&mut self, at: MAddress, current_address: Option<MAddress>, edge_type: Option<u8>) -> NodeIndex {
        let main_block = self.new_block(at);
        let start_block = self.block_of(current_address.unwrap()).unwrap();

        const UNCOND_EDGE: u8 = 2;

        self.ssa.add_control_edge(start_block, main_block, UNCOND_EDGE);
        self.blocks.insert(at, main_block);

        main_block
    }
    
    pub fn add_dynamic(&mut self) -> NodeIndex {
        let action = self.ssa.add_dynamic();
        let dyn_addr = MAddress::new(0xffffffff, 0);
        self.blocks.insert(dyn_addr, action);
        self.sync_register_state(action);
        action
    }

    pub fn current_def_at(&self,
                      variable: VarId,
                      address: MAddress)
                      -> Option<(&MAddress, &NodeIndex)> {
        for (addr, idx) in self.current_def[variable].iter().rev() {
            if self.block_of(*addr) != self.block_of(address) && *addr > address {
                continue;
            }
            return Some((addr, idx));
        }
        None
    }
   
    pub fn current_def_in_block(&self, variable: VarId, address: MAddress) -> Option<&NodeIndex> {
        if let Some(v) = self.current_def_at(variable, address) {
            if self.block_of(*v.0) == self.block_of(address) {
                Some(v.1)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn sync_register_state(&mut self, block: NodeIndex) {
        let rs = self.ssa.registers_at(&block);
        for var in 0..self.variable_types.len() {
            let mut addr = self.addr_of(block);
            let val = self.read_variable(&mut addr, var);
            self.ssa.op_use(rs, var as u8, val);
        }
    }

    fn addr_of(&self, block: NodeIndex) -> MAddress {
        self.ssa.address(&block).unwrap()
    }

    pub fn read_variable(&mut self, address: &mut MAddress, variable: VarId) -> NodeIndex {
        match self.current_def_in_block(variable, *address).cloned() {
            Some(var) => var,
            None => self.read_variable_from_start(variable, address),
        }
    }

    pub fn read_variable_from_start(&mut self, variable: VarId, address: &mut MAddress) -> NodeIndex {
        let block = self.block_of(*address).unwrap();
        let preds = self.ssa.preds_of(block);
        let val = if preds.len() == 1 {
            let mut p_address = self.addr_of(preds[0]);
            self.read_variable(&mut p_address, variable)
        } else {
            panic!("Could not find variable definition in start block.");
        };
        self.write_variable(*address, variable, val);
        val
    }

    pub fn block_of(&self, address: MAddress) -> Option<NodeIndex> {
        let mut last = None;
        let start_address = {
            let start = self.ssa.start_node();
            self.addr_of(start)
        };
        for (baddr, index) in self.blocks.iter().rev() {
            // TODO: Better way to detect start block by using self.ssa.start_block
            // If this is the start block.
            if *baddr == start_address && *baddr != address {
                last = None;
            } else {
                last = Some(*index);
            }
            if *baddr <= address {
                break;
            }
        }
        last
    }

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

    // Core function my bois
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
