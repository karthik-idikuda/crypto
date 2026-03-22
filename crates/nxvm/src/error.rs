//! NXVM errors.

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum VmError {
    #[error("Stack overflow: max depth {max}")]
    StackOverflow { max: usize },

    #[error("Stack underflow: needed {needed}, had {had}")]
    StackUnderflow { needed: usize, had: usize },

    #[error("Out of gas: used {used}, limit {limit}")]
    OutOfGas { used: u64, limit: u64 },

    #[error("Invalid opcode: 0x{0:02X}")]
    InvalidOpcode(u8),

    #[error("Invalid jump destination: {0}")]
    InvalidJump(usize),

    #[error("Execution reverted: {0}")]
    Revert(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Invalid memory access: offset {0}")]
    InvalidMemoryAccess(usize),

    #[error("Call depth exceeded: {0}")]
    CallDepthExceeded(usize),

    #[error("Invalid operand at pc={0}")]
    InvalidOperand(usize),
}
