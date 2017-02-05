//! Defines the `SSA Context` which uses the SSAStorage which
//! internally uses the SSA Form for representing the path, which will allow us to apply
//! optimization passes on a `path`

use std::collections::HashMap;

use r2pipe::structs::LRegInfo;
use petgraph::graph::NodeIndex;
use libsmt::backends::smtlib2::{SMTLib2, SMTProc};
use libsmt::backends::backend::SMTBackend;
use libsmt::logics::qf_abv;
use libsmt::theories::{array_ex, bitvec, core};

use context::context::{Context, ContextAPI, Evaluate, MemoryRead, MemoryWrite, RegisterRead,
                       RegisterWrite};
use engine::rune::RuneControl;
use radeco_lib::middle::ssa::ssastorage::SSAStorage;
use radeco_lib::frontend::ssaconstructor::SSAConstruct;
use esil::parser;

#[derive(Clone, Debug)]
pub struct SSAContext {
    ip: u64,
    pub solver: SMTLib2<qf_abv::QF_ABV>,
    regfile: RuneRegFile,
    mem: RuneMemory,
    pub syms: HashMap<String, NodeIndex>,
    ssa_form: SSAStorage,
    e_old: Option<NodeIndex>,
    e_cur: Option<NodeIndex>,
}

#[derive(Clone, Debug, Default)]
pub struct RuneMemory {
    map: Option<NodeIndex>,
}

#[derive(Clone, Debug, Default)]
struct RegEntry {
    name: String,
    idx: usize,
    start_bit: usize,
    end_bit: usize,
    is_whole: bool,
    alias: Option<String>,
}

impl RegEntry {
    fn new(name: String,
           idx: usize,
           sbit: usize,
           ebit: usize,
           is_whole: bool,
           alias: Option<String>)
        -> RegEntry {
        RegEntry {
            name: name,
            idx: idx,
            start_bit: sbit,
            end_bit: ebit,
            is_whole: is_whole,
            alias: alias,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RuneRegFile {
    current_regs: Vec<Option<NodeIndex>>,
    regfile: HashMap<String, RegEntry>,
    alias_info: HashMap<String, String>,
}

impl RuneRegFile {
    pub fn new(reginfo: &mut LRegInfo) -> RuneRegFile {
        let mut cur_regs = Vec::new();
        let mut regfile = HashMap::new();
        let mut seen_offsets = Vec::new();
        let mut alias_info = HashMap::new();
        reginfo.reg_info.sort_by(|x, y| (y.offset + y.size).cmp(&(x.offset + x.size)));
        for register in &reginfo.reg_info {
            let (idx, s_bit, e_bit, is_whole) = if !seen_offsets.contains(&register.offset) &&
                                                   (register.type_str == "gpr" || register.type_str == "flg") {
                cur_regs.push(None);
                seen_offsets.push(register.offset);
                (cur_regs.len() - 1, 0, register.size - 1, true)
            } else {
                let mut found = 0;
                for (i, offset) in seen_offsets.iter().enumerate() {
                    if register.offset == *offset {
                        found = i;
                        break;
                    }
                }
                (found, 0, register.size - 1, false)
            };

            regfile.insert(register.name.clone(),
                           RegEntry::new(register.name.clone(), idx, s_bit, e_bit, is_whole, None));
        }

        for alias in &reginfo.alias_info {
            alias_info.insert(alias.role_str.clone(), alias.reg.clone());
            // Add this alias info in the corresponding RegEntry too.
            if let Some(info) = regfile.get_mut(&alias.reg) {
                info.alias = Some(alias.role_str.clone());
            }
        }

        RuneRegFile {
            current_regs: cur_regs,
            regfile: regfile,
            alias_info: alias_info,
        }
    }

    fn read(&mut self, reg_name: &str, solver: &mut SMTLib2<qf_abv::QF_ABV>) -> NodeIndex {
        let rentry = &self.regfile.get(reg_name).expect("Unknown Register");
        let idx = self.current_regs[rentry.idx].expect("Unset register - Undefined Behavior. \
                                                        Consider setting an initial value before use!");
        if rentry.is_whole {
            idx
        } else {
            solver.assert(bitvec::OpCodes::Extract((rentry.end_bit) as u64, 0), &[idx])
        }
    }

    // TODO: This is not totally correct as the sizes of registers may not match.
    fn write(&mut self, dest: &str, source: NodeIndex) -> Option<NodeIndex> {
        let rentry = &self.regfile[dest];
        let e_old = self.current_regs[rentry.idx];
        self.current_regs[rentry.idx] = Some(source);
        e_old
    }
}

impl RuneMemory {
    pub fn new() -> RuneMemory {
        RuneMemory { map: None }
    }

    pub fn init_memory(&mut self, solver: &mut SMTLib2<qf_abv::QF_ABV>) {
        let bv_array = qf_abv::array_sort(qf_abv::bv_sort(64), qf_abv::bv_sort(64));
        let idx_ = solver.new_var(Some("mem"), bv_array);
        // Set memory to all 0s
        let arr_const_ty = qf_abv::array_const(qf_abv::bv_sort(64),
                                               qf_abv::bv_sort(64),
                                               bitvec::OpCodes::Const(0, 64));
        let const_0 = solver.new_const(arr_const_ty);
        solver.assert(core::OpCodes::Cmp, &[idx_, const_0]);
        self.map = Some(idx_);
    }

    pub fn read(&mut self,
                addr: NodeIndex,
                read_size: u64,
                solver: &mut SMTLib2<qf_abv::QF_ABV>)
                -> NodeIndex {
        if self.map.is_none() {
            self.init_memory(solver);
        }
        let mem = self.map.unwrap();
        let idx = solver.assert(array_ex::OpCodes::Select, &[mem, addr]);
        if read_size < 64 {
            solver.assert(bitvec::OpCodes::Extract(read_size - 1, 1), &[idx])
        } else {
            idx
        }
    }

    // TODO: Need to handle the case where write_size is not 64.
    pub fn write(&mut self,
                 addr: NodeIndex,
                 data: NodeIndex,
                 _write_size: u64,
                 solver: &mut SMTLib2<qf_abv::QF_ABV>) {
        if self.map.is_none() {
            self.init_memory(solver);
        }
        let mem = self.map.unwrap();
        let new_mem = solver.assert(array_ex::OpCodes::Store, &[mem, addr, data]);
        self.map = Some(new_mem);
    }
}

impl Context for SSAContext {
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

impl  MemoryWrite for SSAContext {
    type VarRef = NodeIndex;

    fn mem_write(&mut self, addr: NodeIndex, data: NodeIndex, write_size: u64) {
        self.mem.write(addr, data, write_size, &mut self.solver);
    }
}

impl Evaluate for SSAContext {
    type VarRef = NodeIndex;
    type IFn = qf_abv::QF_ABV_Fn;

    fn eval<T, Q>(&mut self, smt_fn: T, operands: Q) -> Self::VarRef
        where T: Into<Self::IFn>,
              Q: AsRef<[Self::VarRef]>
    {
        // TODO: Add extract / concat to ensure that the registers are of compatible
        // sizes for
        // operations.
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
            regfile: regfile,
            solver: solver,
            e_old: None,
            e_cur: None,
            syms: HashMap::new(),
            ssa_form: SSAStorage::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use context::context::{Context, ContextAPI, Evaluate, MemoryRead, MemoryWrite, RegisterRead,
                           RegisterWrite};
    use context::utils;

    use libsmt::logics::qf_abv;
    use libsmt::backends::smtlib2::SMTLib2;
    use libsmt::backends::backend::SMTBackend;
    use libsmt::theories::{bitvec, core};
    use libsmt::backends::z3;
}
