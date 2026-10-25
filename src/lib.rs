//! Arcana: a fantasy-inspired general-purpose programming language.
//!
//! This crate contains the whole implementation (lexer, parser, checker,
//! tree-walking interpreter and CLI).  The `arcana` binary is a thin wrapper
//! around [`cli::main_with_args`].

pub mod ast;
pub mod checker;
pub mod cli;
pub mod environment;
pub mod errors;
pub mod int;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod tokens;
pub mod values;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
