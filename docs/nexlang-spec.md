# NEXLANG Specification v1.0

## Overview

NEXLANG is the native smart contract programming language for the NEXARA blockchain. It compiles to NXVM bytecode and provides built-in security guarantees including reentrancy protection, overflow checking, and post-quantum signature verification.

## Syntax

### Contract Declaration

```nexlang
@pqc_secure
contract MyContract {
    state owner: Address;
    state counter: U256;

    event CounterUpdated(value: U256);

    @constructor
    fn init() {
        self.owner = msg.sender;
        self.counter = 0;
    }

    @public
    fn increment() {
        self.counter = self.counter + 1;
        emit CounterUpdated(self.counter);
    }
}
```

### Types

| Type | Description |
|------|-------------|
| `U8` | Unsigned 8-bit integer |
| `U16` | Unsigned 16-bit integer |
| `U32` | Unsigned 32-bit integer |
| `U64` | Unsigned 64-bit integer |
| `U128` | Unsigned 128-bit integer |
| `U256` | Unsigned 256-bit integer |
| `I64` | Signed 64-bit integer |
| `I128` | Signed 128-bit integer |
| `Bool` | Boolean (true/false) |
| `String` | UTF-8 string |
| `Address` | 20-byte account address |
| `Bytes` | Dynamic byte array |
| `Map<K, V>` | Hash map |
| `Array<T>` | Dynamic array |
| `Option<T>` | Optional value |
| Custom | User-defined struct types |

### Annotations

| Annotation | Description |
|------------|-------------|
| `@pqc_secure` | Enables PQC verification on contract |
| `@constructor` | Marks the initialization function |
| `@public` | Function callable externally |
| `@view` | Read-only function (no state changes) |
| `@payable` | Function accepts NXR value |
| `@only_owner` | Restricted to contract owner |
| `@nonreentrant` | Explicit reentrancy guard |

### Statements

```nexlang
// Variable declaration
let x: U256 = 42;

// Assignment
x = x + 1;

// Conditional
if x > 10 {
    // ...
} else {
    // ...
}

// Loop
while x > 0 {
    x = x - 1;
}

// For loop
for i in 0..100 {
    // ...
}

// Requirements (assertions)
require(amount > 0, "Amount must be positive");

// Event emission
emit Transfer(from, to, amount);

// Return
return value;
```

### Expressions

- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Logical: `&&`, `||`, `!`
- Bitwise: `&`, `|`, `^`, `<<`, `>>`
- Member access: `self.field`, `obj.method()`
- Index access: `map[key]`, `array[index]`

### Built-in Globals

| Global | Type | Description |
|--------|------|-------------|
| `msg.sender` | `Address` | Transaction sender |
| `msg.value` | `U256` | NXR amount sent |
| `block.height` | `U256` | Current block number |
| `block.timestamp` | `U256` | Block timestamp |
| `shard.id` | `U256` | Current shard ID |

## Security Model

### Static Analysis

Before deployment, all contracts undergo static analysis checking for:

1. **Reentrancy**: State changes after external calls
2. **Access Control**: Missing owner checks on privileged functions
3. **Overflow**: Arithmetic without bounds checking
4. **Unchecked Returns**: Ignored function return values
5. **Self-Destruct**: Unrestricted contract destruction
6. **tx.origin Usage**: Phishing attack vector

### Severity Levels

- **Critical**: Must fix before deployment
- **High**: Strongly recommended fix
- **Medium**: Potential issue
- **Low**: Code quality improvement
- **Info**: Informational finding

## Compilation Pipeline

```
NEXLANG Source (.nxl)
        ↓ Lexer
    Token Stream
        ↓ Parser
    Abstract Syntax Tree (AST)
        ↓ Type Checker
    Typed AST
        ↓ Security Analyzer
    Audit Report
        ↓ Code Generator
    NXVM Bytecode
        ↓ Serializer
    Deployable Binary (with NXVM magic header)
```

## Gas Costs

| Operation | Gas Cost |
|-----------|----------|
| ADD/SUB | 3 |
| MUL/DIV | 5 |
| SLOAD | 200 |
| SSTORE | 5,000 |
| CALL | 40 |
| HASH (BLAKE3) | 30 |
| VERIFY_SIG | 3,000 |
| CROSS_SHARD_CALL | 10,000 |
| EMIT | 375 |
