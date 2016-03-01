// global _main
// section .text
// main:
// ; Some instructions, not really relevant for our case.

// write:
// ; Funtion epilogue and prologue skipped. Only function body.
// ; Unrelated asm is skipped for clarity of this example.
// ; Assume buf is assigned to be at rbp - 0xa.
// ; in x86_64, address of buf will be in rdi when this function is
// ; called.
// lea rax, [rbp - 0xa]
// add rax, rdi
// mov rax, rsi
// ret

extern crate rune;

use rune::smt::smt::*;
use rune::smt::ssmt::*;
use rune::smt::theories::{array_ex, bitvec, core};
use rune::smt::theories::integer::OpCodes as IOpCodes;
use rune::smt::logics::qf_abv::QF_ABV;

macro_rules! bv_const {
    ($solver: ident, $i: expr, $n: expr) => { $solver.new_const(bitvec::OpCodes::Const($i, $n)) }
}

macro_rules! new_array {
    ($solver: ident, $n: expr, $x: ty, $y: ty) => { $solver.new_var(Some($n), array_ex::Sorts::Array($x.into(), $y.into())) };
    ($solver: ident, $x: ty, $y: ty) => { $solver.new_var(None, array_ex::Sorts::Array($x.into(), $y.into())) };
}


fn main() {
    let mut solver = SMTLib2::new(Solver::Z3, Some(QF_ABV));
    solver.set_logic();

    // Two symbolic vars corresponding to the user inputs.
    let rdi = solver.new_var(Some("rdi"), bitvec::Sorts::BitVector(64));
    let rsi = solver.new_var(Some("rsi"), bitvec::Sorts::BitVector(64));
    let mem = solver.new_var(Some("mem"),
                             array_ex::Sorts::Array(Box::new(bitvec::Sorts::BitVector(64).into()),
                                                    Box::new(bitvec::Sorts::BitVector(64).into())));
    // let mem = new_array!(solver, "mem", bitvec::Sorts::BitVector(64),
    // bitvec::Sorts::BitVector(64));


    // Consts rbp and rsp.
    let _ = bv_const!(solver, 0x8000, 64);
    let rbp = bv_const!(solver, 0x9000, 64);

    let const_a = bv_const!(solver, 0xA, 64);
    let const_4 = bv_const!(solver, 0x4, 64);
    let const_badcafe = bv_const!(solver, 0xcafebabe, 64);
    let const_14 = bv_const!(solver, 0xA, 64);

    let buf = solver.assert(bitvec::OpCodes::Bvsub, &[rbp, const_a]);
    let rax = solver.assert(bitvec::OpCodes::Bvadd, &[buf, rdi]);
    //solver.assert(bitvec::OpCodes::Bvult, &[rdi, const_14]);

    let ret_addr = solver.assert(bitvec::OpCodes::Bvadd, &[rbp, const_4]);

    let new_mem = solver.assert(array_ex::OpCodes::Store, &[mem, rax, rsi]);
    let sel = solver.assert(array_ex::OpCodes::Select, &[new_mem, ret_addr]);
    solver.assert(core::OpCodes::Cmp, &[sel, const_badcafe]);

    if let Ok(result) = solver.solve() {
        println!("{:#?}", result);
        println!("Out-Of-Bounds Write detected!");
        println!("rdi: 0x{:x}; rsi: 0x{:x}", result[&rdi], result[&rsi]);
    } else {
        println!("This program is not vulnerable to Out-Of-Bounds Write");
    }
}
