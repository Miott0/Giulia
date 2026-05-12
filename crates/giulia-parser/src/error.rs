use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected:?}, found {found:?} at line {line}, column {column}")]
    UnexpectedToken {
        expected:   String,
        found:      String,
        line:       usize,
        column:     usize,
    },

    #[error("Expected identifier in {context} at line {line}, column {column}")]
    ExpectedIdentifier {
        context:    String,
        line:       usize,
        column:     usize,
    },

    #[error("Invalid assignment target on row {line}, column {column}")]
    InvalidAssignmentTarget {
        line:       usize,
        column:     usize,
    },

    #[error("Invalid channel type on row {line}, column {column}")]
    InvalidChannelType {
        line:       usize,
        column:     usize,
    }
}
