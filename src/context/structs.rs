//! Defines supplementary structs needed for defining `Context`s.

use std::collections::HashMap;

use r2pipe::structs::LRegInfo;
use petgraph::graph::NodeIndex;
use libsmt::backends::smtlib2::{SMTLib2};
use libsmt::backends::backend::SMTBackend;
use libsmt::logics::qf_abv;
use libsmt::theories::{array_ex, bitvec, core};

#[derive(Clone, Debug, Default)]
pub struct RuneMemory {
    map: Option<NodeIndex>,
}

#[derive(Clone, Debug, Default)]
pub struct RegEntry {
    pub name: String,
    pub idx: usize,
    // 0 indexed
    pub start_bit: usize,
    pub end_bit: usize,
    pub is_whole: bool,
    pub alias: Option<String>,
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

    pub fn get_width(&self) -> usize {
        self.end_bit - self.start_bit + 1
    }
}

#[derive(Clone, Debug, Default)]
pub struct RuneRegFile {
    pub current_regs: Vec<Option<NodeIndex>>,
    pub regfile: HashMap<String, RegEntry>,
    pub alias_info: HashMap<String, String>,
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

    pub fn read(&mut self, reg_name: &str, solver: &mut SMTLib2<qf_abv::QF_ABV>) -> NodeIndex {
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
    pub fn write(&mut self, dest: &str, source: NodeIndex) -> Option<NodeIndex> {
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
                _: u64,
                solver: &mut SMTLib2<qf_abv::QF_ABV>)
                -> NodeIndex {
        if self.map.is_none() {
            self.init_memory(solver);
        }
        let mem = self.map.unwrap();
        let idx = solver.assert(array_ex::OpCodes::Select, &[mem, addr]);
        // Since this feature is unimplemented, patching it for temporary
        // purposes. This HACK has to fixed later.
        idx
        // if read_size < 64 {
        //     solver.assert(bitvec::OpCodes::Extract(read_size - 1, 1), &[idx])
        // } else {
        //     idx
        // }
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
