//! Define break;

use std::fs::File;
use std::io::prelude::*;

use r2api::structs::LRegInfo;
use petgraph::graph::NodeIndex;
use serde_json::{to_string, from_reader};
use std::collections::HashMap;

use libsmt::backends::smtlib2::{SMTLib2, SMTProc};
use libsmt::backends::backend::SMTBackend;
use libsmt::logics::qf_abv;
use libsmt::theories::{array_ex, bitvec, core};

use memory::memory::Memory;
use memory::qword_mem::QWordMemory;

use regstore::regstore::{RegStore, RegEntry, RegStoreAPI};
use regstore::regfile::RuneRegFile;

use utils::utils::{Key, new_ctx};
use context::context::{Context, ContextAPI, Evaluate, RegisterRead, RegisterWrite, MemoryRead, MemoryWrite};

#[derive(Clone, Debug)]
pub struct RuneContext<Mem, Reg> 
    where Mem: Memory,
          Reg: RegStore
{
    ip: u64,
    pub solver: SMTLib2<qf_abv::QF_ABV>,
    regstore: Reg,
    mem: Mem,
    e_old: Option<NodeIndex>,
    e_cur: Option<NodeIndex>,
    // FIXME: Move syms out. 
    pub syms: HashMap<String, NodeIndex>,
}

impl<Mem, Reg> Context for RuneContext<Mem, Reg>
    where Mem: Memory<VarRef=NodeIndex>,
          Reg: RegStore<VarRef=NodeIndex>
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
        self.regstore.get_reg_entry(&reg).alias.clone()
    }

    fn solve<S: SMTProc>(&mut self, p: &mut S) -> HashMap<NodeIndex, u64> {
        self.solver.solve(p).expect("No satisfying solution.")
    }

    fn var_named<T: AsRef<str>>(&self, var: T) -> Option<NodeIndex> {
        self.syms.get(var.as_ref()).cloned()
    }
}

impl<Mem, Reg> RegisterRead for RuneContext<Mem, Reg>
where Mem: Memory<VarRef=NodeIndex>,
      Reg: RegStore<VarRef=NodeIndex>
{
    type VarRef = NodeIndex;

    fn reg_read<T: AsRef<str>>(&mut self, reg: T) -> NodeIndex {
        self.regstore.read(reg.as_ref(), &mut self.solver)
    }
}

impl<Mem, Reg> RegisterWrite for RuneContext<Mem, Reg> 
where Mem: Memory<VarRef=NodeIndex>,
      Reg: RegStore<VarRef=NodeIndex>
{
    type VarRef = NodeIndex;

    fn reg_write<T: AsRef<str>>(&mut self, reg: T, source: NodeIndex) {
        let e_old = self.regstore.write(reg.as_ref(), source);
        // XXX: THIS IS A HACK!
        // IF NOT REG
        if !reg.as_ref().to_owned().ends_with('f') {
            self.e_old = e_old;
            self.e_cur = Some(source);
        }
    }
}

impl<Mem, Reg> MemoryRead for RuneContext<Mem, Reg>
where Mem: Memory<VarRef=NodeIndex>,
      Reg: RegStore<VarRef=NodeIndex>
{
    type VarRef = NodeIndex;
    
    fn mem_read(&mut self, addr: NodeIndex, read_size: usize) -> NodeIndex {
        self.mem.read(addr, read_size, &mut self.solver)
    }
}

impl<Mem, Reg> MemoryWrite for RuneContext<Mem, Reg>
where Mem: Memory<VarRef=NodeIndex>,
      Reg: RegStore<VarRef=NodeIndex>
{
    type VarRef = NodeIndex;

    fn mem_write(&mut self, addr: NodeIndex, data: NodeIndex, write_size: usize) {
        self.mem.write(addr, data, write_size, &mut self.solver);
    }
}

impl<Mem, Reg> Evaluate for RuneContext<Mem, Reg>
where Mem: Memory<VarRef=NodeIndex>,
      Reg: RegStore<VarRef=NodeIndex>
{
    type VarRef = NodeIndex;
    type IFn = qf_abv::QF_ABV_Fn;

    fn eval<T, Q>(&mut self, smt_fn: T, operands: Q) -> Self::VarRef
        where T: Into<Self::IFn>,
              Q: AsRef<[Self::VarRef]>
    {
        // TODO: Add extract / concat to ensure that the registers are of compatible
        // sizes for operations.
        self.solver.assert(smt_fn, operands.as_ref())
    }
}

impl<Mem, Reg> ContextAPI for RuneContext<Mem, Reg>
where Mem: Memory<VarRef=NodeIndex>,
      Reg: RegStore<VarRef=NodeIndex> + RegStoreAPI
{
    fn set_reg_as_const<T: AsRef<str>>(&mut self, reg: T, val: u64) -> NodeIndex {
        // Assert that the register is not currently set/defined.
        if let Some(cval) = self.regstore.get_reg_ref(reg.as_ref()) {
            cval
        } else {
            let cval = self.define_const(val, 64);
            self.regstore.set_reg(reg.as_ref(), cval);
            cval
        } 
    }

    fn set_reg_as_sym<T: AsRef<str>>(&mut self, reg: T) -> NodeIndex {
        // Assert that the register is not currently set/defined.
        assert!(self.regstore.get_reg_ref(reg.as_ref()).is_none());

        let sym = self.solver.new_var(Some(reg.as_ref()), qf_abv::bv_sort(64));
        self.regstore.set_reg(reg.as_ref(), sym);
        self.syms.insert(reg.as_ref().to_owned(), sym);
        sym
    }

    fn set_mem_as_const(&mut self, addr: u64, val: u64, write_size: usize) -> NodeIndex {
        let cval = self.define_const(val, write_size);
        let addr = self.define_const(addr, 64);
        
        // (MAJOR) TODO
        if write_size < 64 {
            unimplemented!();
        } else {
            self.mem_write(addr, cval, 64);
        }
        cval
    }

    fn set_mem_as_sym(&mut self, addr: u64, write_size: usize) -> NodeIndex {
        assert_eq!(write_size, 64, "TODO: Unimplemented set_mem for size < 64!");

        let key = format!("mem_{}", addr);
        let sym = self.solver.new_var(Some(&key), qf_abv::bv_sort(64));
        let addr = self.define_const(addr as u64, 64);
        self.mem_write(addr, sym, write_size);
        self.syms.insert(key, sym);
        sym
    }

    fn zero_registers(&mut self) {
        let cval = Some(self.define_const(0, 64));
        for reg in &mut self.regstore.get_regs() {
            if reg.is_none() {
                *reg = cval;
            }
        }
    }

    fn registers(&self) -> Vec<String> {
        unimplemented!();
    }
}

impl<Mem, Reg> RuneContext<Mem, Reg>
where Mem: Memory,
      Reg: RegStore
{
    pub fn new(ip: Option<u64>,
               mem: Mem,
               regstore: Reg,
               solver: SMTLib2<qf_abv::QF_ABV>)
               -> RuneContext<Mem, Reg> {
        RuneContext {
            ip: ip.unwrap_or(0),
            mem: mem,
            regstore: regstore,
            solver: solver,
            e_old: None,
            e_cur: None,
            syms: HashMap::new(),
        }
    }
}


/*
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

    #[test]
    fn ctx_reg_write() {
        let mut lreginfo = Default::default();
        let mut ctx = utils::new_ctx(None, &None, &None, &mut lreginfo);
        let const_8 = ctx.define_const(8, 64);

        // Test setting rax to 8
        ctx.reg_write("rax", const_8);
        assert_eq!(ctx.reg_read("rax"), const_8);
    }

    fn solver() -> z3::Z3 {
        Default::default()
    }

    #[test]
    fn ctx_reg_read() {
        let mut lreginfo = Default::default();
        let mut ctx = utils::new_ctx(None, &None, &None, &mut lreginfo);

        ctx.set_reg_as_sym("rax");

        // We set rax to be some value, and add constraints on sub-registers of rax. if
        // all is
        // well, these constraints will be consistent and check-sat will return a sat.
        let deadbeef = 0xdeadbeefdeadbeef;
        let const_deadbeef = ctx.define_const(deadbeef, 64);
        let const_deadbeef_32 = ctx.define_const(deadbeef & 0xffffffff, 32);
        let const_deadbeef_16 = ctx.define_const(deadbeef & 0xffff, 16);
        let const_deadbeef_8 = ctx.define_const(deadbeef & 0xff, 8);

        let rax = ctx.reg_read("rax");
        let eax = ctx.reg_read("eax");
        let ax = ctx.reg_read("ax");
        let al = ctx.reg_read("al");

        ctx.eval(core::OpCodes::Cmp, &[rax, const_deadbeef]);
        ctx.eval(core::OpCodes::Cmp, &[eax, const_deadbeef_32]);
        ctx.eval(core::OpCodes::Cmp, &[ax, const_deadbeef_16]);
        ctx.eval(core::OpCodes::Cmp, &[al, const_deadbeef_8]);

        let mut z3: z3::Z3 = Default::default();
        assert!(ctx.solver.check_sat(&mut z3));
    }

    #[test]
    fn ctx_reg_solve_simple() {
        let mut lreginfo = Default::default();
        let mut ctx = utils::new_ctx(None, &None, &None, &mut lreginfo);
        // Set rdi and rsi as symbolic
        ctx.set_reg_as_sym("rdi");
        ctx.set_reg_as_sym("rsi");

        let const_deadbeef = ctx.define_const(0x0000dead0000beef, 64);
        let const_0_32 = ctx.define_const(0, 32);
        let const_0_64 = ctx.define_const(0, 64);

        // Assert that rdi ^ rsi = 0x0000dead0000beef.
        //             edi = 0
        let rdi = ctx.reg_read("rdi");
        let edi = ctx.reg_read("edi");
        let rsi = ctx.reg_read("rsi");
        let rdi_rsi = ctx.eval(bitvec::OpCodes::BvXor, &[rdi, rsi]);

        ctx.eval(bitvec::OpCodes::BvUGt, &[rdi, const_0_64]);
        ctx.eval(bitvec::OpCodes::BvULt, &[rdi, const_deadbeef]);
        ctx.eval(core::OpCodes::Cmp, &[rdi_rsi, const_deadbeef]);
        ctx.eval(core::OpCodes::Cmp, &[edi, const_0_32]);

        let result = {
            let mut z3: z3::Z3 = Default::default();
            ctx.solve(&mut z3)
        };

        assert_eq!(result[&rdi], 0xdead00000000);
        assert_eq!(result[&rsi], 0xbeef);
    }

    #[test]
    fn ctx_mem_read_write() {
        let mut lreginfo = Default::default();
        let mut ctx = utils::new_ctx(None, &None, &None, &mut lreginfo);

        ctx.set_reg_as_sym("rax");

        let rax = ctx.reg_read("rax");
        let addr = ctx.define_const(0xbadcafe, 64);
        let const_deadbeef = ctx.define_const(0xdeadbeef, 64);

        ctx.mem_write(addr, rax, 64);

        let rbx = ctx.mem_read(addr, 64);
        ctx.eval(core::OpCodes::Cmp, &[rbx, const_deadbeef]);

        let result = ctx.solve(&mut solver());
        assert_eq!(result[&rax], 0xdeadbeef);
    }

    #[test]
    fn ctx_mem_sym_read_write() {
        let mut lreginfo = Default::default();
        let mut ctx = utils::new_ctx(None, &None, &None, &mut lreginfo);

        ctx.set_mem_as_sym(0xff41, 64);
        ctx.set_mem_as_sym(0xfe41, 64);

        let addr = ctx.define_const(0xff41, 64);
        let addr_ = ctx.define_const(0xfe41, 64);

        let rax = ctx.mem_read(addr, 64);
        ctx.reg_write("rax", rax);
        let rbx = ctx.mem_read(addr_, 64);
        ctx.reg_write("rbx", rbx);

        let eax = ctx.reg_read("eax");
        let ebx = ctx.reg_read("ebx");
        let const_deadbeef = ctx.define_const(0xdeadbeef, 32);
        let const_badcafe = ctx.define_const(0xbadcafe, 32);

        ctx.eval(core::OpCodes::Cmp, &[eax, const_deadbeef]);
        ctx.eval(core::OpCodes::Cmp, &[ebx, const_badcafe]);

        ctx.solve(&mut solver());
        // TODO: Test does not assert correctness yet.
    }

    #[test]
    fn ctx_test_ip() {
        let mut lreginfo = Default::default();
        let mut ctx = utils::new_ctx(None, &None, &None, &mut lreginfo);

        ctx.set_ip(0xbadcafe);
        assert_eq!(ctx.ip(), 0xbadcafe);

        ctx.increment_ip(4);
        assert_eq!(ctx.ip(), 0xbadcafe + 4);
    }

    #[test]
    fn ctx_test_alias() {
        let mut lreginfo = Default::default();
        let mut ctx = utils::new_ctx(None, &None, &None, &mut lreginfo);
        assert_eq!(ctx.alias_of("rip".to_owned()), Some("PC".to_owned()));
    }

    #[test]
    #[should_panic]
    fn ctx_unset_access_esil_old() {
        let mut lreginfo = Default::default();
        let mut ctx = utils::new_ctx(None, &None, &None, &mut lreginfo);
        ctx.e_old();
    }

    #[test]
    #[should_panic]
    fn ctx_unset_access_esil_cur() {
        let mut lreginfo = Default::default();
        let mut ctx = utils::new_ctx(None, &None, &None, &mut lreginfo);
        ctx.e_cur();
    }

    #[test]
    fn ctx_access_esil_old_cur() {
        let mut lreginfo = Default::default();
        let mut ctx = utils::new_ctx(None, &None, &None, &mut lreginfo);
        let const_8 = ctx.define_const(8, 64);
        let const_32 = ctx.define_const(32, 64);

        // Set rax to 8.
        ctx.reg_write("rax", const_8);
        ctx.reg_write("rbx", const_32);

        // Get handles to rax and rbx
        let rax = ctx.reg_read("rax");
        let rbx = ctx.reg_read("rbx");

        // rax = rax + rbx
        let rax_rbx = ctx.eval(bitvec::OpCodes::BvAdd, &[rax, rbx]);
        ctx.reg_write("rax", rax_rbx);

        // Check
        assert_eq!(ctx.e_old(), const_8);
        assert_eq!(ctx.e_cur(), rax_rbx);
    }

    #[test]
    #[should_panic]
    fn ctx_read_before_set() {
        let mut lreginfo = Default::default();
        let mut ctx = utils::new_ctx(None, &None, &None, &mut lreginfo);
        ctx.reg_read("zf");
    }

    #[test]
    #[should_panic]
    fn ctx_invalid_reg() {
        let mut lreginfo = Default::default();
        let mut ctx = utils::new_ctx(None, &None, &None, &mut lreginfo);
        ctx.reg_read("asassa");
    }
}
*/
