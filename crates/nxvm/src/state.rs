//! NXVM State - persistent storage for smart contracts.

use std::collections::HashMap;
use crate::stack::Value;

/// Contract storage state.
#[derive(Debug, Clone, Default)]
pub struct VmState {
    /// Persistent storage slots (key → value).
    storage: HashMap<u32, Value>,
    /// Memory (byte-addressable, expandable).
    memory: Vec<u8>,
    /// Local variable slots.
    locals: HashMap<u32, Value>,
    /// Call context.
    pub caller: Value,
    pub call_value: u64,
    pub block_height: u64,
    pub timestamp: u64,
    pub chain_id: u64,
}

impl VmState {
    /// Create a new empty state.
    pub fn new() -> Self {
        VmState {
            storage: HashMap::new(),
            memory: Vec::new(),
            locals: HashMap::new(),
            caller: [0u8; 32],
            call_value: 0,
            block_height: 0,
            timestamp: 0,
            chain_id: 20240101,
        }
    }

    // Storage

    /// Load a value from storage.
    pub fn sload(&self, slot: u32) -> Value {
        self.storage.get(&slot).copied().unwrap_or([0u8; 32])
    }

    /// Store a value to storage.
    pub fn sstore(&mut self, slot: u32, value: Value) {
        self.storage.insert(slot, value);
    }

    /// Number of storage slots used.
    pub fn storage_size(&self) -> usize {
        self.storage.len()
    }

    // Locals

    /// Load a local variable.
    pub fn load_local(&self, slot: u32) -> Value {
        self.locals.get(&slot).copied().unwrap_or([0u8; 32])
    }

    /// Store a local variable.
    pub fn store_local(&mut self, slot: u32, value: Value) {
        self.locals.insert(slot, value);
    }

    /// Clear locals (on function return).
    pub fn clear_locals(&mut self) {
        self.locals.clear();
    }

    // Memory

    /// Read 32 bytes from memory at offset.
    pub fn mem_load(&self, offset: usize) -> Value {
        let mut val = [0u8; 32];
        let end = (offset + 32).min(self.memory.len());
        if offset < self.memory.len() {
            let len = end - offset;
            val[..len].copy_from_slice(&self.memory[offset..end]);
        }
        val
    }

    /// Write 32 bytes to memory at offset.
    pub fn mem_store(&mut self, offset: usize, value: &Value) {
        let end = offset + 32;
        if end > self.memory.len() {
            self.memory.resize(end, 0);
        }
        self.memory[offset..end].copy_from_slice(value);
    }

    /// Current memory size in bytes.
    pub fn memory_size(&self) -> usize {
        self.memory.len()
    }

    /// Compute storage root (BLAKE3 hash of all storage).
    pub fn storage_root(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        let mut slots: Vec<_> = self.storage.iter().collect();
        slots.sort_by_key(|(&k, _)| k);
        for (slot, value) in slots {
            hasher.update(&slot.to_le_bytes());
            hasher.update(value);
        }
        *hasher.finalize().as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack::{u64_to_value, value_to_u64, ZERO};

    #[test]
    fn test_storage_load_store() {
        let mut state = VmState::new();
        assert_eq!(state.sload(0), ZERO);
        let val = u64_to_value(42);
        state.sstore(0, val);
        assert_eq!(value_to_u64(&state.sload(0)), 42);
    }

    #[test]
    fn test_locals() {
        let mut state = VmState::new();
        let val = u64_to_value(100);
        state.store_local(0, val);
        assert_eq!(value_to_u64(&state.load_local(0)), 100);
        state.clear_locals();
        assert_eq!(state.load_local(0), ZERO);
    }

    #[test]
    fn test_memory() {
        let mut state = VmState::new();
        assert_eq!(state.memory_size(), 0);
        let val = u64_to_value(999);
        state.mem_store(0, &val);
        assert_eq!(state.memory_size(), 32);
        let loaded = state.mem_load(0);
        assert_eq!(value_to_u64(&loaded), 999);
    }

    #[test]
    fn test_storage_root_deterministic() {
        let mut s1 = VmState::new();
        let mut s2 = VmState::new();
        s1.sstore(0, u64_to_value(1));
        s1.sstore(1, u64_to_value(2));
        s2.sstore(1, u64_to_value(2));
        s2.sstore(0, u64_to_value(1));
        assert_eq!(s1.storage_root(), s2.storage_root());
    }

    #[test]
    fn test_default_context() {
        let state = VmState::new();
        assert_eq!(state.chain_id, 20240101);
        assert_eq!(state.block_height, 0);
    }
}
