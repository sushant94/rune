//global _main
//section .text
	//main:
		//; Some instructions, not really relevant for our case.
	
	//write:
		//; Funtion epilogue and prologue skipped. Only function body.
		//; Unrelated asm is skipped for clarity of this example.
		//; Assume buf is assigned to be at rbp - 0xa.
		//; in x86_64, address of buf will be in rdi when this function is
		//; called.
		//lea rax, [rbp - 0xa]
		//add rax, rdi
		//mov rax, rsi
		//ret

extern crate rune;
//extern

use rune::smt::smt::*;
use rune::smt::ssmt::*;
use rune::smt::theories::{bitvec, core};
use rune::smt::theories::integer::OpCodes as IOpCodes;
//use petgraph::graph::NodeIndex;

fn main() {
    let mut solver = SMTLib2::new(Solver::Z3);
    solver.set_logic(Logic::QF_BV);

    // Two symbolic vars corresponding to the user inputs.
    let rdi = solver.new_var(Some("rdi"), Type::BitVector(64));
    let rsi = solver.new_var(Some("rsi"), Type::BitVector(64));

    // Consts rbp and rsp.
    let rsp = solver.new_const(0x8000, Type::BitVector(64));
    let rbp = solver.new_const(0x9000, Type::BitVector(64));
    let const_a = solver.new_const(0xa, Type::BitVector(64));
    let const_4 = solver.new_const(0x4, Type::BitVector(64));
    let const_badcafe = solver.new_const(0xbadcafe, Type::BitVector(64));

    let buf = solver.assert(NodeData::BVOps(bitvec::OpCodes::bvsub), &[rbp, const_a]);
    let rax = solver.assert(NodeData::BVOps(bitvec::OpCodes::bvadd), &[buf, rdi]);
    
    let ret_addr = solver.assert(NodeData::BVOps(bitvec::OpCodes::bvadd), &[rbp, const_4]);

    let selector = solver.assert(IOpCodes::Cmp.into(), &[rax, ret_addr]);
    //solver.assert(NodeData::CoreOps(core::OpCodes::ITE), &[selector, assign_1, assign_2]);

    if let Ok(result) = solver.solve() {
        println!("{:#?}", result);
        println!("Out-Of-Bounds Write detected!");
        println!("rdi: 0x{:x}; rsi: 0x{:x}", result[&rdi], result[&rsi]);
    } else {
        println!("This program is not vulnerable to Out-Of-Bounds Write");
    }
}
