//! Module that comtains SimpleSMT Backend Implementation.
//!
//! This backend basically translates the opcodes into smtlib2 compatible syntax formula and is
//! stored as string. This is written out to file and provided as input to a SMT solver (like z3)
//! when results are required. This maybe inefficient as we have to perform disk IO multiple times
//! and rely on other processes. Regardless, this provides a very simple interface to work with in
//! absense of bindings for rust.
