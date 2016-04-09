//! Utilities and other miscellaneous functions for RuneContext.

use context::rune_ctx::{RuneContext, RuneMemory, RuneRegFile};
use context::context::{ContextAPI};
use libsmt::backends::smtlib2::SMTLib2;
use libsmt::logics::qf_abv;

use std::collections::HashMap;

#[derive(Debug, Clone)]
enum Key {
    Mem(usize),
    Reg(String),
}

fn to_key<T: AsRef<str>>(s: T) -> Key {
    let v = s.as_ref();
    if v.len() > 2 && &v[0..2] == "0x" {
        Key::Mem(usize::from_str_radix(&v[2..], 16).expect("Invalid number!"))
    } else if v.chars().nth(0).unwrap().is_digit(10) {
        Key::Mem(usize::from_str_radix(&v, 10).expect("Invalid number!"))
    } else {
        Key::Reg(v.to_owned())
    }
}

pub fn new_ctx(ip: Option<u64>,
               syms: Option<Vec<String>>,
               consts: Option<HashMap<String, u64>>)
               -> RuneContext {
    let rregfile = {
        use r2pipe::r2::R2;
        let mut r2 = R2::new(Some("malloc://64".to_owned())).expect("Unable to spawn r2!");
        r2.send("e asm.bits = 64");
        r2.send("e asm.arch = x86");
        r2.flush();
        let mut lreginfo = r2.get_reg_info().expect("Unable to retrieve register information!");
        r2.close();
        RuneRegFile::new(&mut lreginfo)
    };

    let mut rmem = RuneMemory::new();
    let mut smt = SMTLib2::new(Some(qf_abv::QF_ABV));
    rmem.init_memory(&mut smt);
    let mut ctx = RuneContext::new(ip, rmem, rregfile, smt);

    if let Some(ref sym_vars) = syms {
        for var in sym_vars {
            match to_key(var) {
                Key::Mem(addr) => ctx.set_mem_as_sym(addr, 64),
                Key::Reg(ref reg) => ctx.set_reg_as_sym(reg),
            }
        }
    }

    if let Some(ref const_var) = consts {
        for (k, v) in const_var {
            match to_key(k) {
                Key::Mem(addr) => ctx.set_mem_as_const(addr, *v, 64),
                Key::Reg(ref reg) => ctx.set_reg_as_const(reg, *v),
            }
        }
    }

    ctx
}
