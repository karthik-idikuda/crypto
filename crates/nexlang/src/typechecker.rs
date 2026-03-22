//! NEXLANG Type Checker - validates types across the AST.

use std::collections::HashMap;
use crate::ast::*;
use crate::errors::NexlangError;

/// Type environment mapping variable names to their types.
#[derive(Debug, Clone)]
pub struct TypeEnv {
    scopes: Vec<HashMap<String, NexType>>,
    functions: HashMap<String, (Vec<NexType>, Option<NexType>)>,
    events: HashMap<String, Vec<NexType>>,
    state_vars: HashMap<String, NexType>,
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            events: HashMap::new(),
            state_vars: HashMap::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: &str, ty: NexType) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&NexType> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        self.state_vars.get(name)
    }

    pub fn define_function(&mut self, name: &str, params: Vec<NexType>, ret: Option<NexType>) {
        self.functions.insert(name.to_string(), (params, ret));
    }

    pub fn lookup_function(&self, name: &str) -> Option<&(Vec<NexType>, Option<NexType>)> {
        self.functions.get(name)
    }

    pub fn define_event(&mut self, name: &str, fields: Vec<NexType>) {
        self.events.insert(name.to_string(), fields);
    }

    pub fn define_state_var(&mut self, name: &str, ty: NexType) {
        self.state_vars.insert(name.to_string(), ty);
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// Type checker for NEXLANG programs.
pub struct TypeChecker {
    env: TypeEnv,
    errors: Vec<NexlangError>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            env: TypeEnv::new(),
            errors: Vec::new(),
        }
    }

    /// Check an entire program, returning collected errors.
    pub fn check_program(&mut self, program: &Program) -> Vec<NexlangError> {
        for contract in &program.contracts {
            self.check_contract(contract);
        }
        std::mem::take(&mut self.errors)
    }

    fn check_contract(&mut self, contract: &Contract) {
        // Register state vars
        for var in &contract.state_vars {
            self.env.define_state_var(&var.name, var.ty.clone());
        }
        // Register events
        for event in &contract.events {
            let types: Vec<NexType> = event.fields.iter().map(|f| f.ty.clone()).collect();
            self.env.define_event(&event.name, types);
        }
        // Register functions
        for func in &contract.functions {
            let param_types: Vec<NexType> = func.params.iter().map(|p| p.ty.clone()).collect();
            self.env.define_function(&func.name, param_types, func.return_type.clone());
        }
        // Check functions
        for func in &contract.functions {
            self.check_function(func);
        }
    }

    fn check_function(&mut self, func: &Function) {
        self.env.push_scope();
        for param in &func.params {
            self.env.define(&param.name, param.ty.clone());
        }
        for stmt in &func.body {
            self.check_statement(stmt, &func.return_type);
        }
        self.env.pop_scope();
    }

    fn check_statement(&mut self, stmt: &Statement, expected_return: &Option<NexType>) {
        match stmt {
            Statement::Let { name, ty, value, .. } => {
                let inferred = self.infer_type(value);
                if let Some(declared) = ty {
                    if let Some(ref inferred_ty) = inferred {
                        if !types_compatible(declared, inferred_ty) {
                            self.errors.push(NexlangError::TypeError(
                                format!("expected {:?}, found {:?}", declared, inferred_ty),
                            ));
                        }
                    }
                    self.env.define(name, declared.clone());
                } else if let Some(ref inferred_ty) = inferred {
                    self.env.define(name, inferred_ty.clone());
                } else {
                    self.errors.push(NexlangError::TypeError(
                        "expected known type, found unknown".into(),
                    ));
                }
            }
            Statement::Assign { target, value } => {
                let target_ty = self.infer_type(target);
                let value_ty = self.infer_type(value);
                if let (Some(t), Some(v)) = (&target_ty, &value_ty) {
                    if !types_compatible(t, v) {
                        self.errors.push(NexlangError::TypeError(
                            format!("expected {:?}, found {:?}", t, v),
                        ));
                    }
                }
            }
            Statement::Return(Some(expr)) => {
                let ret_ty = self.infer_type(expr);
                if let (Some(expected), Some(ref actual)) = (expected_return, &ret_ty) {
                    if !types_compatible(expected, actual) {
                        self.errors.push(NexlangError::TypeError(
                            format!("expected {:?}, found {:?}", expected, actual),
                        ));
                    }
                }
            }
            Statement::If { condition, then_body, else_body } => {
                let cond_ty = self.infer_type(condition);
                if let Some(ref ty) = cond_ty {
                    if !matches!(ty, NexType::Bool) {
                        self.errors.push(NexlangError::TypeError(
                            format!("expected Bool, found {:?}", ty),
                        ));
                    }
                }
                self.env.push_scope();
                for s in then_body { self.check_statement(s, expected_return); }
                self.env.pop_scope();
                if let Some(else_b) = else_body {
                    self.env.push_scope();
                    for s in else_b { self.check_statement(s, expected_return); }
                    self.env.pop_scope();
                }
            }
            Statement::While { condition, body } => {
                let cond_ty = self.infer_type(condition);
                if let Some(ref ty) = cond_ty {
                    if !matches!(ty, NexType::Bool) {
                        self.errors.push(NexlangError::TypeError(
                            format!("expected Bool, found {:?}", ty),
                        ));
                    }
                }
                self.env.push_scope();
                for s in body { self.check_statement(s, expected_return); }
                self.env.pop_scope();
            }
            Statement::Emit { event_name, args } => {
                if let Some(expected_types) = self.env.events.get(event_name).cloned() {
                    if args.len() != expected_types.len() {
                        self.errors.push(NexlangError::TypeError(
                            format!("expected {} args, found {} args", expected_types.len(), args.len()),
                        ));
                    }
                } else {
                    self.errors.push(NexlangError::UndefinedFunction(event_name.clone()));
                }
            }
            Statement::Require { condition, .. } => {
                let cond_ty = self.infer_type(condition);
                if let Some(ref ty) = cond_ty {
                    if !matches!(ty, NexType::Bool) {
                        self.errors.push(NexlangError::TypeError(
                            format!("expected Bool, found {:?}", ty),
                        ));
                    }
                }
            }
            Statement::Expression(_) | Statement::Return(None) => {}
            Statement::For { body, .. } => {
                self.env.push_scope();
                for s in body { self.check_statement(s, expected_return); }
                self.env.pop_scope();
            }
            Statement::Block(stmts) => {
                self.env.push_scope();
                for s in stmts { self.check_statement(s, expected_return); }
                self.env.pop_scope();
            }
        }
    }

    fn infer_type(&self, expr: &Expression) -> Option<NexType> {
        match expr {
            Expression::IntLiteral(_) => Some(NexType::U128),
            Expression::BoolLiteral(_) => Some(NexType::Bool),
            Expression::StringLiteral(_) => Some(NexType::String),
            Expression::AddressLiteral(_) => Some(NexType::Address),
            Expression::Identifier(name) => self.env.lookup(name).cloned(),
            Expression::MsgSender => Some(NexType::Address),
            Expression::MsgValue => Some(NexType::U128),
            Expression::BlockHeight => Some(NexType::U64),
            Expression::SelfRef => None,
            Expression::BinaryOp { left, op, .. } => {
                let left_ty = self.infer_type(left);
                match op {
                    BinaryOperator::Eq | BinaryOperator::Ne | BinaryOperator::Lt
                    | BinaryOperator::Gt | BinaryOperator::Le | BinaryOperator::Ge
                    | BinaryOperator::And | BinaryOperator::Or => Some(NexType::Bool),
                    _ => left_ty,
                }
            }
            Expression::UnaryOp { operand, op } => {
                match op {
                    UnaryOperator::Not => Some(NexType::Bool),
                    UnaryOperator::Neg => self.infer_type(operand),
                }
            }
            Expression::FunctionCall { name, .. } => {
                self.env.lookup_function(name).and_then(|(_, ret)| ret.clone())
            }
            Expression::MethodCall { .. } => None,
            Expression::FieldAccess { .. } => None,
            Expression::IndexAccess { object, .. } => {
                match self.infer_type(object)? {
                    NexType::Array(inner) => Some(*inner),
                    NexType::Map(_, val) => Some(*val),
                    _ => None,
                }
            }
            Expression::ArrayLiteral(elems) => {
                if let Some(first) = elems.first() {
                    let ty = self.infer_type(first)?;
                    Some(NexType::Array(Box::new(ty)))
                } else {
                    None
                }
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if two types are compatible.
pub fn types_compatible(a: &NexType, b: &NexType) -> bool {
    match (a, b) {
        (NexType::U8, NexType::U8) => true,
        (NexType::U16, NexType::U16) => true,
        (NexType::U32, NexType::U32) => true,
        (NexType::U64, NexType::U64) => true,
        (NexType::U128, NexType::U128) => true,
        (NexType::U256, NexType::U256) => true,
        (NexType::I64, NexType::I64) => true,
        (NexType::I128, NexType::I128) => true,
        (NexType::Bool, NexType::Bool) => true,
        (NexType::String, NexType::String) => true,
        (NexType::Address, NexType::Address) => true,
        (NexType::Bytes, NexType::Bytes) => true,
        (NexType::Map(k1, v1), NexType::Map(k2, v2)) => {
            types_compatible(k1, k2) && types_compatible(v1, v2)
        }
        (NexType::Array(a), NexType::Array(b)) => types_compatible(a, b),
        (NexType::Custom(a), NexType::Custom(b)) => a == b,
        // Integer literal widening
        (NexType::U128, NexType::U8 | NexType::U16 | NexType::U32 | NexType::U64 | NexType::U256) => true,
        (NexType::U8 | NexType::U16 | NexType::U32 | NexType::U64 | NexType::U256, NexType::U128) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn check(source: &str) -> Vec<NexlangError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let prog = parser.parse_program().unwrap();
        let mut checker = TypeChecker::new();
        checker.check_program(&prog)
    }

    #[test]
    fn test_no_errors_for_valid() {
        let errors = check("contract Token {\n  pub fn get() -> u128 {\n    return 42;\n  }\n}");
        assert!(errors.is_empty(), "Expected 0 errors, got: {:?}", errors);
    }

    #[test]
    fn test_type_error_condition() {
        let errors = check("contract T {\n  pub fn f() {\n    if 42 { return; }\n  }\n}");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_variable_type_tracking() {
        let errors = check("contract T {\n  pub fn f() {\n    let x: u128 = 10;\n    return;\n  }\n}");
        assert!(errors.is_empty(), "Expected 0 errors, got: {:?}", errors);
    }

    #[test]
    fn test_type_env_scopes() {
        let mut env = TypeEnv::new();
        env.define("x", NexType::U64);
        assert!(env.lookup("x").is_some());
        env.push_scope();
        env.define("y", NexType::Bool);
        assert!(env.lookup("x").is_some());
        assert!(env.lookup("y").is_some());
        env.pop_scope();
        assert!(env.lookup("y").is_none());
    }

    #[test]
    fn test_types_compatible() {
        assert!(types_compatible(&NexType::U128, &NexType::U128));
        assert!(types_compatible(&NexType::Bool, &NexType::Bool));
        assert!(!types_compatible(&NexType::Bool, &NexType::U128));
    }
}
