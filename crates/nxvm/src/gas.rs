//! NXVM Gas metering.

use std::collections::HashMap;
use crate::error::VmError;
use crate::opcodes::Opcode;

/// Gas costs for each opcode.
pub static GAS_TABLE: &[(Opcode, u64)] = &[
    (Opcode::Nop, 1),
    (Opcode::Halt, 0),
    (Opcode::Push, 3),
    (Opcode::Pop, 2),
    (Opcode::Dup, 3),
    (Opcode::Swap, 3),
    (Opcode::Add, 3),
    (Opcode::Sub, 3),
    (Opcode::Mul, 5),
    (Opcode::Div, 5),
    (Opcode::Mod, 5),
    (Opcode::Neg, 3),
    (Opcode::Eq, 3),
    (Opcode::Ne, 3),
    (Opcode::Lt, 3),
    (Opcode::Gt, 3),
    (Opcode::Le, 3),
    (Opcode::Ge, 3),
    (Opcode::And, 3),
    (Opcode::Or, 3),
    (Opcode::Not, 3),
    (Opcode::Jump, 8),
    (Opcode::JumpIf, 10),
    (Opcode::JumpIfNot, 10),
    (Opcode::Call, 40),
    (Opcode::Return, 5),
    (Opcode::Load, 3),
    (Opcode::Store, 3),
    (Opcode::MemLoad, 3),
    (Opcode::MemStore, 3),
    (Opcode::SLoad, 200),
    (Opcode::SStore, 5000),
    (Opcode::MsgSender, 2),
    (Opcode::MsgValue, 2),
    (Opcode::BlockHeight, 2),
    (Opcode::Timestamp, 2),
    (Opcode::ChainId, 2),
    (Opcode::GasRemaining, 2),
    (Opcode::Emit, 375),
    (Opcode::Revert, 0),
    (Opcode::Hash, 30),
    (Opcode::VerifySig, 3000),
    (Opcode::CrossShardCall, 10000),
    (Opcode::CrossShardReturn, 100),
];

/// Gas counter with limit enforcement.
#[derive(Debug, Clone)]
pub struct GasCounter {
    used: u64,
    limit: u64,
    costs: HashMap<u8, u64>,
}

impl GasCounter {
    /// Create a gas counter with the given limit.
    pub fn new(limit: u64) -> Self {
        let mut costs = HashMap::new();
        for &(op, cost) in GAS_TABLE {
            costs.insert(op as u8, cost);
        }
        GasCounter { used: 0, limit, costs }
    }

    /// Consume gas for an opcode. Returns error if over limit.
    pub fn consume(&mut self, opcode: Opcode) -> Result<(), VmError> {
        let cost = self.costs.get(&(opcode as u8)).copied().unwrap_or(1);
        self.charge(cost)
    }

    /// Charge a specific amount of gas.
    pub fn charge(&mut self, amount: u64) -> Result<(), VmError> {
        self.used = self.used.saturating_add(amount);
        if self.used > self.limit {
            Err(VmError::OutOfGas { used: self.used, limit: self.limit })
        } else {
            Ok(())
        }
    }

    /// Gas used so far.
    pub fn gas_used(&self) -> u64 {
        self.used
    }

    /// Gas remaining.
    pub fn gas_remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    /// Gas limit.
    pub fn gas_limit(&self) -> u64 {
        self.limit
    }

    /// Reset gas counter.
    pub fn reset(&mut self) {
        self.used = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_consume() {
        let mut gas = GasCounter::new(100);
        gas.consume(Opcode::Add).unwrap(); // 3
        assert_eq!(gas.gas_used(), 3);
        assert_eq!(gas.gas_remaining(), 97);
    }

    #[test]
    fn test_gas_out_of_gas() {
        let mut gas = GasCounter::new(10);
        gas.consume(Opcode::SStore).unwrap_err(); // 5000 > 10
    }

    #[test]
    fn test_gas_sload_cost() {
        let mut gas = GasCounter::new(1000);
        gas.consume(Opcode::SLoad).unwrap(); // 200
        assert_eq!(gas.gas_used(), 200);
    }

    #[test]
    fn test_gas_reset() {
        let mut gas = GasCounter::new(100);
        gas.consume(Opcode::Push).unwrap();
        gas.reset();
        assert_eq!(gas.gas_used(), 0);
    }

    #[test]
    fn test_cross_shard_expensive() {
        let gas = GasCounter::new(100_000);
        let cost = gas.costs.get(&(Opcode::CrossShardCall as u8)).unwrap();
        assert_eq!(*cost, 10000);
    }
}
