//! Trait and struct implementations for rune symbolic engine

use std::collections::HashMap;

use context::{Context, RefType};
use bv::BitVector;
use explorer::{InstructionStream, PathExplorer, R2Stream};
use engine::engine::{Configure, Engine, EngineResult};
use esil::lexer::{Token, Tokenizer};
use esil::parser::{Parse, Parser};

/// `ctx` - Context on which the instance of rune operates on
/// `explorer` - Path decision algortihm
/// `intermediates` - Stores values that are intermediates during symbolic execution. These are not
/// a part of register or memory
/// `ip` - Instruction pointer
pub struct Rune<Ctx, Exp>
    where Ctx: Context,
          Exp: PathExplorer
{
    ctx: Ctx,
    explorer: Exp,
    intermediates: Vec<Ctx::BV>,
    ip: u64,
}

impl<Ctx, Exp> Configure for Rune<Ctx, Exp>
where Ctx: Context<Idx = RefType<usize, usize>>,
      Exp: PathExplorer<I = R2Stream>
{
    type For = Rune<Ctx, Exp>;
    fn configure(_: &mut Rune<Ctx, Exp>) -> EngineResult<()> {
        unimplemented!()
    }
}

impl<Ctx, Exp> Rune<Ctx, Exp>
where Ctx: Context<Idx = RefType<usize, usize>>,
      Exp: PathExplorer<I = R2Stream>
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
            _ => panic!(""),
        };
        Ok(Some(read))
    }

    fn process_op(&mut self,
                  token: Token,
                  lhs: Option<Ctx::BV>,
                  rhs: Option<Ctx::BV>)
                  -> EngineResult<Option<Ctx::BV>> {

        // asserts to check validity.
        if token.is_arity_zero() {
            return Ok(None);
        }

        let lhs = lhs.unwrap();
        let result = match token {
            Token::ECmp => lhs.eq(rhs.as_ref().unwrap()),
            Token::ELt => lhs.lt(rhs.as_ref().unwrap()),
            Token::EGt => lhs.gt(rhs.as_ref().unwrap()),
            Token::EEq => unimplemented!(),
            Token::EIf => unimplemented!(),
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
            Token::EPoke(_) => unimplemented!(),
            Token::EPeek(_) => unimplemented!(),
            Token::EPop => unimplemented!(),
            Token::EGoto => unimplemented!(),
            Token::EBreak => unimplemented!(),
            Token::ENop => unimplemented!(),
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
      Exp: PathExplorer<I = R2Stream>
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
        while let Some(ref opinfo) = self.explorer.next(&mut self.ctx) {
            let esil = opinfo.esil.as_ref().unwrap();
            // Set the instruction pointer to the correct location.
            self.ip = opinfo.offset.unwrap();
            while let Some(ref token) = p.parse::<_, Tokenizer>(esil) {
                let (lhs, rhs) = p.fetch_operands(token);
                let lhs = try!(self.process_in(lhs));
                let rhs = try!(self.process_in(rhs));
                if let Ok(Some(ref res)) = self.process_op(token.clone(), lhs, rhs) {
                    let rt = self.process_out(res);
                    p.push(rt);
                }
            }
        }
        Ok(())
    }
}
