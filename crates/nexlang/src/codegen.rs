//! NEXLANG Code Generator - compiles AST to NXVM bytecode.

use crate::ast::*;
use crate::errors::NexlangError;
use std::collections::HashMap;

/// Opcodes for the NXVM virtual machine.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum Opcode {
    Nop = 0x00,
    Push = 0x01,
    Pop = 0x02,
    Add = 0x10,
    Sub = 0x11,
    Mul = 0x12,
    Div = 0x13,
    Mod = 0x14,
    And = 0x20,
    Or = 0x21,
    Not = 0x22,
    Eq = 0x30,
    Ne = 0x31,
    Lt = 0x32,
    Gt = 0x33,
    Le = 0x34,
    Ge = 0x35,
    Jump = 0x40,
    JumpIf = 0x41,
    JumpIfNot = 0x42,
    Call = 0x50,
    Return = 0x51,
    Load = 0x60,
    Store = 0x61,
    SLoad = 0x70,
    SStore = 0x71,
    MsgSender = 0x80,
    MsgValue = 0x81,
    BlockHeight = 0x82,
    Emit = 0x90,
    Revert = 0xF0,
    Halt = 0xFF,
}

/// A single bytecode instruction.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand: Option<Vec<u8>>,
}

/// Compiled contract.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompiledContract {
    pub name: String,
    pub functions: HashMap<String, CompiledFunction>,
    pub state_layout: HashMap<String, u32>,
}

/// Compiled function.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompiledFunction {
    pub name: String,
    pub param_count: usize,
    pub instructions: Vec<Instruction>,
}

/// Code generator state.
pub struct CodeGenerator {
    locals: HashMap<String, u32>,
    next_local: u32,
    state_slots: HashMap<String, u32>,
    next_state_slot: u32,
    instructions: Vec<Instruction>,
    label_counter: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            locals: HashMap::new(),
            next_local: 0,
            state_slots: HashMap::new(),
            next_state_slot: 0,
            instructions: Vec::new(),
            label_counter: 0,
        }
    }

    /// Compile a full program.
    pub fn compile_program(&mut self, program: &Program) -> Result<Vec<CompiledContract>, NexlangError> {
        let mut compiled = Vec::new();
        for contract in &program.contracts {
            compiled.push(self.compile_contract(contract)?);
        }
        Ok(compiled)
    }

    fn compile_contract(&mut self, contract: &Contract) -> Result<CompiledContract, NexlangError> {
        self.state_slots.clear();
        self.next_state_slot = 0;

        // Assign storage slots for state variables
        for var in &contract.state_vars {
            self.state_slots.insert(var.name.clone(), self.next_state_slot);
            self.next_state_slot += 1;
        }

        let mut functions = HashMap::new();
        for func in &contract.functions {
            let compiled_fn = self.compile_function(func)?;
            functions.insert(func.name.clone(), compiled_fn);
        }

        Ok(CompiledContract {
            name: contract.name.clone(),
            functions,
            state_layout: self.state_slots.clone(),
        })
    }

    fn compile_function(&mut self, func: &Function) -> Result<CompiledFunction, NexlangError> {
        self.locals.clear();
        self.next_local = 0;
        self.instructions.clear();

        // Allocate param slots
        for param in &func.params {
            let slot = self.next_local;
            self.locals.insert(param.name.clone(), slot);
            self.next_local += 1;
        }

        for stmt in &func.body {
            self.compile_statement(stmt)?;
        }

        // Implicit return at end
        self.emit(Opcode::Return, None);

        Ok(CompiledFunction {
            name: func.name.clone(),
            param_count: func.params.len(),
            instructions: std::mem::take(&mut self.instructions),
        })
    }

    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), NexlangError> {
        match stmt {
            Statement::Let { name, value, .. } => {
                self.compile_expression(value)?;
                let slot = self.next_local;
                self.locals.insert(name.clone(), slot);
                self.next_local += 1;
                self.emit(Opcode::Store, Some(slot.to_le_bytes().to_vec()));
            }
            Statement::Assign { target, value } => {
                self.compile_expression(value)?;
                match target {
                    Expression::Identifier(name) => {
                        if let Some(&slot) = self.state_slots.get(name) {
                            self.emit(Opcode::SStore, Some(slot.to_le_bytes().to_vec()));
                        } else if let Some(&slot) = self.locals.get(name) {
                            self.emit(Opcode::Store, Some(slot.to_le_bytes().to_vec()));
                        }
                    }
                    _ => {
                        // For complex assignments (field/index), just Store to temp
                        self.emit(Opcode::Store, Some(0u32.to_le_bytes().to_vec()));
                    }
                }
            }
            Statement::Return(Some(expr)) => {
                self.compile_expression(expr)?;
                self.emit(Opcode::Return, None);
            }
            Statement::Return(None) => {
                self.emit(Opcode::Return, None);
            }
            Statement::If { condition, then_body, else_body } => {
                self.compile_expression(condition)?;
                let else_label = self.new_label();
                let end_label = self.new_label();
                self.emit(Opcode::JumpIfNot, Some((else_label as u32).to_le_bytes().to_vec()));
                for s in then_body { self.compile_statement(s)?; }
                self.emit(Opcode::Jump, Some((end_label as u32).to_le_bytes().to_vec()));
                // else_label target (placeholder — actual label resolution not done here)
                if let Some(eb) = else_body {
                    for s in eb { self.compile_statement(s)?; }
                }
                // end_label target
            }
            Statement::While { condition, body } => {
                let loop_start = self.instructions.len();
                self.compile_expression(condition)?;
                let end_label = self.new_label();
                self.emit(Opcode::JumpIfNot, Some((end_label as u32).to_le_bytes().to_vec()));
                for s in body { self.compile_statement(s)?; }
                self.emit(Opcode::Jump, Some((loop_start as u32).to_le_bytes().to_vec()));
                // end_label target
            }
            Statement::Emit { event_name, args } => {
                for arg in args {
                    self.compile_expression(arg)?;
                }
                let name_bytes = event_name.as_bytes().to_vec();
                self.emit(Opcode::Emit, Some(name_bytes));
            }
            Statement::Require { condition, message } => {
                self.compile_expression(condition)?;
                let msg_bytes = message.as_bytes().to_vec();
                self.emit(Opcode::JumpIfNot, None);
                self.emit(Opcode::Revert, Some(msg_bytes));
            }
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                self.emit(Opcode::Pop, None);
            }
            Statement::For { body, .. } => {
                for s in body { self.compile_statement(s)?; }
            }
            Statement::Block(stmts) => {
                for s in stmts { self.compile_statement(s)?; }
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<(), NexlangError> {
        match expr {
            Expression::IntLiteral(n) => {
                self.emit(Opcode::Push, Some((*n as u64).to_le_bytes().to_vec()));
            }
            Expression::BoolLiteral(b) => {
                self.emit(Opcode::Push, Some(vec![if *b { 1 } else { 0 }]));
            }
            Expression::StringLiteral(s) => {
                self.emit(Opcode::Push, Some(s.as_bytes().to_vec()));
            }
            Expression::AddressLiteral(addr) => {
                self.emit(Opcode::Push, Some(addr.as_bytes().to_vec()));
            }
            Expression::Identifier(name) => {
                if let Some(&slot) = self.state_slots.get(name) {
                    self.emit(Opcode::SLoad, Some(slot.to_le_bytes().to_vec()));
                } else if let Some(&slot) = self.locals.get(name) {
                    self.emit(Opcode::Load, Some(slot.to_le_bytes().to_vec()));
                } else {
                    // Unknown variable — push 0 as fallback
                    self.emit(Opcode::Push, Some(vec![0]));
                }
            }
            Expression::MsgSender => { self.emit(Opcode::MsgSender, None); }
            Expression::MsgValue => { self.emit(Opcode::MsgValue, None); }
            Expression::BlockHeight => { self.emit(Opcode::BlockHeight, None); }
            Expression::BinaryOp { left, op, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                let opcode = match op {
                    BinaryOperator::Add => Opcode::Add,
                    BinaryOperator::Sub => Opcode::Sub,
                    BinaryOperator::Mul => Opcode::Mul,
                    BinaryOperator::Div => Opcode::Div,
                    BinaryOperator::Mod => Opcode::Mod,
                    BinaryOperator::Eq => Opcode::Eq,
                    BinaryOperator::Ne => Opcode::Ne,
                    BinaryOperator::Lt => Opcode::Lt,
                    BinaryOperator::Gt => Opcode::Gt,
                    BinaryOperator::Le => Opcode::Le,
                    BinaryOperator::Ge => Opcode::Ge,
                    BinaryOperator::And => Opcode::And,
                    BinaryOperator::Or => Opcode::Or,
                    _ => Opcode::Nop,
                };
                self.emit(opcode, None);
            }
            Expression::UnaryOp { op, operand } => {
                self.compile_expression(operand)?;
                match op {
                    UnaryOperator::Not => self.emit(Opcode::Not, None),
                    UnaryOperator::Neg => {
                        self.emit(Opcode::Push, Some(0u64.to_le_bytes().to_vec()));
                        self.emit(Opcode::Sub, None);
                    }
                }
            }
            Expression::FunctionCall { name, args } => {
                for arg in args {
                    self.compile_expression(arg)?;
                }
                let name_bytes = name.as_bytes().to_vec();
                self.emit(Opcode::Call, Some(name_bytes));
            }
            Expression::MethodCall { object, method, args } => {
                self.compile_expression(object)?;
                for arg in args {
                    self.compile_expression(arg)?;
                }
                let method_bytes = method.as_bytes().to_vec();
                self.emit(Opcode::Call, Some(method_bytes));
            }
            Expression::FieldAccess { object, .. } => {
                self.compile_expression(object)?;
            }
            Expression::IndexAccess { object, index } => {
                self.compile_expression(object)?;
                self.compile_expression(index)?;
            }
            Expression::ArrayLiteral(elems) => {
                for elem in elems {
                    self.compile_expression(elem)?;
                }
                self.emit(Opcode::Push, Some((elems.len() as u64).to_le_bytes().to_vec()));
            }
            Expression::SelfRef => {
                self.emit(Opcode::Push, Some(vec![0]));
            }
        }
        Ok(())
    }

    fn emit(&mut self, opcode: Opcode, operand: Option<Vec<u8>>) {
        self.instructions.push(Instruction { opcode, operand });
    }

    fn new_label(&mut self) -> usize {
        let label = self.label_counter;
        self.label_counter += 1;
        label
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialize compiled contract to bytes.
pub fn serialize_bytecode(contract: &CompiledContract) -> Vec<u8> {
    let mut bytes = Vec::new();
    // Magic header
    bytes.extend_from_slice(b"NXVM");
    // Version
    bytes.push(1);
    // Contract name length + name
    let name_bytes = contract.name.as_bytes();
    bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
    bytes.extend_from_slice(name_bytes);
    // Function count
    bytes.extend_from_slice(&(contract.functions.len() as u32).to_le_bytes());
    for (fname, func) in &contract.functions {
        // Function name
        let fn_bytes = fname.as_bytes();
        bytes.extend_from_slice(&(fn_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(fn_bytes);
        // Param count
        bytes.extend_from_slice(&(func.param_count as u32).to_le_bytes());
        // Instruction count
        bytes.extend_from_slice(&(func.instructions.len() as u32).to_le_bytes());
        for instr in &func.instructions {
            bytes.push(instr.opcode.clone() as u8);
            if let Some(ref operand) = instr.operand {
                bytes.extend_from_slice(&(operand.len() as u32).to_le_bytes());
                bytes.extend_from_slice(operand);
            } else {
                bytes.extend_from_slice(&0u32.to_le_bytes());
            }
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn compile(source: &str) -> Vec<CompiledContract> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let prog = parser.parse_program().unwrap();
        let mut gen = CodeGenerator::new();
        gen.compile_program(&prog).unwrap()
    }

    #[test]
    fn test_compile_empty_contract() {
        let contracts = compile("contract Token { }");
        assert_eq!(contracts.len(), 1);
        assert_eq!(contracts[0].name, "Token");
    }

    #[test]
    fn test_compile_simple_function() {
        let contracts = compile("contract Token {\n  pub fn get() -> u128 {\n    return 42;\n  }\n}");
        assert_eq!(contracts[0].functions.len(), 1);
        let func = &contracts[0].functions["get"];
        assert!(!func.instructions.is_empty());
        assert_eq!(func.instructions[0].opcode, Opcode::Push);
    }

    #[test]
    fn test_compile_arithmetic() {
        let contracts = compile("contract Math {\n  pub fn add(a: u128, b: u128) -> u128 {\n    return a + b;\n  }\n}");
        let func = &contracts[0].functions["add"];
        let has_add = func.instructions.iter().any(|i| i.opcode == Opcode::Add);
        assert!(has_add);
    }

    #[test]
    fn test_state_storage() {
        let contracts = compile("contract Token {\n  let mut supply: u128 = 0;\n  pub fn set(val: u128) {\n    supply = val;\n  }\n}");
        assert!(contracts[0].state_layout.contains_key("supply"));
    }

    #[test]
    fn test_serialize_bytecode() {
        let contracts = compile("contract Token {\n  pub fn get() -> u128 {\n    return 1;\n  }\n}");
        let bytes = serialize_bytecode(&contracts[0]);
        assert_eq!(&bytes[0..4], b"NXVM");
        assert_eq!(bytes[4], 1); // version
    }

    #[test]
    fn test_opcode_values() {
        assert_eq!(Opcode::Nop as u8, 0x00);
        assert_eq!(Opcode::Push as u8, 0x01);
        assert_eq!(Opcode::Halt as u8, 0xFF);
        assert_eq!(Opcode::Add as u8, 0x10);
    }
}
