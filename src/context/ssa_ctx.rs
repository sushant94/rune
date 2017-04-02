//! Defines the `SSA Context` which uses SSAStorage imported from RadecoLib, 
//! internally uses the SSA Form for representing the path, 
//! which will allow us to apply optimization passes on a `path`

use std::collections::HashMap;
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

use context::utils::{Key, to_key};
use explorer::directed_explorer::BranchType;

use constructor::path_constructor::PathConstructor;

use r2pipe::r2::R2;
use r2pipe::structs::LRegInfo;

#[derive(Clone, Debug)]
pub struct SSAContext
{
    ip: u64,
    pub solver: SMTLib2<qf_abv::QF_ABV>,
    regfile: RuneRegFile,
    mem: RuneMemory,
    pub syms: HashMap<String, NodeIndex>,
    d_map: HashMap<u64, BranchType>,
    e_old: Option<NodeIndex>,
    e_cur: Option<NodeIndex>,
    // A Constructor instance for importing the add_to_path() function which will allow us to
    // construct the path on the fly in the eval loop!
    constructor: PathConstructor,
}

impl Context for SSAContext
{
    fn set_e_old(&mut self, i: NodeIndex) {
        self.e_old = Some(i);
    }
    
    fn set_e_cur(&mut self, i: NodeIndex) {
        self.e_cur = Some(i);
    }

    fn e_old(&self) -> NodeIndex {
        assert!(self.e_old.is_some(), "e_old accessed before being set!");
        self.e_old.unwrap()
    }

    fn e_cur(&self) -> NodeIndex {
        assert!(self.e_cur.is_some(), "e_cur accessed before being set!");
        self.e_cur.unwrap()
    }

    fn ip(&self) -> u64 {
        self.ip
    }

    fn is_symbolic(&self) -> bool {
        true
    }

    fn increment_ip(&mut self, by: u64) {
        self.ip += by;
    }

    fn set_ip(&mut self, to: u64) {
        self.ip = to;
    }

    fn define_const(&mut self, c: u64, size: usize) -> NodeIndex {
        self.solver.new_const(bitvec::OpCodes::Const(c, size))
    }

    fn alias_of(&self, reg: String) -> Option<String> {
        self.regfile.regfile[&reg].alias.clone()
    }

    fn solve<S: SMTProc>(&mut self, p: &mut S) -> HashMap<NodeIndex, u64> {
        self.solver.solve(p).expect("No satisfying solution.")
    }

    fn var_named<T: AsRef<str>>(&self, var: T) -> Option<NodeIndex> {
        self.syms.get(var.as_ref()).cloned()
    }
}

impl RegisterRead for SSAContext {
    type VarRef = NodeIndex;

    fn reg_read<T: AsRef<str>>(&mut self, reg: T) -> NodeIndex {
        self.regfile.read(reg.as_ref(), &mut self.solver)
    }
}

impl RegisterWrite for SSAContext {
    type VarRef = NodeIndex;

    fn reg_write<T: AsRef<str>>(&mut self, reg: T, source: NodeIndex) {
        let e_old = self.regfile.write(reg.as_ref(), source);
        // XXX: THIS IS A HACK!
        if !reg.as_ref().to_owned().ends_with('f') {
            self.e_old = e_old;
            self.e_cur = Some(source);
        }
    }
}

impl MemoryRead for SSAContext {
    type VarRef = NodeIndex;

    fn mem_read(&mut self, addr: NodeIndex, size: u64) -> NodeIndex {
        self.mem.read(addr, size, &mut self.solver)
    }
}

impl MemoryWrite for SSAContext {
    type VarRef = NodeIndex;

    fn mem_write(&mut self, addr: NodeIndex, data: NodeIndex, write_size: u64) {
        self.mem.write(addr, data, write_size, &mut self.solver);
    }
}

impl Evaluate for SSAContext {
    type VarRef = NodeIndex;
    type IFn = qf_abv::QF_ABV_Fn;

    fn eval<T, Q>(&mut self, smt_fn: T, operands: Q) -> Self::VarRef
        where T: Into<Self::IFn> + Clone,
              Q: AsRef<[Self::VarRef]> + Clone
    {
        // Tree construction logic goes in here. 
        // We can access the operands and the current context here.
        self.constructor.add_to_path(smt_fn.clone(), operands.clone());

        // Assertion wil be done after the path has been constructed which will allow us
        // implement further optimizations.
        self.solver.assert(smt_fn, &operands.as_ref())
    }
}

impl ContextAPI for SSAContext {
    fn set_reg_as_const<T: AsRef<str>>(&mut self, reg: T, val: u64) -> NodeIndex {
        let rentry = self.regfile.regfile[reg.as_ref()].clone();
        // Assert that the register is not currently set/defined.
        assert!(self.regfile.current_regs[rentry.idx].is_none());
        let cval = self.define_const(val, 64);
        self.regfile.current_regs[rentry.idx] = Some(cval);
        cval
    }

    fn set_reg_as_sym<T: AsRef<str>>(&mut self, reg: T) -> NodeIndex {
        let rentry = self.regfile.regfile[reg.as_ref()].clone();
        // Assert that the register is not currently set/defined.
        assert!(self.regfile.current_regs[rentry.idx].is_none());
        let sym = self.solver.new_var(Some(reg.as_ref()), qf_abv::bv_sort(64));
        self.regfile.current_regs[rentry.idx] = Some(sym);
        self.syms.insert(reg.as_ref().to_owned(), sym);
        sym
    }

    fn set_mem_as_const(&mut self, addr: usize, val: u64, write_size: u64) -> NodeIndex {
        let cval = self.define_const(val, write_size as usize);
        let addr = self.define_const(addr as u64, 64);
        // TODO
        if write_size < 64 {
            unimplemented!();
        } else {
            self.mem_write(addr, cval, 64);
        }
        cval
    }

    fn set_mem_as_sym(&mut self, addr: usize, write_size: u64) -> NodeIndex {
        assert!(write_size == 64,
                "TODO: Unimplemented set_mem for size < 64!");
        let key = format!("mem_{}", addr);
        let sym = self.solver.new_var(Some(&key), qf_abv::bv_sort(64));
        let addr = self.define_const(addr as u64, 64);
        self.mem_write(addr, sym, write_size);
        self.syms.insert(key, sym);
        sym
    }

    fn zero_registers(&mut self) {
        let cval = Some(self.define_const(0, 64));
        for reg in &mut self.regfile.current_regs {
            if reg.is_none() {
                *reg = cval;
            }
        }
    }

    fn registers(&self) -> Vec<String> {
        unimplemented!();
    }
}

impl SSAContext {
    pub fn new(ip: Option<u64>,
               mem: RuneMemory,
               regfile: RuneRegFile,
               solver: SMTLib2<qf_abv::QF_ABV>)
               -> SSAContext {
        SSAContext {
            ip: ip.unwrap_or(0),
            mem: mem,
            regfile: regfile.clone(),
            solver: solver,
            e_old: None,
            e_cur: None,
            constructor: PathConstructor::new(SSAStorage::new(), regfile),
            syms: HashMap::new(),
            d_map: HashMap::new(),
        }
    }

    pub fn set_decisions(&mut self, decision_list: Vec<(u64, char)>) {
        let mut d_map: HashMap<u64, BranchType> = HashMap::new();

        for tup in decision_list {
            d_map.entry(tup.0).or_insert(BranchType::from(tup.1));
        }

        self.d_map = d_map;
    }

    pub fn initialize(&mut self, stream: &mut R2, ip: Option<u64>, syms: Option<Vec<String>>, consts: Option<HashMap<String, u64>>) {
        stream.set_config_var("asm", "bits", "64");
        stream.set_config_var("asm", "bits", "x86");

        let mut lreginfo = stream.reg_info().expect("Unable to retrieve register information");
        let rregfile = RuneRegFile::new(&mut lreginfo);

        let mut rmem = RuneMemory::new();
        let mut smt = SMTLib2::new(Some(qf_abv::QF_ABV));
        rmem.init_memory(&mut smt);
        let mut ctx = SSAContext::new(ip, rmem, rregfile, smt);

        if let Some(ref sym_vars) = syms {
            for var in sym_vars {
                let _ = match to_key(var) {
                    Key::Mem(addr) => ctx.set_mem_as_sym(addr, 64),
                    Key::Reg(ref reg) => ctx.set_reg_as_sym(reg),
                };
            }
        }

        if let Some(ref const_var) = consts {
            for (k, v) in const_var {
                let _ = match to_key(k) {
                    Key::Mem(addr) => ctx.set_mem_as_const(addr, *v, 64),
                    Key::Reg(ref reg) => ctx.set_reg_as_const(reg, *v),
                };
            }
        }

        // The path would look something like this ->
        // [ Start block (added in the initialize function!)]
        //                    |
        //                    v
        //  [ Main block representing the path we want to explore ]
        //                    |
        //                    v
        //              [ end block ]
/*        let start_address = MAddress::new(0, 0);*/
        //let start_block = ctx.ssa.add_block(start_address);

        //ctx.ssa.mark_start_node(&start_block);

        //for (i, reg) in ctx.regfile.regfile.iter().enumerate() {
            //let vt = ValueType::Integer { width: reg.get_width() };
            //let argnode = ctx.ssa.add_comment(vt, reg.name);
            //ctx.current_def[i].insert(start_address, argnode);
            //ctx.outputs.insert(argnode, i);
        //}

        //// Insert mem as a pseudo variable.
        //let reglen = ctx.regfile.regfile.whole_names.len(); 
        //ctx.set_mem_id(reglen);

        //{
            //let mem_comment = ctx.ssa.add_comment(start_address, ValueType::Integer { width: 0 }, "mem".to_owned());
            //ctx.current_def[i].insert(start_address, mem_comment);
            //ctx.outputs.insert(mem_comment, i);
        //}
    }

    pub fn add_to_path<T, Q>(&mut self, smt_fn: T, operands: Q)
    where T: Into<qf_abv::QF_ABV_Fn> + Clone,
          Q: AsRef<[NodeIndex]>
    {
        // Check init_blocks() and see if it is required in this context 
        // before starting the construction of the path.
        
        // Emulate process_in() and process_op() from ssa_constructor.rs
        // The difference being that there is no involvement of phiplacement.
        // We will be using the decision map wherever necessary for path creation
        // and completely ignore the other branch.
        // One way to approach this would be to directly override the instruction to 
        // `jmp` and not consider the condition under which it jumped.
        // If no decision is maintained, follow the true branch.
        // This behaviour should be changed and both branches should be queued for exploration.
    }
}
