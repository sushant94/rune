//! The Rune Symbolic Emulator Project.
//!
//! Radare2 Symbolic Emulator for all!
//! TODO: Add project notes, descriptions and notes.

// Support for extra lints for clippy
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate petgraph;
extern crate esil;
extern crate r2pipe;
extern crate rustc_serialize;
extern crate regex;
#[macro_use] extern crate libsmt;
#[macro_use] extern crate serde_derive;

pub mod context {
    pub mod context;
    pub mod rune_ctx;
    pub mod utils;
}

pub mod explorer {
    pub mod explorer;
    pub mod dfs;
    pub mod bfs;
    pub mod command;
}

pub mod engine {
    pub mod engine;
    pub mod rune;
    pub mod hook;
    pub mod breakpt;
}

pub mod stream;
