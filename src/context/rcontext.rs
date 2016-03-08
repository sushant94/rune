//! Implementation of context for rune.

use std::collections::HashMap;
use std::fmt::Debug;

use r2pipe::structs::LRegInfo;
use petgraph::graph::NodeIndex;
use libsmt::ssmt::{SMTLib2, SMTSolver};
use libsmt::smt::SMTBackend;
use libsmt::logics::qf_abv::QF_ABV;
use libsmt::theories::{bitvec, core};

pub struct RuneContext {
    solver: SMTLib2<QF_ABV>,
    current_regs: RuneRegFile,
    mem: RuneMemory,
}

pub struct RuneMemory;

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

    pub fn read(&mut self, reg_name: String, solver: &mut SMTLib2<QF_ABV>) -> NodeIndex {
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
                 solver: &mut SMTLib2<QF_ABV>)
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
}
