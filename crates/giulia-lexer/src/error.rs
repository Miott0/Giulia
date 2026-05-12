use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexError {
    #[error("Unexpected character '{character}' at line {line}, column {column}")]
    UnexpectedCharacter {
        character: char,
        line: usize,
        column: usize,
    },

    #[error("String not terminated at line {line}, column {column}")]
    UnterminatedString {
        line: usize,
        column: usize,
    },

    #[error("Comentario multilinha não terminado na linha {line}, coluna {column}")]
    UnterminatedComment {
        line: usize,
        column: usize,
    },
}