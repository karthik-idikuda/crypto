//! Integration test: VM execution
//! Instruction construction → VM execution → result verification

use nxvm::{
    Vm, Opcode,
    stack::{u64_to_value, bool_to_value},
    vm::instr,
};

#[test]
fn test_simple_halt() {
    let program = vec![instr(Opcode::Halt, None)];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
    // Halt costs 0 gas per GAS_TABLE
}

#[test]
fn test_push_and_halt() {
    let val = u64_to_value(42);
    let program = vec![
        instr(Opcode::Push, Some(val.to_vec())),
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
}

#[test]
fn test_arithmetic_add() {
    let program = vec![
        instr(Opcode::Push, Some(u64_to_value(10).to_vec())),
        instr(Opcode::Push, Some(u64_to_value(20).to_vec())),
        instr(Opcode::Add, None),
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
}

#[test]
fn test_arithmetic_sub() {
    let program = vec![
        instr(Opcode::Push, Some(u64_to_value(50).to_vec())),
        instr(Opcode::Push, Some(u64_to_value(30).to_vec())),
        instr(Opcode::Sub, None),
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
}

#[test]
fn test_arithmetic_mul() {
    let program = vec![
        instr(Opcode::Push, Some(u64_to_value(6).to_vec())),
        instr(Opcode::Push, Some(u64_to_value(7).to_vec())),
        instr(Opcode::Mul, None),
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
}

#[test]
fn test_arithmetic_div() {
    let program = vec![
        instr(Opcode::Push, Some(u64_to_value(100).to_vec())),
        instr(Opcode::Push, Some(u64_to_value(4).to_vec())),
        instr(Opcode::Div, None),
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
}

#[test]
fn test_division_by_zero() {
    let program = vec![
        instr(Opcode::Push, Some(u64_to_value(42).to_vec())),
        instr(Opcode::Push, Some(u64_to_value(0).to_vec())),
        instr(Opcode::Div, None),
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute();
    assert!(result.is_err(), "Division by zero should be an error");
}

#[test]
fn test_comparison_eq() {
    let program = vec![
        instr(Opcode::Push, Some(u64_to_value(42).to_vec())),
        instr(Opcode::Push, Some(u64_to_value(42).to_vec())),
        instr(Opcode::Eq, None),
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
}

#[test]
fn test_comparison_lt() {
    let program = vec![
        instr(Opcode::Push, Some(u64_to_value(5).to_vec())),
        instr(Opcode::Push, Some(u64_to_value(10).to_vec())),
        instr(Opcode::Lt, None),
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
}

#[test]
fn test_dup_and_swap() {
    let program = vec![
        instr(Opcode::Push, Some(u64_to_value(1).to_vec())),
        instr(Opcode::Dup, None),
        instr(Opcode::Push, Some(u64_to_value(2).to_vec())),
        instr(Opcode::Swap, None),
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
}

#[test]
fn test_logic_and_or_not() {
    let program = vec![
        instr(Opcode::Push, Some(bool_to_value(true).to_vec())),
        instr(Opcode::Push, Some(bool_to_value(false).to_vec())),
        instr(Opcode::And, None),
        // Stack: false
        instr(Opcode::Not, None),
        // Stack: true
        instr(Opcode::Push, Some(bool_to_value(false).to_vec())),
        instr(Opcode::Or, None),
        // Stack: true
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
}

#[test]
fn test_store_and_load() {
    // SStore: reads slot from operand (u32 le), pops value from stack
    // SLoad: reads slot from operand (u32 le), pushes value to stack
    let slot_bytes = 0u32.to_le_bytes().to_vec();
    let val = u64_to_value(999);
    let program = vec![
        instr(Opcode::Push, Some(val.to_vec())),            // push value
        instr(Opcode::SStore, Some(slot_bytes.clone())),    // sstore to slot 0
        instr(Opcode::SLoad, Some(slot_bytes)),             // sload from slot 0
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
}

#[test]
fn test_emit_event() {
    // Emit: takes event name from operand, pops one value from stack
    let program = vec![
        instr(Opcode::Push, Some(u64_to_value(42).to_vec())),        // data value
        instr(Opcode::Emit, Some(b"Transfer".to_vec())),             // emit "Transfer"
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
    assert!(!result.logs.is_empty(), "Should have emitted at least one log");
    assert_eq!(result.logs[0].event, "Transfer");
}

#[test]
fn test_out_of_gas() {
    // Very low gas limit with many operations
    let program = vec![
        instr(Opcode::Push, Some(u64_to_value(1).to_vec())),
        instr(Opcode::Push, Some(u64_to_value(2).to_vec())),
        instr(Opcode::Add, None),
        instr(Opcode::Push, Some(u64_to_value(3).to_vec())),
        instr(Opcode::Add, None),
        instr(Opcode::Push, Some(u64_to_value(4).to_vec())),
        instr(Opcode::Mul, None),
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 5); // Tiny gas limit
    let result = vm.execute();
    assert!(result.is_err(), "Should run out of gas");
}

#[test]
fn test_stack_underflow() {
    let program = vec![
        instr(Opcode::Pop, None), // Pop from empty stack
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute();
    assert!(result.is_err(), "Should fail on stack underflow");
}

#[test]
fn test_conditional_jump() {
    // JumpIf takes destination from operand (u32 le), pops condition from stack
    let program = vec![
        instr(Opcode::Push, Some(bool_to_value(true).to_vec())),         // 0: push true
        instr(Opcode::JumpIf, Some(3u32.to_le_bytes().to_vec())),       // 1: jump to 3 if true
        instr(Opcode::Push, Some(u64_to_value(99).to_vec())),           // 2: skipped
        instr(Opcode::Push, Some(u64_to_value(42).to_vec())),           // 3: lands here
        instr(Opcode::Halt, None),                                       // 4: halt
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
}

#[test]
fn test_revert() {
    // Revert: takes message from operand, doesn't pop from stack
    let program = vec![
        instr(Opcode::Revert, Some(b"test revert".to_vec())),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute();
    assert!(result.is_err(), "Revert should produce an error");
}

#[test]
fn test_vm_context_fields() {
    let caller = u64_to_value(0xBEEF);
    let program = vec![
        instr(Opcode::MsgSender, None),
        instr(Opcode::Pop, None),
        instr(Opcode::BlockHeight, None),
        instr(Opcode::Pop, None),
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    vm.set_context(caller, 1000, 42);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
}

#[test]
fn test_gas_tracking() {
    let program = vec![
        instr(Opcode::Push, Some(u64_to_value(1).to_vec())),
        instr(Opcode::Push, Some(u64_to_value(2).to_vec())),
        instr(Opcode::Add, None),
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
    assert!(result.gas_used > 0, "Gas should be consumed");
    assert!(result.gas_used < 100_000, "Should not use all gas for simple program");
}

#[test]
fn test_nop() {
    let program = vec![
        instr(Opcode::Nop, None),
        instr(Opcode::Nop, None),
        instr(Opcode::Nop, None),
        instr(Opcode::Halt, None),
    ];
    let mut vm = Vm::new(program, 100_000);
    let result = vm.execute().expect("should execute");
    assert!(result.success);
}
