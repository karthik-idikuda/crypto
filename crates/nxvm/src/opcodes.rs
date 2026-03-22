//! NXVM Opcodes - instruction set for the NEXARA VM.

use serde::{Deserialize, Serialize};

/// Opcode for the NXVM instruction set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Opcode {
    // Control
    Nop = 0x00,
    Halt = 0x01,

    // Stack
    Push = 0x10,
    Pop = 0x11,
    Dup = 0x12,
    Swap = 0x13,

    // Arithmetic
    Add = 0x20,
    Sub = 0x21,
    Mul = 0x22,
    Div = 0x23,
    Mod = 0x24,
    Neg = 0x25,

    // Comparison
    Eq = 0x30,
    Ne = 0x31,
    Lt = 0x32,
    Gt = 0x33,
    Le = 0x34,
    Ge = 0x35,

    // Logic
    And = 0x40,
    Or = 0x41,
    Not = 0x42,

    // Flow
    Jump = 0x50,
    JumpIf = 0x51,
    JumpIfNot = 0x52,
    Call = 0x53,
    Return = 0x54,

    // Memory
    Load = 0x60,
    Store = 0x61,
    MemLoad = 0x62,
    MemStore = 0x63,

    // Storage
    SLoad = 0x70,
    SStore = 0x71,

    // Context
    MsgSender = 0x80,
    MsgValue = 0x81,
    BlockHeight = 0x82,
    Timestamp = 0x83,
    ChainId = 0x84,
    GasRemaining = 0x85,

    // Events & reverts
    Emit = 0x90,
    Revert = 0x91,

    // Crypto
    Hash = 0xA0,
    VerifySig = 0xA1,

    // Cross-shard
    CrossShardCall = 0xB0,
    CrossShardReturn = 0xB1,
}

impl Opcode {
    /// Decode a byte into an opcode.
    pub fn from_byte(byte: u8) -> Option<Opcode> {
        match byte {
            0x00 => Some(Opcode::Nop),
            0x01 => Some(Opcode::Halt),
            0x10 => Some(Opcode::Push),
            0x11 => Some(Opcode::Pop),
            0x12 => Some(Opcode::Dup),
            0x13 => Some(Opcode::Swap),
            0x20 => Some(Opcode::Add),
            0x21 => Some(Opcode::Sub),
            0x22 => Some(Opcode::Mul),
            0x23 => Some(Opcode::Div),
            0x24 => Some(Opcode::Mod),
            0x25 => Some(Opcode::Neg),
            0x30 => Some(Opcode::Eq),
            0x31 => Some(Opcode::Ne),
            0x32 => Some(Opcode::Lt),
            0x33 => Some(Opcode::Gt),
            0x34 => Some(Opcode::Le),
            0x35 => Some(Opcode::Ge),
            0x40 => Some(Opcode::And),
            0x41 => Some(Opcode::Or),
            0x42 => Some(Opcode::Not),
            0x50 => Some(Opcode::Jump),
            0x51 => Some(Opcode::JumpIf),
            0x52 => Some(Opcode::JumpIfNot),
            0x53 => Some(Opcode::Call),
            0x54 => Some(Opcode::Return),
            0x60 => Some(Opcode::Load),
            0x61 => Some(Opcode::Store),
            0x62 => Some(Opcode::MemLoad),
            0x63 => Some(Opcode::MemStore),
            0x70 => Some(Opcode::SLoad),
            0x71 => Some(Opcode::SStore),
            0x80 => Some(Opcode::MsgSender),
            0x81 => Some(Opcode::MsgValue),
            0x82 => Some(Opcode::BlockHeight),
            0x83 => Some(Opcode::Timestamp),
            0x84 => Some(Opcode::ChainId),
            0x85 => Some(Opcode::GasRemaining),
            0x90 => Some(Opcode::Emit),
            0x91 => Some(Opcode::Revert),
            0xA0 => Some(Opcode::Hash),
            0xA1 => Some(Opcode::VerifySig),
            0xB0 => Some(Opcode::CrossShardCall),
            0xB1 => Some(Opcode::CrossShardReturn),
            _ => None,
        }
    }

    /// Whether this opcode has an inline operand.
    pub fn has_operand(self) -> bool {
        matches!(
            self,
            Opcode::Push
                | Opcode::Jump
                | Opcode::JumpIf
                | Opcode::JumpIfNot
                | Opcode::Call
                | Opcode::Load
                | Opcode::Store
                | Opcode::MemLoad
                | Opcode::MemStore
                | Opcode::SLoad
                | Opcode::SStore
                | Opcode::Emit
                | Opcode::Revert
                | Opcode::CrossShardCall
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_roundtrip() {
        for byte in 0..=0xFF_u8 {
            if let Some(op) = Opcode::from_byte(byte) {
                assert_eq!(op as u8, byte);
            }
        }
    }

    #[test]
    fn test_has_operand() {
        assert!(Opcode::Push.has_operand());
        assert!(Opcode::Jump.has_operand());
        assert!(!Opcode::Add.has_operand());
        assert!(!Opcode::Halt.has_operand());
    }

    #[test]
    fn test_invalid_opcode() {
        assert!(Opcode::from_byte(0xFE).is_none());
        assert!(Opcode::from_byte(0x99).is_none());
    }
}
