//! NXVM - Main virtual machine execution engine.

use crate::error::VmError;
use crate::gas::GasCounter;
use crate::opcodes::Opcode;
use crate::stack::*;
use crate::state::VmState;

/// Maximum call depth.
pub const MAX_CALL_DEPTH: usize = 256;

/// VM execution result.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub return_data: Vec<u8>,
    pub gas_used: u64,
    pub logs: Vec<LogEntry>,
}

/// Emitted log entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub event: String,
    pub data: Vec<Value>,
}

/// Bytecode instruction in the VM's internal format.
#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: Option<Vec<u8>>,
}

/// The NEXARA Virtual Machine.
pub struct Vm {
    stack: Stack,
    state: VmState,
    gas: GasCounter,
    program: Vec<Instruction>,
    pc: usize,
    call_depth: usize,
    logs: Vec<LogEntry>,
    halted: bool,
}

impl Vm {
    /// Create a new VM with program and gas limit.
    pub fn new(program: Vec<Instruction>, gas_limit: u64) -> Self {
        Vm {
            stack: Stack::new(),
            state: VmState::new(),
            gas: GasCounter::new(gas_limit),
            program,
            pc: 0,
            call_depth: 0,
            logs: Vec::new(),
            halted: false,
        }
    }

    /// Set the execution context.
    pub fn set_context(&mut self, caller: Value, call_value: u64, block_height: u64) {
        self.state.caller = caller;
        self.state.call_value = call_value;
        self.state.block_height = block_height;
    }

    /// Get mutable reference to state (for preloading storage).
    pub fn state_mut(&mut self) -> &mut VmState {
        &mut self.state
    }

    /// Get reference to state.
    pub fn state(&self) -> &VmState {
        &self.state
    }

    /// Execute the program and return result.
    pub fn execute(&mut self) -> Result<ExecutionResult, VmError> {
        while !self.halted && self.pc < self.program.len() {
            self.step()?;
        }
        Ok(ExecutionResult {
            success: true,
            return_data: Vec::new(),
            gas_used: self.gas.gas_used(),
            logs: std::mem::take(&mut self.logs),
        })
    }

    /// Execute a single step.
    fn step(&mut self) -> Result<(), VmError> {
        if self.pc >= self.program.len() {
            self.halted = true;
            return Ok(());
        }

        let instr = self.program[self.pc].clone();
        self.gas.consume(instr.opcode)?;

        match instr.opcode {
            Opcode::Nop => {}
            Opcode::Halt => { self.halted = true; }

            Opcode::Push => {
                let operand = instr.operand.as_ref()
                    .ok_or(VmError::InvalidOperand(self.pc))?;
                let mut val = ZERO;
                let len = operand.len().min(32);
                val[..len].copy_from_slice(&operand[..len]);
                self.stack.push(val)?;
            }
            Opcode::Pop => { self.stack.pop()?; }
            Opcode::Dup => { self.stack.dup()?; }
            Opcode::Swap => { self.stack.swap()?; }

            // Arithmetic
            Opcode::Add => {
                let b = value_to_u64(&self.stack.pop()?);
                let a = value_to_u64(&self.stack.pop()?);
                self.stack.push(u64_to_value(a.wrapping_add(b)))?;
            }
            Opcode::Sub => {
                let b = value_to_u64(&self.stack.pop()?);
                let a = value_to_u64(&self.stack.pop()?);
                self.stack.push(u64_to_value(a.wrapping_sub(b)))?;
            }
            Opcode::Mul => {
                let b = value_to_u64(&self.stack.pop()?);
                let a = value_to_u64(&self.stack.pop()?);
                self.stack.push(u64_to_value(a.wrapping_mul(b)))?;
            }
            Opcode::Div => {
                let b = value_to_u64(&self.stack.pop()?);
                let a = value_to_u64(&self.stack.pop()?);
                if b == 0 { return Err(VmError::DivisionByZero); }
                self.stack.push(u64_to_value(a / b))?;
            }
            Opcode::Mod => {
                let b = value_to_u64(&self.stack.pop()?);
                let a = value_to_u64(&self.stack.pop()?);
                if b == 0 { return Err(VmError::DivisionByZero); }
                self.stack.push(u64_to_value(a % b))?;
            }
            Opcode::Neg => {
                let a = value_to_u64(&self.stack.pop()?);
                self.stack.push(u64_to_value(0u64.wrapping_sub(a)))?;
            }

            // Comparison
            Opcode::Eq => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(bool_to_value(a == b))?;
            }
            Opcode::Ne => {
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                self.stack.push(bool_to_value(a != b))?;
            }
            Opcode::Lt => {
                let b = value_to_u64(&self.stack.pop()?);
                let a = value_to_u64(&self.stack.pop()?);
                self.stack.push(bool_to_value(a < b))?;
            }
            Opcode::Gt => {
                let b = value_to_u64(&self.stack.pop()?);
                let a = value_to_u64(&self.stack.pop()?);
                self.stack.push(bool_to_value(a > b))?;
            }
            Opcode::Le => {
                let b = value_to_u64(&self.stack.pop()?);
                let a = value_to_u64(&self.stack.pop()?);
                self.stack.push(bool_to_value(a <= b))?;
            }
            Opcode::Ge => {
                let b = value_to_u64(&self.stack.pop()?);
                let a = value_to_u64(&self.stack.pop()?);
                self.stack.push(bool_to_value(a >= b))?;
            }

            // Logic
            Opcode::And => {
                let b = value_to_bool(&self.stack.pop()?);
                let a = value_to_bool(&self.stack.pop()?);
                self.stack.push(bool_to_value(a && b))?;
            }
            Opcode::Or => {
                let b = value_to_bool(&self.stack.pop()?);
                let a = value_to_bool(&self.stack.pop()?);
                self.stack.push(bool_to_value(a || b))?;
            }
            Opcode::Not => {
                let a = value_to_bool(&self.stack.pop()?);
                self.stack.push(bool_to_value(!a))?;
            }

            // Flow
            Opcode::Jump => {
                let dest = self.read_operand_u32(&instr)? as usize;
                if dest >= self.program.len() {
                    return Err(VmError::InvalidJump(dest));
                }
                self.pc = dest;
                return Ok(());
            }
            Opcode::JumpIf => {
                let dest = self.read_operand_u32(&instr)? as usize;
                let cond = value_to_bool(&self.stack.pop()?);
                if cond {
                    if dest >= self.program.len() {
                        return Err(VmError::InvalidJump(dest));
                    }
                    self.pc = dest;
                    return Ok(());
                }
            }
            Opcode::JumpIfNot => {
                let dest = self.read_operand_u32(&instr)? as usize;
                let cond = value_to_bool(&self.stack.pop()?);
                if !cond {
                    if dest >= self.program.len() {
                        return Err(VmError::InvalidJump(dest));
                    }
                    self.pc = dest;
                    return Ok(());
                }
            }
            Opcode::Call => {
                if self.call_depth >= MAX_CALL_DEPTH {
                    return Err(VmError::CallDepthExceeded(self.call_depth));
                }
                self.call_depth += 1;
                // In a full implementation, this would save the frame and jump
            }
            Opcode::Return => {
                if self.call_depth > 0 { self.call_depth -= 1; }
                self.halted = true;
            }

            // Memory & Storage
            Opcode::Load => {
                let slot = self.read_operand_u32(&instr)?;
                let val = self.state.load_local(slot);
                self.stack.push(val)?;
            }
            Opcode::Store => {
                let slot = self.read_operand_u32(&instr)?;
                let val = self.stack.pop()?;
                self.state.store_local(slot, val);
            }
            Opcode::MemLoad => {
                let offset = self.read_operand_u32(&instr)? as usize;
                let val = self.state.mem_load(offset);
                self.stack.push(val)?;
            }
            Opcode::MemStore => {
                let offset = self.read_operand_u32(&instr)? as usize;
                let val = self.stack.pop()?;
                self.state.mem_store(offset, &val);
            }
            Opcode::SLoad => {
                let slot = self.read_operand_u32(&instr)?;
                let val = self.state.sload(slot);
                self.stack.push(val)?;
            }
            Opcode::SStore => {
                let slot = self.read_operand_u32(&instr)?;
                let val = self.stack.pop()?;
                self.state.sstore(slot, val);
            }

            // Context
            Opcode::MsgSender => { self.stack.push(self.state.caller)?; }
            Opcode::MsgValue => { self.stack.push(u64_to_value(self.state.call_value))?; }
            Opcode::BlockHeight => { self.stack.push(u64_to_value(self.state.block_height))?; }
            Opcode::Timestamp => { self.stack.push(u64_to_value(self.state.timestamp))?; }
            Opcode::ChainId => { self.stack.push(u64_to_value(self.state.chain_id))?; }
            Opcode::GasRemaining => { self.stack.push(u64_to_value(self.gas.gas_remaining()))?; }

            // Events
            Opcode::Emit => {
                let event_name = instr.operand.as_ref()
                    .map(|o| String::from_utf8_lossy(o).to_string())
                    .unwrap_or_default();
                let data_val = self.stack.pop()?;
                self.logs.push(LogEntry {
                    event: event_name,
                    data: vec![data_val],
                });
            }
            Opcode::Revert => {
                let msg = instr.operand.as_ref()
                    .map(|o| String::from_utf8_lossy(o).to_string())
                    .unwrap_or_else(|| "Execution reverted".into());
                return Err(VmError::Revert(msg));
            }

            // Crypto
            Opcode::Hash => {
                let val = self.stack.pop()?;
                let hash = blake3::hash(&val);
                self.stack.push(*hash.as_bytes())?;
            }
            Opcode::VerifySig => {
                // Simplified: pop sig, msg, pubkey; push true
                self.stack.pop()?;
                self.stack.pop()?;
                self.stack.pop()?;
                self.stack.push(bool_to_value(true))?;
            }

            // Cross-shard (placeholder)
            Opcode::CrossShardCall | Opcode::CrossShardReturn => {
                // Cross-shard operations are handled by the runtime
            }
        }

        self.pc += 1;
        Ok(())
    }

    fn read_operand_u32(&self, instr: &Instruction) -> Result<u32, VmError> {
        let operand = instr.operand.as_ref()
            .ok_or(VmError::InvalidOperand(self.pc))?;
        if operand.len() < 4 {
            // Pad short operands
            let mut buf = [0u8; 4];
            let len = operand.len().min(4);
            buf[..len].copy_from_slice(&operand[..len]);
            return Ok(u32::from_le_bytes(buf));
        }
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&operand[..4]);
        Ok(u32::from_le_bytes(buf))
    }
}

/// Helper to build instruction lists for testing.
pub fn instr(opcode: Opcode, operand: Option<Vec<u8>>) -> Instruction {
    Instruction { opcode, operand }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_halt() {
        let program = vec![
            instr(Opcode::Push, Some(42u64.to_le_bytes().to_vec())),
            instr(Opcode::Halt, None),
        ];
        let mut vm = Vm::new(program, 100_000);
        let result = vm.execute().unwrap();
        assert!(result.success);
        assert!(result.gas_used > 0);
    }

    #[test]
    fn test_add() {
        let program = vec![
            instr(Opcode::Push, Some(10u64.to_le_bytes().to_vec())),
            instr(Opcode::Push, Some(20u64.to_le_bytes().to_vec())),
            instr(Opcode::Add, None),
            instr(Opcode::Halt, None),
        ];
        let mut vm = Vm::new(program, 100_000);
        vm.execute().unwrap();
        let top = vm.stack.pop().unwrap();
        assert_eq!(value_to_u64(&top), 30);
    }

    #[test]
    fn test_div_by_zero() {
        let program = vec![
            instr(Opcode::Push, Some(10u64.to_le_bytes().to_vec())),
            instr(Opcode::Push, Some(0u64.to_le_bytes().to_vec())),
            instr(Opcode::Div, None),
        ];
        let mut vm = Vm::new(program, 100_000);
        let err = vm.execute().unwrap_err();
        assert!(matches!(err, VmError::DivisionByZero));
    }

    #[test]
    fn test_comparison() {
        let program = vec![
            instr(Opcode::Push, Some(5u64.to_le_bytes().to_vec())),
            instr(Opcode::Push, Some(10u64.to_le_bytes().to_vec())),
            instr(Opcode::Lt, None),
            instr(Opcode::Halt, None),
        ];
        let mut vm = Vm::new(program, 100_000);
        vm.execute().unwrap();
        assert!(value_to_bool(&vm.stack.pop().unwrap()));
    }

    #[test]
    fn test_storage() {
        let program = vec![
            instr(Opcode::Push, Some(999u64.to_le_bytes().to_vec())),
            instr(Opcode::SStore, Some(0u32.to_le_bytes().to_vec())),
            instr(Opcode::SLoad, Some(0u32.to_le_bytes().to_vec())),
            instr(Opcode::Halt, None),
        ];
        let mut vm = Vm::new(program, 100_000);
        vm.execute().unwrap();
        let val = vm.stack.pop().unwrap();
        assert_eq!(value_to_u64(&val), 999);
    }

    #[test]
    fn test_out_of_gas() {
        let program = vec![
            instr(Opcode::Push, Some(1u64.to_le_bytes().to_vec())),
        ];
        let mut vm = Vm::new(program, 1); // only 1 gas
        let err = vm.execute().unwrap_err();
        assert!(matches!(err, VmError::OutOfGas { .. }));
    }

    #[test]
    fn test_revert() {
        let program = vec![
            instr(Opcode::Revert, Some(b"boom".to_vec())),
        ];
        let mut vm = Vm::new(program, 100_000);
        let err = vm.execute().unwrap_err();
        match err {
            VmError::Revert(msg) => assert_eq!(msg, "boom"),
            _ => panic!("Expected Revert"),
        }
    }

    #[test]
    fn test_hash_opcode() {
        let program = vec![
            instr(Opcode::Push, Some(42u64.to_le_bytes().to_vec())),
            instr(Opcode::Hash, None),
            instr(Opcode::Halt, None),
        ];
        let mut vm = Vm::new(program, 100_000);
        vm.execute().unwrap();
        let hash_val = vm.stack.pop().unwrap();
        assert_ne!(hash_val, ZERO);
    }

    #[test]
    fn test_context() {
        let program = vec![
            instr(Opcode::BlockHeight, None),
            instr(Opcode::Halt, None),
        ];
        let mut vm = Vm::new(program, 100_000);
        vm.set_context(ZERO, 0, 12345);
        vm.execute().unwrap();
        assert_eq!(value_to_u64(&vm.stack.pop().unwrap()), 12345);
    }
}
