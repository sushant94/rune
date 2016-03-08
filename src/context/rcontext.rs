//! Implementation of context for rune.

use std::collections::{BTreeMap, HashMap};

use std::fmt::Debug;

use r2pipe::structs::LRegInfo;
use petgraph::graph::NodeIndex;
use libsmt::ssmt::{SMTLib2, SMTSolver};
use libsmt::smt::SMTBackend;
use libsmt::logics::qf_abv;

use libsmt::theories::{array_ex, bitvec, core};

pub struct RuneContext {
    solver: SMTLib2<qf_abv::QF_ABV>,
    current_regs: RuneRegFile,
    mem: RuneMemory,
}

#[derive(Clone, Debug)]
pub struct RuneMemory {
    map: Option<NodeIndex>,
}

#[derive(Clone, Debug)]
struct RegEntry {
    name: String,
    idx: usize,
    // 0 indexed
    start_bit: usize,
    end_bit: usize,
    is_whole: bool,
}

impl RegEntry {
    fn new(name: String, idx: usize, sbit: usize, ebit: usize, is_whole: bool) -> RegEntry {
        RegEntry {
            name: name,
            idx: idx,
            start_bit: sbit,
            end_bit: ebit,
            is_whole: is_whole,
        }
    }
}

#[derive(Clone, Debug)]
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
            let (idx, sbit, ebit, is_whole) = if !seen_offsets.contains(&register.offset) &&
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
                           RegEntry::new(register.name.clone(), idx, sbit, ebit, is_whole));
        }

        for alias in &reginfo.alias_info {
            alias_info.insert(alias.role_str.clone(), alias.reg.clone());
        }

        RuneRegFile {
            current_regs: cur_regs,
            regfile: regfile,
            alias_info: alias_info,
        }
    }

    pub fn read(&mut self, reg_name: String, solver: &mut SMTLib2<qf_abv::QF_ABV>) -> NodeIndex {
        let rentry = &self.regfile[&reg_name];
        let idx = self.current_regs[rentry.idx].unwrap();
        if !rentry.is_whole {
            solver.assert(bitvec::OpCodes::Extract((rentry.end_bit + 1) as u64, 1),
                          &[idx])
        } else {
            idx
        }
    }

    pub fn write(&mut self,
                 dest: String,
                 source: NodeIndex,
                 solver: &mut SMTLib2<qf_abv::QF_ABV>)
                 -> NodeIndex {
        let rentry = &self.regfile[&dest];
        if !rentry.is_whole {
            let widx = self.current_regs[rentry.idx].unwrap();
            // Add extract operations to trim to correct size of the register.
            let idx = solver.assert(bitvec::OpCodes::Extract((rentry.end_bit + 1) as u64, 1),
                                    &[widx]);
            solver.assert(core::OpCodes::Cmp, &[idx, source])
        } else {
            let idx = self.current_regs[rentry.idx].unwrap();
            let nidx = solver.assert(core::OpCodes::Cmp, &[idx, source]);
            self.current_regs[rentry.idx] = Some(nidx);
            nidx
        }
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
        let arr_const_ty = qf_abv::array_const(qf_abv::bv_sort(64), qf_abv::bv_sort(64), bitvec::OpCodes::Const(0, 64));
        let const_0 = solver.new_const(arr_const_ty);
        let idx = solver.assert(core::OpCodes::Cmp, &[idx_, const_0]);
        self.map = Some(idx);
    }

    pub fn read(&mut self, addr: u64, read_size: u64, solver: &mut SMTLib2<qf_abv::QF_ABV>) -> NodeIndex {
        if self.map.is_none() {
            self.init_memory(solver);
        }
        let mem = self.map.unwrap();
        let const_addr = bv_const!(solver, addr, 64);
        let mut idx = solver.assert(array_ex::OpCodes::Select, &[mem, const_addr]);
        if read_size < 64 {
            solver.assert(bitvec::OpCodes::Extract(read_size - 1, 1), &[idx])
        } else {
            idx
        }
    }

    pub fn write(&mut self, addr: u64, data: NodeIndex, write_size: u64, solver: &mut SMTLib2<qf_abv::QF_ABV>) {
        if self.map.is_none() {
            self.init_memory(solver);
        }
        let mem = self.map.unwrap();
        let const_addr = bv_const!(solver, addr, 64);
        let new_mem = solver.assert(array_ex::OpCodes::Store, &[mem, const_addr, data]);
        self.map = Some(new_mem);
    }
}
