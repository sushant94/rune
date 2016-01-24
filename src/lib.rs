//! The Rune Symbolic Emulator Project.
//!
//! Radare2 Symbolic Emulator for all!
//! TODO: Add project notes, descriptions and notes.

// Support for extra lints for clippy
//#![cfg_attr(feature="clippy", feature(plugin))]
//#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate petgraph;
extern crate esil;
extern crate r2pipe;


pub mod bv;
pub mod context;
pub mod explorer;
pub mod engine {
    pub mod engine;
    pub mod rune;
    pub mod hook;
    pub mod breakpt;
}
