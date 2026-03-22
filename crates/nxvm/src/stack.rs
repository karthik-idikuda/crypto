//! NXVM Stack - bounded value stack for the VM.

use crate::error::VmError;

/// Maximum stack depth.
pub const MAX_STACK_DEPTH: usize = 1024;

/// Stack value — all values are 256-bit (stored as [u8; 32]).
pub type Value = [u8; 32];

/// Zero value constant.
pub const ZERO: Value = [0u8; 32];

/// VM operand stack.
#[derive(Debug, Clone)]
pub struct Stack {
    data: Vec<Value>,
    max_depth: usize,
}

impl Stack {
    /// Create a new stack with default max depth.
    pub fn new() -> Self {
        Stack {
            data: Vec::with_capacity(256),
            max_depth: MAX_STACK_DEPTH,
        }
    }

    /// Create a stack with custom max depth.
    pub fn with_max_depth(max: usize) -> Self {
        Stack {
            data: Vec::with_capacity(max.min(256)),
            max_depth: max,
        }
    }

    /// Push a value onto the stack.
    pub fn push(&mut self, value: Value) -> Result<(), VmError> {
        if self.data.len() >= self.max_depth {
            return Err(VmError::StackOverflow { max: self.max_depth });
        }
        self.data.push(value);
        Ok(())
    }

    /// Pop a value from the stack.
    pub fn pop(&mut self) -> Result<Value, VmError> {
        self.data.pop().ok_or(VmError::StackUnderflow { needed: 1, had: 0 })
    }

    /// Peek at the top value without popping.
    pub fn peek(&self) -> Result<&Value, VmError> {
        self.data.last().ok_or(VmError::StackUnderflow { needed: 1, had: 0 })
    }

    /// Duplicate the top value.
    pub fn dup(&mut self) -> Result<(), VmError> {
        let val = *self.peek()?;
        self.push(val)
    }

    /// Swap the top two values.
    pub fn swap(&mut self) -> Result<(), VmError> {
        let len = self.data.len();
        if len < 2 {
            return Err(VmError::StackUnderflow { needed: 2, had: len });
        }
        self.data.swap(len - 1, len - 2);
        Ok(())
    }

    /// Current stack size.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear the stack.
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

/// Encode a u64 as a 32-byte value (little-endian).
pub fn u64_to_value(n: u64) -> Value {
    let mut val = ZERO;
    val[..8].copy_from_slice(&n.to_le_bytes());
    val
}

/// Decode a 32-byte value as u64 (little-endian).
pub fn value_to_u64(val: &Value) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&val[..8]);
    u64::from_le_bytes(bytes)
}

/// Encode a bool as a value.
pub fn bool_to_value(b: bool) -> Value {
    let mut val = ZERO;
    if b { val[0] = 1; }
    val
}

/// Decode a value as bool (nonzero = true).
pub fn value_to_bool(val: &Value) -> bool {
    val.iter().any(|&b| b != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut stack = Stack::new();
        let val = u64_to_value(42);
        stack.push(val).unwrap();
        assert_eq!(stack.len(), 1);
        let popped = stack.pop().unwrap();
        assert_eq!(value_to_u64(&popped), 42);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_stack_overflow() {
        let mut stack = Stack::with_max_depth(2);
        stack.push(ZERO).unwrap();
        stack.push(ZERO).unwrap();
        assert!(stack.push(ZERO).is_err());
    }

    #[test]
    fn test_stack_underflow() {
        let mut stack = Stack::new();
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_dup() {
        let mut stack = Stack::new();
        stack.push(u64_to_value(99)).unwrap();
        stack.dup().unwrap();
        assert_eq!(stack.len(), 2);
        assert_eq!(value_to_u64(&stack.pop().unwrap()), 99);
        assert_eq!(value_to_u64(&stack.pop().unwrap()), 99);
    }

    #[test]
    fn test_swap() {
        let mut stack = Stack::new();
        stack.push(u64_to_value(1)).unwrap();
        stack.push(u64_to_value(2)).unwrap();
        stack.swap().unwrap();
        assert_eq!(value_to_u64(&stack.pop().unwrap()), 1);
        assert_eq!(value_to_u64(&stack.pop().unwrap()), 2);
    }

    #[test]
    fn test_bool_encoding() {
        assert!(value_to_bool(&bool_to_value(true)));
        assert!(!value_to_bool(&bool_to_value(false)));
    }
}
