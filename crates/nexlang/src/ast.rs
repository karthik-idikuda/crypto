//! NEXLANG Abstract Syntax Tree.

use serde::{Serialize, Deserialize};

/// A complete NEXLANG program (smart contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub contracts: Vec<Contract>,
}

/// A smart contract definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub name: String,
    pub state_vars: Vec<StateVar>,
    pub functions: Vec<Function>,
    pub events: Vec<Event>,
    pub annotations: Vec<Annotation>,
}

/// A state variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVar {
    pub name: String,
    pub ty: NexType,
    pub visibility: Visibility,
    pub mutable: bool,
    pub default_value: Option<Expression>,
}

/// A function definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<NexType>,
    pub body: Vec<Statement>,
    pub visibility: Visibility,
    pub annotations: Vec<Annotation>,
    pub is_constructor: bool,
    pub is_view: bool,
}

/// A function parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub ty: NexType,
}

/// An event definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub fields: Vec<Param>,
}

/// An annotation (e.g., @nonreentrant, @onlyOwner).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub name: String,
    pub args: Vec<String>,
}

/// Visibility specifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Internal,
}

/// NEXLANG types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NexType {
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    I64,
    I128,
    Bool,
    String,
    Address,
    Bytes,
    Array(Box<NexType>),
    Map(Box<NexType>, Box<NexType>),
    Option(Box<NexType>),
    Custom(String),
    Unit,
}

/// Statements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Let {
        name: String,
        ty: Option<NexType>,
        value: Expression,
        mutable: bool,
    },
    Assign {
        target: Expression,
        value: Expression,
    },
    Return(Option<Expression>),
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    For {
        var: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    Expression(Expression),
    Emit {
        event_name: String,
        args: Vec<Expression>,
    },
    Require {
        condition: Expression,
        message: String,
    },
    Block(Vec<Statement>),
}

/// Expressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    IntLiteral(u128),
    StringLiteral(String),
    BoolLiteral(bool),
    AddressLiteral(String),
    Identifier(String),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },
    IndexAccess {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    ArrayLiteral(Vec<Expression>),
    SelfRef,
    MsgSender,
    MsgValue,
    BlockHeight,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
    BitAnd, BitOr, BitXor,
    Shl, Shr,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,
    Neg,
}
