//! Utilities and other miscellaneous functions for `RuneContext`

use context::rune_ctx::{RuneContext, RuneMemory, RuneRegFile};
use context::context::{ContextAPI};

use libsmt::backends::smtlib2::SMTLib2;
use libsmt::logics::qf_abv;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValType {
    Concrete(usize),
    Symbolic,
    Break,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Key {
    Mem(usize),
    Reg(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SAssignment {
    pub lvalue: Key,
    pub rvalue: ValType,
}

/// Hex/Decimal to Memory address, any other string maps to Registers
///
/// Useful when input strings is to be interpretted either as a Memory Address or a register name.
pub fn to_key<T: AsRef<str>>(s: T) -> Key {
    let v = s.as_ref();
    if v.len() > 2 && &v[0..2] == "0x" {
        Key::Mem(usize::from_str_radix(&v[2..], 16).expect("Invalid number!"))
    } else if v.chars().nth(0).unwrap().is_digit(10) {
        Key::Mem(usize::from_str_radix(v, 10).expect("Invalid number!"))
    } else {
        Key::Reg(v.to_owned())
    }
}

pub fn to_valtype<T: AsRef<str>>(s: T) -> Option<ValType> {
    let v = s.as_ref();

    if v == "SYM" {
        Some(ValType::Symbolic)
    } else if let Some(val) = convert_to_u64(v) {
        Some(ValType::Concrete(val as usize))
    } else {
        None
    }
}

pub fn to_assignment<T: AsRef<str>>(s: T) -> Option<SAssignment> {
    let v = s.as_ref();
    let ops: Vec<&str> = v.split('=').collect();

    let lvalue: Key = to_key(ops[0].trim());
    if let Some(rvalue) = to_valtype(ops[1].trim()) {
        Some(SAssignment {
                lvalue: lvalue,
                rvalue: rvalue,
            })
    } else {
        None
    }
}

pub fn convert_to_u64<T: AsRef<str>>(s: T) -> Option<u64> {
    let v = s.as_ref();
    if v.len() > 2 && &v[0..2] == "0x" {
        if let Ok(val) = usize::from_str_radix(&v[2..], 16) {
            Some(val as u64)
        } else {
            None
        }
    } else if v.chars().nth(0).unwrap().is_digit(10) {
        if let Ok(val) = usize::from_str_radix(v, 10) {
            Some(val as u64)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn new_ctx(ip: Option<u64>,
               syms: &Option<Vec<Key>>,
               consts: &Option<Vec<(Key, u64)>>)
               -> RuneContext {
    let rregfile = {
        use r2pipe::r2::R2;
        let mut r2 = R2::new(Some("malloc://64".to_owned())).expect("Unable to spawn r2!");
        // TODO: Fix it based on the binary being used.
        r2.send("e asm.bits = 64");
        r2.send("e asm.arch = x86");
        r2.flush();
        let mut lreginfo = r2.reg_info().expect("Unable to retrieve register information!");
        r2.close();
        RuneRegFile::new(&mut lreginfo)
    };

    let mut rmem = RuneMemory::new();
    let mut smt = SMTLib2::new(Some(qf_abv::QF_ABV));
    rmem.init_memory(&mut smt);
    let mut ctx = RuneContext::new(ip, rmem, rregfile, smt);

    if let Some(ref sym_vars) = *syms {
        for var in sym_vars {
            let  _ = match *var {
                Key::Mem(addr) => ctx.set_mem_as_sym(addr, 64),
                Key::Reg(ref reg) => ctx.set_reg_as_sym(reg),
            };
        }
    }

    if let Some(ref const_var) = *consts {
        for &(ref k, v) in const_var.iter() {
            let _ = match *k {
                Key::Mem(addr) => ctx.set_mem_as_const(addr, v, 64),
                Key::Reg(ref reg) => ctx.set_reg_as_const(reg, v),
            };
        }
    }
    ctx
}
