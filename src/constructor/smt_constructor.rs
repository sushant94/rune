use std::marker::PhantomData;

use radeco_lib::middle::ir::{MOpcode};
use radeco_lib::middle::ssa::ssa_traits::{NodeType, SSA, SSAMod, SSAWalk};
use radeco_lib::middle::ssa::ssastorage::{SSAStorage};

use libsmt::logics::qf_abv;
use libsmt::theories::{array_ex, bitvec, core};
use libsmt::backends::z3;
use libsmt::backends::smtlib2::{SMTLib2};
use libsmt::backends::backend::SMTBackend;

pub struct Converter<'a, I, S>
    where I: Iterator<Item = S::ValueRef>,
          S: 'a + SSA + SSAMod + SSAWalk<I>
{
    ssa: &'a S,
    foo: PhantomData<I>,
}

impl<'a, I, S> Converter<'a, I, S>
where I: Iterator<Item=S::ValueRef>,
      S: 'a + SSA + SSAMod + SSAWalk<I>
{
    pub fn new(ssa: &'a S) -> Converter<'a, I, S> {
        Converter {
            ssa: ssa,
            foo: PhantomData
        }
    }

    fn to_smt(&self, op: MOpcode) -> qf_abv::QF_ABV_Fn {
        match op {
            MOpcode::OpAdd => bitvec::OpCodes::BvAdd.into(),
            MOpcode::OpSub => bitvec::OpCodes::BvSub.into(),
            MOpcode::OpMul => bitvec::OpCodes::BvMul.into(),
            MOpcode::OpDiv => bitvec::OpCodes::BvUDiv.into(),
            MOpcode::OpMod => bitvec::OpCodes::BvURem.into(),
            MOpcode::OpAnd => bitvec::OpCodes::BvAnd.into(),
            MOpcode::OpOr => bitvec::OpCodes::BvOr.into(),
            MOpcode::OpXor => bitvec::OpCodes::BvXor.into(),
            MOpcode::OpNot => bitvec::OpCodes::BvNeg.into(),
            MOpcode::OpEq => unimplemented!(),
            MOpcode::OpCmp => core::OpCodes::Cmp.into(),
            MOpcode::OpLt => bitvec::OpCodes::BvULt.into(),
            MOpcode::OpGt => bitvec::OpCodes::BvUGt.into(),
            MOpcode::OpLsl => bitvec::OpCodes::BvShl.into(),
            MOpcode::OpLsr => bitvec::OpCodes::BvLShr.into(),
            MOpcode::OpLoad => array_ex::OpCodes::Select.into(),
            MOpcode::OpStore => array_ex::OpCodes::Store.into(),
            _ => panic!("This opcode is either unimplemented or is not an opcode at all!"),
        }
    }
}

pub fn convert(ssa: SSAStorage) {
	let mut walker = ssa.inorder_walk();
	let mut solver = SMTLib2::new(Some(qf_abv::QF_ABV));
	let mut converter = Converter::new(&ssa);
	let mut z3: z3::Z3 = Default::default();
	for node_index in walker.nodes.iter() {
		let mut operands = ssa.get_operands(node_index);
		let mut opcode = match ssa.get_node_data(node_index) {
			Ok(x) => 
            {
                let mut opcode = match x.nt {
                    NodeType::Op(op) => op,
                    NodeType::Comment(ref s) => MOpcode::OpInvalid,
                    _ => panic!("GG"),
                };
                opcode
            },
			Err(x) => MOpcode::OpInvalid,
		};
		if opcode != MOpcode::OpInvalid {
			solver.assert(converter.to_smt(opcode), &operands.as_ref());
		}
	}
	// let result = solver.solve(&mut z3);
}
