//! NEXLANG Parser - Parses tokens into AST.

use crate::ast::*;
use crate::lexer::{Token, TokenKind};
use crate::errors::NexlangError;

/// NEXLANG Parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Create a new parser from tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Parse a complete program.
    pub fn parse_program(&mut self) -> Result<Program, NexlangError> {
        let mut contracts = Vec::new();
        self.skip_newlines();
        while !self.is_at_end() {
            contracts.push(self.parse_contract()?);
            self.skip_newlines();
        }
        Ok(Program { contracts })
    }

    /// Parse a contract definition.
    fn parse_contract(&mut self) -> Result<Contract, NexlangError> {
        // Collect annotations
        let annotations = self.parse_annotations()?;
        self.expect(TokenKind::Contract)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::LBrace)?;
        self.skip_newlines();

        let mut state_vars = Vec::new();
        let mut functions = Vec::new();
        let mut events = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(TokenKind::RBrace) {
                break;
            }

            // Peek for annotations
            let item_annots = self.parse_annotations()?;

            if self.check(TokenKind::Event) {
                events.push(self.parse_event()?);
            } else if self.check(TokenKind::Fn) || self.check(TokenKind::Pub) || self.check(TokenKind::Priv) {
                functions.push(self.parse_function(item_annots)?);
            } else if self.check(TokenKind::Let) {
                state_vars.push(self.parse_state_var()?);
            } else {
                let tok = self.peek();
                return Err(NexlangError::ParseError {
                    line: tok.map(|t| t.line).unwrap_or(0),
                    message: format!("Unexpected token: {:?}", tok.map(|t| &t.kind)),
                });
            }
            self.skip_newlines();
        }
        self.expect(TokenKind::RBrace)?;

        Ok(Contract {
            name,
            state_vars,
            functions,
            events,
            annotations,
        })
    }

    fn parse_annotations(&mut self) -> Result<Vec<Annotation>, NexlangError> {
        let mut annots = Vec::new();
        self.skip_newlines();
        while self.check(TokenKind::At) {
            self.advance();
            let name = self.expect_identifier()?;
            let mut args = Vec::new();
            if self.check(TokenKind::LParen) {
                self.advance();
                while !self.check(TokenKind::RParen) && !self.is_at_end() {
                    args.push(self.expect_identifier()?);
                    if self.check(TokenKind::Comma) { self.advance(); }
                }
                self.expect(TokenKind::RParen)?;
            }
            annots.push(Annotation { name, args });
            self.skip_newlines();
        }
        Ok(annots)
    }

    fn parse_event(&mut self) -> Result<Event, NexlangError> {
        self.expect(TokenKind::Event)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::LParen)?;
        let mut fields = Vec::new();
        while !self.check(TokenKind::RParen) && !self.is_at_end() {
            let fname = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let ty = self.parse_type()?;
            fields.push(Param { name: fname, ty });
            if self.check(TokenKind::Comma) { self.advance(); }
        }
        self.expect(TokenKind::RParen)?;
        self.skip_newlines();
        Ok(Event { name, fields })
    }

    fn parse_function(&mut self, annotations: Vec<Annotation>) -> Result<Function, NexlangError> {
        let visibility = self.parse_visibility();
        self.expect(TokenKind::Fn)?;
        let name = self.expect_identifier()?;
        let is_constructor = name == "init" || name == "constructor";

        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        while !self.check(TokenKind::RParen) && !self.is_at_end() {
            let pname = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let ty = self.parse_type()?;
            params.push(Param { name: pname, ty });
            if self.check(TokenKind::Comma) { self.advance(); }
        }
        self.expect(TokenKind::RParen)?;

        let return_type = if self.check(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let is_view = annotations.iter().any(|a| a.name == "view");

        let body = self.parse_block()?;

        Ok(Function {
            name,
            params,
            return_type,
            body,
            visibility,
            annotations,
            is_constructor,
            is_view,
        })
    }

    fn parse_state_var(&mut self) -> Result<StateVar, NexlangError> {
        self.expect(TokenKind::Let)?;
        let mutable = if self.check(TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;

        let default_value = if self.check(TokenKind::Assign) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume_semicolon();

        Ok(StateVar {
            name,
            ty,
            visibility: Visibility::Private,
            mutable,
            default_value,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, NexlangError> {
        self.expect(TokenKind::LBrace)?;
        self.skip_newlines();
        let mut stmts = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_statement()?);
            self.skip_newlines();
        }
        self.expect(TokenKind::RBrace)?;
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Statement, NexlangError> {
        self.skip_newlines();
        if self.check(TokenKind::Let) {
            self.parse_let_statement()
        } else if self.check(TokenKind::Return) {
            self.parse_return_statement()
        } else if self.check(TokenKind::If) {
            self.parse_if_statement()
        } else if self.check(TokenKind::While) {
            self.parse_while_statement()
        } else if self.check(TokenKind::Emit) {
            self.parse_emit_statement()
        } else if self.check(TokenKind::Require) {
            self.parse_require_statement()
        } else {
            let expr = self.parse_expression()?;
            if self.check(TokenKind::Assign) {
                self.advance();
                let value = self.parse_expression()?;
                self.consume_semicolon();
                Ok(Statement::Assign { target: expr, value })
            } else {
                self.consume_semicolon();
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, NexlangError> {
        self.expect(TokenKind::Let)?;
        let mutable = if self.check(TokenKind::Mut) { self.advance(); true } else { false };
        let name = self.expect_identifier()?;
        let ty = if self.check(TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(TokenKind::Assign)?;
        let value = self.parse_expression()?;
        self.consume_semicolon();
        Ok(Statement::Let { name, ty, value, mutable })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, NexlangError> {
        self.expect(TokenKind::Return)?;
        if self.check(TokenKind::Semicolon) || self.check(TokenKind::Newline) || self.check(TokenKind::RBrace) {
            self.consume_semicolon();
            return Ok(Statement::Return(None));
        }
        let expr = self.parse_expression()?;
        self.consume_semicolon();
        Ok(Statement::Return(Some(expr)))
    }

    fn parse_if_statement(&mut self) -> Result<Statement, NexlangError> {
        self.expect(TokenKind::If)?;
        let condition = self.parse_expression()?;
        let then_body = self.parse_block()?;
        let else_body = if self.check(TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Statement::If { condition, then_body, else_body })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, NexlangError> {
        self.expect(TokenKind::While)?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Statement::While { condition, body })
    }

    fn parse_emit_statement(&mut self) -> Result<Statement, NexlangError> {
        self.expect(TokenKind::Emit)?;
        let event_name = self.expect_identifier()?;
        self.expect(TokenKind::LParen)?;
        let mut args = Vec::new();
        while !self.check(TokenKind::RParen) && !self.is_at_end() {
            args.push(self.parse_expression()?);
            if self.check(TokenKind::Comma) { self.advance(); }
        }
        self.expect(TokenKind::RParen)?;
        self.consume_semicolon();
        Ok(Statement::Emit { event_name, args })
    }

    fn parse_require_statement(&mut self) -> Result<Statement, NexlangError> {
        self.expect(TokenKind::Require)?;
        self.expect(TokenKind::LParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::Comma)?;
        let message = match &self.peek().map(|t| t.kind.clone()) {
            Some(TokenKind::StringLiteral(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => {
                let tok = self.peek();
                return Err(NexlangError::ParseError {
                    line: tok.map(|t| t.line).unwrap_or(0),
                    message: "Expected string literal in require".into(),
                });
            }
        };
        self.expect(TokenKind::RParen)?;
        self.consume_semicolon();
        Ok(Statement::Require { condition, message })
    }

    fn parse_expression(&mut self) -> Result<Expression, NexlangError> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Expression, NexlangError> {
        let mut left = self.parse_and_expr()?;
        while self.check(TokenKind::Or) {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expression, NexlangError> {
        let mut left = self.parse_comparison()?;
        while self.check(TokenKind::And) {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, NexlangError> {
        let mut left = self.parse_additive()?;
        while let Some(op) = self.match_comparison_op() {
            let right = self.parse_additive()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expression, NexlangError> {
        let mut left = self.parse_multiplicative()?;
        while self.check(TokenKind::Plus) || self.check(TokenKind::Minus) {
            let op = if self.check(TokenKind::Plus) { BinaryOperator::Add } else { BinaryOperator::Sub };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, NexlangError> {
        let mut left = self.parse_unary()?;
        while self.check(TokenKind::Star) || self.check(TokenKind::Slash) || self.check(TokenKind::Percent) {
            let op = if self.check(TokenKind::Star) { BinaryOperator::Mul }
                else if self.check(TokenKind::Slash) { BinaryOperator::Div }
                else { BinaryOperator::Mod };
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, NexlangError> {
        if self.check(TokenKind::Not) {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expression::UnaryOp { op: UnaryOperator::Not, operand: Box::new(operand) });
        }
        if self.check(TokenKind::Minus) {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expression::UnaryOp { op: UnaryOperator::Neg, operand: Box::new(operand) });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expression, NexlangError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(TokenKind::Dot) {
                self.advance();
                let field = self.expect_identifier()?;
                if self.check(TokenKind::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.check(TokenKind::RParen) && !self.is_at_end() {
                        args.push(self.parse_expression()?);
                        if self.check(TokenKind::Comma) { self.advance(); }
                    }
                    self.expect(TokenKind::RParen)?;
                    expr = Expression::MethodCall {
                        object: Box::new(expr),
                        method: field,
                        args,
                    };
                } else {
                    expr = Expression::FieldAccess {
                        object: Box::new(expr),
                        field,
                    };
                }
            } else if self.check(TokenKind::LBracket) {
                self.advance();
                let index = self.parse_expression()?;
                self.expect(TokenKind::RBracket)?;
                expr = Expression::IndexAccess {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.check(TokenKind::LParen) {
                // Function call on identifier
                if let Expression::Identifier(name) = &expr {
                    let name = name.clone();
                    self.advance();
                    let mut args = Vec::new();
                    while !self.check(TokenKind::RParen) && !self.is_at_end() {
                        args.push(self.parse_expression()?);
                        if self.check(TokenKind::Comma) { self.advance(); }
                    }
                    self.expect(TokenKind::RParen)?;
                    expr = Expression::FunctionCall { name, args };
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, NexlangError> {
        let tok = self.peek().cloned();
        match tok.as_ref().map(|t| &t.kind) {
            Some(TokenKind::IntLiteral(n)) => {
                let n = *n;
                self.advance();
                Ok(Expression::IntLiteral(n))
            }
            Some(TokenKind::StringLiteral(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::StringLiteral(s))
            }
            Some(TokenKind::BoolLiteral(b)) => {
                let b = *b;
                self.advance();
                Ok(Expression::BoolLiteral(b))
            }
            Some(TokenKind::SelfKw) => {
                self.advance();
                Ok(Expression::SelfRef)
            }
            Some(TokenKind::MsgSender) => {
                self.advance();
                Ok(Expression::MsgSender)
            }
            Some(TokenKind::MsgValue) => {
                self.advance();
                Ok(Expression::MsgValue)
            }
            Some(TokenKind::BlockHeight) => {
                self.advance();
                Ok(Expression::BlockHeight)
            }
            Some(TokenKind::Identifier(_)) => {
                let name = self.expect_identifier()?;
                Ok(Expression::Identifier(name))
            }
            Some(TokenKind::LParen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            Some(TokenKind::LBracket) => {
                self.advance();
                let mut elems = Vec::new();
                while !self.check(TokenKind::RBracket) && !self.is_at_end() {
                    elems.push(self.parse_expression()?);
                    if self.check(TokenKind::Comma) { self.advance(); }
                }
                self.expect(TokenKind::RBracket)?;
                Ok(Expression::ArrayLiteral(elems))
            }
            _ => {
                let line = tok.as_ref().map(|t| t.line).unwrap_or(0);
                Err(NexlangError::ParseError {
                    line,
                    message: format!("Unexpected token in expression: {:?}", tok.map(|t| t.kind)),
                })
            }
        }
    }

    fn parse_type(&mut self) -> Result<NexType, NexlangError> {
        let tok = self.peek().cloned();
        match tok.as_ref().map(|t| &t.kind) {
            Some(TokenKind::U8) => { self.advance(); Ok(NexType::U8) }
            Some(TokenKind::U16) => { self.advance(); Ok(NexType::U16) }
            Some(TokenKind::U32) => { self.advance(); Ok(NexType::U32) }
            Some(TokenKind::U64) => { self.advance(); Ok(NexType::U64) }
            Some(TokenKind::U128) => { self.advance(); Ok(NexType::U128) }
            Some(TokenKind::U256) => { self.advance(); Ok(NexType::U256) }
            Some(TokenKind::I64) => { self.advance(); Ok(NexType::I64) }
            Some(TokenKind::I128) => { self.advance(); Ok(NexType::I128) }
            Some(TokenKind::Bool) => { self.advance(); Ok(NexType::Bool) }
            Some(TokenKind::StringType) => { self.advance(); Ok(NexType::String) }
            Some(TokenKind::Address) => { self.advance(); Ok(NexType::Address) }
            Some(TokenKind::Bytes) => { self.advance(); Ok(NexType::Bytes) }
            Some(TokenKind::Map) => {
                self.advance();
                self.expect(TokenKind::Lt)?;
                let key = self.parse_type()?;
                self.expect(TokenKind::Comma)?;
                let val = self.parse_type()?;
                self.expect(TokenKind::Gt)?;
                Ok(NexType::Map(Box::new(key), Box::new(val)))
            }
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                Ok(NexType::Custom(name))
            }
            _ => {
                let line = tok.as_ref().map(|t| t.line).unwrap_or(0);
                Err(NexlangError::ParseError {
                    line,
                    message: format!("Expected type, got {:?}", tok.map(|t| t.kind)),
                })
            }
        }
    }

    fn parse_visibility(&mut self) -> Visibility {
        if self.check(TokenKind::Pub) { self.advance(); Visibility::Public }
        else if self.check(TokenKind::Priv) { self.advance(); Visibility::Private }
        else if self.check(TokenKind::Internal) { self.advance(); Visibility::Internal }
        else { Visibility::Private }
    }

    fn match_comparison_op(&mut self) -> Option<BinaryOperator> {
        if self.check(TokenKind::Eq) { self.advance(); Some(BinaryOperator::Eq) }
        else if self.check(TokenKind::Ne) { self.advance(); Some(BinaryOperator::Ne) }
        else if self.check(TokenKind::Lt) { self.advance(); Some(BinaryOperator::Lt) }
        else if self.check(TokenKind::Gt) { self.advance(); Some(BinaryOperator::Gt) }
        else if self.check(TokenKind::Le) { self.advance(); Some(BinaryOperator::Le) }
        else if self.check(TokenKind::Ge) { self.advance(); Some(BinaryOperator::Ge) }
        else { None }
    }

    // Helpers

    fn peek(&self) -> Option<&Token> {
        self.skip_newlines_peek()
    }

    fn skip_newlines_peek(&self) -> Option<&Token> {
        let mut i = self.pos;
        while i < self.tokens.len() {
            if !matches!(self.tokens[i].kind, TokenKind::Newline) {
                return Some(&self.tokens[i]);
            }
            i += 1;
        }
        None
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().map(|t| std::mem::discriminant(&t.kind) == std::mem::discriminant(&kind)).unwrap_or(false)
    }

    fn advance(&mut self) -> Option<&Token> {
        self.skip_newlines();
        if self.pos < self.tokens.len() {
            let tok = &self.tokens[self.pos];
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn skip_newlines(&mut self) {
        while self.pos < self.tokens.len() && matches!(self.tokens[self.pos].kind, TokenKind::Newline) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), NexlangError> {
        if self.check(kind.clone()) {
            self.advance();
            Ok(())
        } else {
            let tok = self.peek().cloned();
            Err(NexlangError::ParseError {
                line: tok.as_ref().map(|t| t.line).unwrap_or(0),
                message: format!("Expected {:?}, got {:?}", kind, tok.map(|t| t.kind)),
            })
        }
    }

    fn expect_identifier(&mut self) -> Result<String, NexlangError> {
        let tok = self.peek().cloned();
        match tok.as_ref().map(|t| &t.kind) {
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(NexlangError::ParseError {
                line: tok.as_ref().map(|t| t.line).unwrap_or(0),
                message: format!("Expected identifier, got {:?}", tok.map(|t| t.kind)),
            }),
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().map(|t| matches!(t.kind, TokenKind::Eof)).unwrap_or(true)
    }

    fn consume_semicolon(&mut self) {
        if self.check(TokenKind::Semicolon) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> Result<Program, NexlangError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse_program()
    }

    #[test]
    fn test_parse_empty_contract() {
        let prog = parse("contract Token { }").unwrap();
        assert_eq!(prog.contracts.len(), 1);
        assert_eq!(prog.contracts[0].name, "Token");
    }

    #[test]
    fn test_parse_state_var() {
        let prog = parse("contract Token {\n  let mut balance: u128 = 0;\n}").unwrap();
        assert_eq!(prog.contracts[0].state_vars.len(), 1);
        assert_eq!(prog.contracts[0].state_vars[0].name, "balance");
        assert!(prog.contracts[0].state_vars[0].mutable);
    }

    #[test]
    fn test_parse_function() {
        let prog = parse("contract Token {\n  pub fn transfer(to: Address, amount: u128) {\n    return;\n  }\n}").unwrap();
        assert_eq!(prog.contracts[0].functions.len(), 1);
        assert_eq!(prog.contracts[0].functions[0].name, "transfer");
        assert_eq!(prog.contracts[0].functions[0].params.len(), 2);
    }

    #[test]
    fn test_parse_annotation() {
        let prog = parse("contract Token {\n  @nonreentrant\n  pub fn withdraw() {\n    return;\n  }\n}").unwrap();
        assert_eq!(prog.contracts[0].functions[0].annotations.len(), 1);
        assert_eq!(prog.contracts[0].functions[0].annotations[0].name, "nonreentrant");
    }

    #[test]
    fn test_parse_event() {
        let prog = parse("contract Token {\n  event Transfer(from: Address, to: Address, amount: u128)\n}").unwrap();
        assert_eq!(prog.contracts[0].events.len(), 1);
        assert_eq!(prog.contracts[0].events[0].name, "Transfer");
    }
}
