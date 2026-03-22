//! NEXLANG - The smart contract language for NEXARA.
//!
//! A Rust-like language with built-in security features:
//! - Reentrancy protection
//! - Integer overflow checks
//! - Access control annotations
//! - Gas metering

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod typechecker;
pub mod security;
pub mod codegen;
pub mod errors;

pub use lexer::{Lexer, Token, TokenKind};
pub use parser::Parser;
pub use ast::*;
pub use typechecker::TypeChecker;
pub use security::SecurityAnalyzer;
pub use codegen::CodeGenerator;
pub use errors::NexlangError;
