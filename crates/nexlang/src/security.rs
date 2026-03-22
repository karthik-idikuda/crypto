//! NEXLANG Security Analyzer - detects vulnerabilities in contracts.

use std::collections::HashSet;
use crate::ast::*;

/// Security finding with severity.
#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub severity: Severity,
    pub category: String,
    pub message: String,
    pub function_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Security analyzer for NEXLANG contracts.
pub struct SecurityAnalyzer {
    findings: Vec<SecurityFinding>,
}

impl SecurityAnalyzer {
    pub fn new() -> Self {
        SecurityAnalyzer { findings: Vec::new() }
    }

    /// Analyze a program for security issues.
    pub fn analyze(&mut self, program: &Program) -> Vec<SecurityFinding> {
        for contract in &program.contracts {
            self.analyze_contract(contract);
        }
        std::mem::take(&mut self.findings)
    }

    fn analyze_contract(&mut self, contract: &Contract) {
        for func in &contract.functions {
            self.check_reentrancy(func);
            self.check_access_control(func);
            self.check_overflow_patterns(func);
            self.check_unchecked_return(func);
            self.check_self_destruct(func);
            self.check_tx_origin(func);
        }
    }

    /// Detect potential reentrancy: state changes after external calls.
    fn check_reentrancy(&mut self, func: &Function) {
        let has_guard = func.annotations.iter().any(|a| a.name == "nonreentrant");
        if has_guard {
            return;
        }

        let mut found_external_call = false;
        let mut state_change_after_call = false;

        for stmt in &func.body {
            self.walk_statement(stmt, &mut found_external_call, &mut state_change_after_call);
        }

        if state_change_after_call {
            self.findings.push(SecurityFinding {
                severity: Severity::Critical,
                category: "Reentrancy".into(),
                message: "State modification after external call without @nonreentrant guard".into(),
                function_name: func.name.clone(),
            });
        }
    }

    fn walk_statement(&self, stmt: &Statement, found_call: &mut bool, state_after: &mut bool) {
        match stmt {
            Statement::Expression(expr) | Statement::Return(Some(expr)) => {
                if self.has_external_call(expr) {
                    *found_call = true;
                }
            }
            Statement::Assign { value, .. } => {
                if *found_call {
                    *state_after = true;
                }
                if self.has_external_call(value) {
                    *found_call = true;
                }
            }
            Statement::Let { value, .. } => {
                if self.has_external_call(value) {
                    *found_call = true;
                }
            }
            Statement::If { then_body, else_body, .. } => {
                for s in then_body { self.walk_statement(s, found_call, state_after); }
                if let Some(eb) = else_body {
                    for s in eb { self.walk_statement(s, found_call, state_after); }
                }
            }
            Statement::While { body, .. } | Statement::For { body, .. } => {
                for s in body { self.walk_statement(s, found_call, state_after); }
            }
            _ => {}
        }
    }

    fn has_external_call(&self, expr: &Expression) -> bool {
        match expr {
            Expression::MethodCall { method, .. } => {
                matches!(method.as_str(), "transfer" | "call" | "send" | "delegatecall")
            }
            Expression::FunctionCall { name, .. } => {
                matches!(name.as_str(), "transfer" | "call" | "send")
            }
            Expression::BinaryOp { left, right, .. } => {
                self.has_external_call(left) || self.has_external_call(right)
            }
            _ => false,
        }
    }

    /// Check that public state-changing functions have access control.
    fn check_access_control(&mut self, func: &Function) {
        if func.visibility != Visibility::Public {
            return;
        }
        if func.is_view || func.is_constructor {
            return;
        }

        let has_require = func.body.iter().any(|s| matches!(s, Statement::Require { .. }));
        let has_access_annotation = func.annotations.iter().any(|a| {
            matches!(a.name.as_str(), "onlyOwner" | "onlyAdmin" | "restricted" | "access")
        });

        let modifies_state = func.body.iter().any(|s| matches!(s, Statement::Assign { .. }));

        if modifies_state && !has_require && !has_access_annotation {
            self.findings.push(SecurityFinding {
                severity: Severity::High,
                category: "AccessControl".into(),
                message: "Public state-changing function lacks access control".into(),
                function_name: func.name.clone(),
            });
        }
    }

    /// Detect integer overflow patterns.
    fn check_overflow_patterns(&mut self, func: &Function) {
        let mut has_unchecked_math = false;
        self.scan_overflow_in_stmts(&func.body, &mut has_unchecked_math);

        let has_safe_math = func.annotations.iter().any(|a| a.name == "checked");
        if has_unchecked_math && !has_safe_math {
            self.findings.push(SecurityFinding {
                severity: Severity::Medium,
                category: "Overflow".into(),
                message: "Arithmetic operations without overflow protection".into(),
                function_name: func.name.clone(),
            });
        }
    }

    fn scan_overflow_in_stmts(&self, stmts: &[Statement], found: &mut bool) {
        for stmt in stmts {
            match stmt {
                Statement::Let { value, .. } | Statement::Expression(value) => {
                    if self.has_arithmetic(value) { *found = true; }
                }
                Statement::Assign { value, .. } => {
                    if self.has_arithmetic(value) { *found = true; }
                }
                _ => {}
            }
        }
    }

    fn has_arithmetic(&self, expr: &Expression) -> bool {
        match expr {
            Expression::BinaryOp { op, left, right, .. } => {
                matches!(op, BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul)
                    || self.has_arithmetic(left)
                    || self.has_arithmetic(right)
            }
            _ => false,
        }
    }

    fn check_unchecked_return(&mut self, func: &Function) {
        let mut external_calls_without_check = false;
        for stmt in &func.body {
            if let Statement::Expression(expr) = stmt {
                if self.has_external_call(expr) {
                    external_calls_without_check = true;
                }
            }
        }
        if external_calls_without_check {
            self.findings.push(SecurityFinding {
                severity: Severity::Medium,
                category: "UncheckedReturn".into(),
                message: "External call return value not checked".into(),
                function_name: func.name.clone(),
            });
        }
    }

    fn check_self_destruct(&mut self, func: &Function) {
        for stmt in &func.body {
            if let Statement::Expression(Expression::FunctionCall { name, .. }) = stmt {
                if name == "selfdestruct" || name == "self_destruct" {
                    self.findings.push(SecurityFinding {
                        severity: Severity::Critical,
                        category: "SelfDestruct".into(),
                        message: "Contract contains self-destruct capability".into(),
                        function_name: func.name.clone(),
                    });
                }
            }
        }
    }

    fn check_tx_origin(&mut self, func: &Function) {
        let mut found = false;
        self.scan_tx_origin_stmts(&func.body, &mut found);
        if found {
            self.findings.push(SecurityFinding {
                severity: Severity::High,
                category: "TxOrigin".into(),
                message: "Use of tx_origin for authorization is unsafe".into(),
                function_name: func.name.clone(),
            });
        }
    }

    fn scan_tx_origin_stmts(&self, stmts: &[Statement], found: &mut bool) {
        for stmt in stmts {
            match stmt {
                Statement::Require { condition, .. } => {
                    if self.expr_uses_tx_origin(condition) { *found = true; }
                }
                Statement::If { condition, .. } => {
                    if self.expr_uses_tx_origin(condition) { *found = true; }
                }
                _ => {}
            }
        }
    }

    fn expr_uses_tx_origin(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(name) => name == "tx_origin",
            Expression::FieldAccess { field, .. } => field == "origin",
            Expression::BinaryOp { left, right, .. } => {
                self.expr_uses_tx_origin(left) || self.expr_uses_tx_origin(right)
            }
            _ => false,
        }
    }
}

impl Default for SecurityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Perform a full security audit on a program.
pub fn audit_program(program: &Program) -> Vec<SecurityFinding> {
    let mut analyzer = SecurityAnalyzer::new();
    analyzer.analyze(program)
}

/// Collect all unique identifiers in a set of statements (for dead code hints).
pub fn collect_referenced_identifiers(stmts: &[Statement]) -> HashSet<String> {
    let mut ids = HashSet::new();
    for stmt in stmts {
        collect_ids_stmt(stmt, &mut ids);
    }
    ids
}

fn collect_ids_stmt(stmt: &Statement, ids: &mut HashSet<String>) {
    match stmt {
        Statement::Let { value, name, .. } => {
            ids.insert(name.clone());
            collect_ids_expr(value, ids);
        }
        Statement::Assign { target, value } => {
            collect_ids_expr(target, ids);
            collect_ids_expr(value, ids);
        }
        Statement::Expression(e) | Statement::Return(Some(e)) => collect_ids_expr(e, ids),
        Statement::If { condition, then_body, else_body } => {
            collect_ids_expr(condition, ids);
            for s in then_body { collect_ids_stmt(s, ids); }
            if let Some(eb) = else_body { for s in eb { collect_ids_stmt(s, ids); } }
        }
        Statement::While { condition, body } => {
            collect_ids_expr(condition, ids);
            for s in body { collect_ids_stmt(s, ids); }
        }
        Statement::Emit { args, .. } => {
            for a in args { collect_ids_expr(a, ids); }
        }
        Statement::Require { condition, .. } => collect_ids_expr(condition, ids),
        Statement::For { body, .. } => {
            for s in body { collect_ids_stmt(s, ids); }
        }
        _ => {}
    }
}

fn collect_ids_expr(expr: &Expression, ids: &mut HashSet<String>) {
    match expr {
        Expression::Identifier(name) => { ids.insert(name.clone()); }
        Expression::BinaryOp { left, right, .. } => {
            collect_ids_expr(left, ids);
            collect_ids_expr(right, ids);
        }
        Expression::UnaryOp { operand, .. } => collect_ids_expr(operand, ids),
        Expression::FunctionCall { args, .. } => {
            for a in args { collect_ids_expr(a, ids); }
        }
        Expression::MethodCall { object, args, .. } => {
            collect_ids_expr(object, ids);
            for a in args { collect_ids_expr(a, ids); }
        }
        Expression::FieldAccess { object, .. } => collect_ids_expr(object, ids),
        Expression::IndexAccess { object, index } => {
            collect_ids_expr(object, ids);
            collect_ids_expr(index, ids);
        }
        Expression::ArrayLiteral(elems) => {
            for e in elems { collect_ids_expr(e, ids); }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn analyze(source: &str) -> Vec<SecurityFinding> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let prog = parser.parse_program().unwrap();
        audit_program(&prog)
    }

    #[test]
    fn test_no_findings_for_safe_contract() {
        let findings = analyze("contract Safe {\n  @view\n  pub fn get() -> u128 {\n    return 42;\n  }\n}");
        let critical = findings.iter().filter(|f| f.severity == Severity::Critical).count();
        assert_eq!(critical, 0);
    }

    #[test]
    fn test_detect_access_control_missing() {
        let findings = analyze("contract Vuln {\n  let mut owner: Address = msg_sender;\n  pub fn set_owner(new_owner: Address) {\n    owner = new_owner;\n  }\n}");
        let access = findings.iter().any(|f| f.category == "AccessControl");
        assert!(access, "Should detect missing access control");
    }

    #[test]
    fn test_nonreentrant_suppresses() {
        let findings = analyze("contract Safe {\n  @nonreentrant\n  pub fn withdraw() {\n    return;\n  }\n}");
        let reentrancy = findings.iter().any(|f| f.category == "Reentrancy");
        assert!(!reentrancy);
    }

    #[test]
    fn test_severity_levels() {
        assert_ne!(Severity::Critical, Severity::Low);
        assert_eq!(Severity::High, Severity::High);
    }

    #[test]
    fn test_collect_identifiers() {
        let stmts = vec![
            Statement::Let {
                name: "x".into(),
                ty: Some(NexType::U128),
                value: Expression::Identifier("y".into()),
                mutable: false,
            },
        ];
        let ids = collect_referenced_identifiers(&stmts);
        assert!(ids.contains("x"));
        assert!(ids.contains("y"));
    }
}
