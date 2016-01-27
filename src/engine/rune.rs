//! Trait and struct implementations for rune symbolic engine

use std::collections::HashMap;

use context::{Context, RefType};
use bv::BitVector;
use explorer::{InstructionStream, PathExplorer, R2Stream};
use engine::engine::{Configure, Engine, EngineResult};
use esil::lexer::{Token, Tokenizer};
use esil::parser::{Parse, Parser};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RuneControl {
    Continue,
    Skip,
    Halt,
    Break,
}

pub struct Rune<Ctx, Exp>
    where Ctx: Context,
          Exp: PathExplorer
{
    /// Context on which the instance of rune operates on
    ctx: Ctx,
    /// Path decision algorithm
    explorer: Exp,
    /// Stores values that are intermediates during symbolic execution. These are not
    /// a part of register or memory
    intermediates: Vec<Ctx::BV>,
    /// Instruction pointer
    ip: u64,
}

impl<Ctx, Exp> Configure for Rune<Ctx, Exp>
where Ctx: Context<Idx = RefType<usize, usize>>,
      Exp: PathExplorer<I = R2Stream, C = RuneControl>
{
    type For = Rune<Ctx, Exp>;
    fn configure(_: &mut Rune<Ctx, Exp>) -> EngineResult<()> {
        unimplemented!()
    }
}

impl<Ctx, Exp> Rune<Ctx, Exp>
where Ctx: Context<Idx = RefType<usize, usize>>,
      Exp: PathExplorer<I = R2Stream, C = RuneControl>
{
    fn process_in(&mut self, t: Option<Token>) -> EngineResult<Option<Ctx::BV>> {
        if t.is_none() {
            return Ok(None);
        }
        let read = match t.unwrap() {
            Token::ERegister(ref name) | Token::EIdentifier(ref name) => {
                // TODO: use try! - implement from::From for the error type.
                self.ctx.read(RefType::RegisterIdent(name.clone())).expect("Register Error")
            }
            Token::EEntry(ref id) => self.intermediates[*id].clone(),
            Token::EConstant(value) => self.ctx.new_value(value),
            Token::EAddress => self.ctx.new_value(self.ip),
            _ => panic!("Not an operand"),
        };
        Ok(Some(read))
    }

    fn process_op(&mut self,
                  token: Token,
                  lhs: Option<Ctx::BV>,
                  rhs: Option<Ctx::BV>,
                  control: &mut RuneControl)
                  -> EngineResult<Option<Ctx::BV>> {

        // asserts to check validity.
        if token.is_arity_zero() {
            return Ok(None);
        }

        let lhs = lhs.unwrap();
        // Instructions that do not produce a result
        // Example: Mem Write / Eq / If / EndIf
        match token {
            Token::EEq => {
                self.ctx.update_bv(&lhs, rhs.unwrap()).expect("Could not set value");
                return Ok(None);
            }
            Token::EIf => {
                *control = self.explorer.register_branch(lhs, &mut self.ctx);
                return Ok(None);
            }
            // TODO: Adjust width
            Token::EPoke(_) => {
                // Check if the access is a symbolic access.
                // TODO: We do not support symbolic accesses just yet.
                if lhs.is_symbolic() {
                    panic!("Rune has detected a symbolic memory access. \
                            This feature is not implemented yet.");
                }
                self.ctx
                    .write_mem(RefType::MemAddr(lhs.into()), rhs.as_ref().unwrap())
                    .expect("Mem Write Error");
                return Ok(None);
            }
            Token::ENop => return Ok(None),
            _ => { }
        }

        let result = match token {
            Token::ECmp => lhs.eq(rhs.as_ref().unwrap()),
            Token::ELt => lhs.lt(rhs.as_ref().unwrap()),
            Token::EGt => lhs.gt(rhs.as_ref().unwrap()),
            Token::EEndIf => unimplemented!(),
            Token::ELsl => lhs << rhs.unwrap(),
            Token::ELsr => lhs >> rhs.unwrap(),
            Token::ERor => unimplemented!(),
            Token::ERol => unimplemented!(),
            Token::EAnd => lhs & rhs.unwrap(),
            Token::EOr => lhs | rhs.unwrap(),
            Token::ENeg => !lhs,
            Token::EMul => lhs * rhs.unwrap(),
            Token::EXor => lhs ^ rhs.unwrap(),
            Token::EAdd => lhs + rhs.unwrap(),
            Token::ESub => lhs - rhs.unwrap(),
            Token::EDiv => lhs / rhs.unwrap(),
            Token::EMod => lhs % rhs.unwrap(),
            // TODO: Adjust width.
            Token::EPeek(_) => {
                // Check if the access is a symbolic access.
                // TODO: We do not support symbolic accesses just yet.
                if lhs.is_symbolic() {
                    panic!("Rune has detected a symbolic memory access. \
                            This feature is not implemented yet.");
                }
                // TODO: Error conversion trait and use try!
                self.ctx.read_mem(RefType::MemAddr(lhs.into())).expect("Mem Read Error")
            }
            Token::EPop => unimplemented!(),
            Token::EGoto => unimplemented!(),
            Token::EBreak => unimplemented!(),
            _ => unreachable!(),
        };

        Ok(Some(result))
    }

    // Write out to intermediates and return a token to it.
    fn process_out(&mut self, res: &Ctx::BV) -> Token {
        self.intermediates.push(res.clone());
        Token::EEntry(self.intermediates.len() - 1)
    }
}

impl<Ctx, Exp> Engine for Rune<Ctx, Exp>
where Ctx: Context<Idx = RefType<usize, usize>>,
      Exp: PathExplorer<I = R2Stream, C = RuneControl>
{
    type Ctx = Ctx;
    type Exp = Exp;

    fn new<T>() -> Rune<Ctx, Exp>
        where T: Configure<For = Rune<Ctx, Exp>>
    {
        let mut rune = Rune {
            ctx: Self::Ctx::new(),
            explorer: Self::Exp::new(),
            intermediates: Vec::new(),
            ip: 0,
        };

        {
            T::configure(&mut rune).expect("Config Error");
        }
        rune
    }

    fn run(&mut self) -> EngineResult<()> {
        let mut p = Parser::init(None);
        let mut control;
        while let Some(ref opinfo) = self.explorer.next(&mut self.ctx) {
            let esil = opinfo.esil.as_ref().unwrap();
            // Set the instruction pointer to the correct location.
            self.ip = opinfo.offset.unwrap();
            // Reset control
            control = RuneControl::Continue;
            while let Some(ref token) = p.parse::<_, Tokenizer>(esil) {
                let (lhs, rhs) = p.fetch_operands(token);
                let lhs = try!(self.process_in(lhs));
                let rhs = try!(self.process_in(rhs));
                if let Ok(Some(ref res)) = self.process_op(token.clone(), lhs, rhs, &mut control) {
                    let rt = self.process_out(res);
                    p.push(rt);
                }

                // Decide action based on control.
                match control {
                    RuneControl::Continue => {},
                    RuneControl::Skip => break,
                    _ => unimplemented!(),
                }
            }
        }
        Ok(())
    }
}
