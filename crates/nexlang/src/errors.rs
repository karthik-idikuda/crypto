use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum NexlangError {
    #[error("Lexer error at line {line}, col {col}: {message}")]
    LexerError { line: usize, col: usize, message: String },
    #[error("Parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },
    #[error("Type error: {0}")]
    TypeError(String),
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    #[error("Code generation error: {0}")]
    CodeGenError(String),
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("Undefined function: {0}")]
    UndefinedFunction(String),
    #[error("Duplicate definition: {0}")]
    DuplicateDefinition(String),
}
