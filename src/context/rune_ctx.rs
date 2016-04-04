//! Defines `RuneContext`

use std::collections::HashMap;

use r2pipe::structs::LRegInfo;
use petgraph::graph::NodeIndex;
use libsmt::backends::smtlib2::SMTLib2;
use libsmt::backends::backend::SMTBackend;
use libsmt::logics::qf_abv;
use libsmt::theories::{array_ex, bitvec, core};

use context::context::{Context, Evaluate, MemoryRead, MemoryWrite, RegisterRead, RegisterWrite};

// TODO: Handle symbolic jumps

#[derive(Clone, Debug)]
pub struct RuneContext {
    ip: u64,
    solver: SMTLib2<qf_abv::QF_ABV>,
    regfile: RuneRegFile,
    mem: RuneMemory,
}

#[derive(Clone, Debug, Default)]
pub struct RuneMemory {
    map: Option<NodeIndex>,
}

#[derive(Clone, Debug, Default)]
struct RegEntry {
    name: String,
    idx: usize,
    // 0 indexed
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
                                                 register.type_str == "gpr" {
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
        let rentry = &self.regfile[reg_name];
        let idx = self.current_regs[rentry.idx].unwrap();
        if rentry.is_whole {
            idx
        } else {
            solver.assert(bitvec::OpCodes::Extract((rentry.end_bit + 1) as u64, 1),
                          &[idx])
        }
    }

    // TODO: This is not totally correct as the sizes of registers may not match.
    fn write(&mut self,
             dest: &str,
             source: NodeIndex) {
        let rentry = &self.regfile[dest];
        self.current_regs[rentry.idx] = Some(source);
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
        let idx = solver.assert(core::OpCodes::Cmp, &[idx_, const_0]);
        self.map = Some(idx);
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

impl Context for RuneContext {
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
}

impl RegisterRead for RuneContext {
    type VarRef = NodeIndex;

    fn reg_read<T: AsRef<str>>(&mut self, reg: T) -> NodeIndex {
        self.regfile.read(reg.as_ref(), &mut self.solver)
    }
}

impl RegisterWrite for RuneContext {
    type VarRef = NodeIndex;

    fn reg_write<T: AsRef<str>>(&mut self, reg: T, source: NodeIndex) {
        self.regfile.write(reg.as_ref(), source);
    }
}

impl MemoryRead for RuneContext {
    type VarRef = NodeIndex;

    fn mem_read(&mut self, addr: NodeIndex, size: u64) -> NodeIndex {
        self.mem.read(addr, size, &mut self.solver)
    }
}

impl  MemoryWrite for RuneContext {
    type VarRef = NodeIndex;

    fn mem_write(&mut self, addr: NodeIndex, data: NodeIndex, write_size: u64) {
        self.mem.write(addr, data, write_size, &mut self.solver);
    }
}

impl Evaluate for RuneContext {
    type VarRef = NodeIndex;
    type IFn = qf_abv::QF_ABV_Fn;

    fn eval<T, Q>(&mut self, smt_fn: T, operands: Q) -> Self::VarRef
        where T: Into<Self::IFn>,
              Q: AsRef<[Self::VarRef]>
    {
        self.solver.assert(smt_fn, &operands.as_ref())
    }
}
