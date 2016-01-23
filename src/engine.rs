//! Defines traits and structs that perform the actual sumbolic emulation.

use context::Context;
use explorer::PathExplorer;

pub enum EngineError {
}

pub trait Engine: Configure
{
    type C: Context;
    type P: PathExplorer;

    fn new<S: AsRef<str>>(Option<S>) -> Self;
    fn run(&mut self) -> Result<(), EngineError>;
}

pub trait Configure {
    fn json_configure<S: AsRef<str>>(&mut self, S);
}

pub struct Rune<C, P>
    where C: Context,
          P: PathExplorer
{
    ctx: C,
    explorer: P,
}

impl<C, P> Configure for Rune<C, P>
where C: Context,
      P: PathExplorer
{
    fn json_configure<S: AsRef<str>>(&mut self, config: S) {
        unimplemented!()
    }
}

impl<C, P> Engine for Rune<C, P>
where C: Context,
      P: PathExplorer
{
    type C = C;
    type P = P;

    fn new<S: AsRef<str>>(config: Option<S>) -> Rune<C, P> {
        let mut rune = Rune {
            ctx: Self::C::new(),
            explorer: Self::P::new(),
        };

        config.map(|c| rune.json_configure(c));
        rune
    }

    fn run(&mut self) -> Result<(), EngineError> {
        unimplemented!()
    }
}
