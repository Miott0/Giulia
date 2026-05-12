use thiserror::Error;

use crate::value::Value;

#[derive(Error, Debug, Clone)]
pub enum RuntimeError {
    #[error("Variable not found: `{name}` at line {line}, column {column}")]
    VariableNotFound { name: String, line: usize, column: usize },

    #[error("Type error: {message} at line {line}, column {column}")]
    TypeError { message: String, line: usize, column: usize },

    #[error("Division by zero at line {line}, column {column}")]
    DivisionByZero { line: usize, column: usize },

    #[error("Return value: {0:?}")]
    Return(Box<Value>),

    #[error("Native function error: {message} at line {line}, column {column}")]
    NativeError { message: String, line: usize, column: usize },
}
