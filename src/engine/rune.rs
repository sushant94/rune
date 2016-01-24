//! Trait and struct implementations for rune symbolic engine

use context::Context;
use explorer::{InstructionStream, PathExplorer, r2stream};
use engine::engine::{Configure, Engine, EngineError, EngineResult};
use esil::lexer::{Token, Tokenizer};
use esil::parser::{Parse, Parser};
use r2pipe::structs::LOpInfo;

use std::collections::HashMap;

pub struct Rune<C, P>
    where C: Context,
          P: PathExplorer
{
    ctx: C,
    explorer: P,
    temps: HashMap<C::I, C::T>,
}

impl<C, P> Configure for Rune<C, P>
where C: Context,
      P: PathExplorer<I = r2stream>
{
    type For = Rune<C, P>;
    fn configure(engine: &mut Rune<C, P>) -> EngineResult<()> {
        unimplemented!()
    }
}

impl<C, P> Rune<C, P>
where C: Context,
      P: PathExplorer<I = r2stream>
{
    fn process_in(&self, t: Option<Token>) -> EngineResult<C::T> {
        if t.is_none() {
            return Err(EngineError::Undefined);
        }
        match t.unwrap() {
            Token::ERegister(ref name) | Token::EIdentifier(ref name) => {}
            Token::EEntry(ref id) => {}
            Token::EConstant(value) => {}
            Token::EOld => {}
            Token::ECur => {}
            Token::ELastsz => {}
            Token::EAddress => {}
            _ => panic!(""),
        }
        unimplemented!()
    }

    fn process_op(&mut self,
                  token: Token,
                  lhs: Option<C::T>,
                  rhs: Option<C::T>)
                  -> EngineResult<C::T> {
        match token {
            Token::ECmp => {}
            Token::ELt => {}
            Token::EGt => {}
            Token::EEq => {}
            Token::EIf => {}
            Token::EEndIf => {}
            Token::ELsl => {}
            Token::ELsr => {}
            Token::ERor => {}
            Token::ERol => {}
            Token::EAnd => {}
            Token::EOr => {}
            Token::ENeg => {}
            Token::EMul => {}
            Token::EXor => {}
            Token::EAdd => {}
            Token::ESub => {}
            Token::EDiv => {}
            Token::EMod => {}
            Token::EPoke(n) => {}
            Token::EPeek(n) => {}
            Token::EPop => {}
            Token::EGoto => {}
            Token::EBreak => {}
            Token::ENop => {}
            _ => {}
        }

        Err(EngineError::Undefined)
    }

    fn process_out(&mut self, res: &C::T) -> Token {
        unimplemented!()
    }
}

impl<C, P> Engine for Rune<C, P>
where C: Context,
      P: PathExplorer<I = r2stream>
{
    type C = C;
    type P = P;

    fn new<T>() -> Rune<C, P>
        where T: Configure<For = Rune<C, P>>
    {
        let mut rune = Rune {
            ctx: Self::C::new(),
            explorer: Self::P::new(),
            temps: HashMap::new(),
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
            while let Some(ref token) = p.parse::<_, Tokenizer>(esil) {
                let (lhs, rhs) = p.fetch_operands(token);
                let lhs = self.process_in(lhs).ok();
                let rhs = self.process_in(rhs).ok();
                let res = self.process_op(token.clone(), lhs, rhs).ok();

                if res.is_some() {
                    let res = res.unwrap();
                    let rt = self.process_out(&res);
                    p.push(rt);
                }
            }
        }
        Ok(())
    }
}
