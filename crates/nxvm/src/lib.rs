//! NXVM - NEXARA Virtual Machine for smart contract execution.

pub mod error;
pub mod opcodes;
pub mod stack;
pub mod gas;
pub mod vm;
pub mod state;

pub use error::VmError;
pub use opcodes::Opcode;
pub use stack::Stack;
pub use gas::{GasCounter, GAS_TABLE};
pub use vm::Vm;
pub use state::VmState;
