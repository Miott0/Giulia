pub mod error;
pub mod token;

use logos::Logos;
use token::{Span, SpannedToken, Token};
use error::LexError;

pub type LexResult = Result<Vec<SpannedToken>, Vec<LexError>>;

/// Converte `source` em uma lista de `SpannedToken`.
/// Coleta todos os erros (`LexError`) encontrados e os retorna em vetor.
pub fn lex(source: &str) -> LexResult {
    let mut tokens: Vec<SpannedToken> = Vec::new();
    let mut errors: Vec<LexError> = Vec::new();

    let mut out_bytes = source.as_bytes().to_vec();
    let mut search_start = 0usize;
    while let Some(rel_start) = source[search_start..].find("/:") {
        let start = search_start + rel_start;
        let after = &source[start + 2..];
        let rel_end = after.find(":/");
        let rel_nl = after.find('\n');

        if let Some(re) = rel_end {
            let end = start + 2 + re + 2;
            for i in start..end {
                if out_bytes[i] != b'\n' {
                    out_bytes[i] = b' ';
                }
            }
            search_start = end;
        } else if let Some(nl) = rel_nl {
            let end = start + 2 + nl;
            for i in start..end {
                if out_bytes[i] != b'\n' {
                    out_bytes[i] = b' ';
                }
            }
            search_start = end + 1;
        } else {
            for i in start..out_bytes.len() {
                if out_bytes[i] != b'\n' {
                    out_bytes[i] = b' ';
                }
            }
            break;
        }
    }
    let processed = String::from_utf8(out_bytes).expect("UTF-8 preserved");
    let mut line: usize = 1;
    let mut line_start: usize = 0;

    let mut lexer = Token::lexer(&processed);

    while let Some(item) = lexer.next() {
        let range = lexer.span();
        let span = Span { start: range.start, end: range.end };
        let column = span.start.saturating_sub(line_start) + 1; // coluna começa em 1

        match item {
            Ok(Token::Newline) => {
                tokens.push(SpannedToken { token: Token::Newline, span, line, column });
                line += 1;
                line_start = span.end;
            }
            Ok(tok) => {
                tokens.push(SpannedToken { token: tok, span, line, column });
            }
            Err(_) => {
                // caractere inválido/token não reconhecido
                let slice = &source[span.start..span.end];
                let bad = slice.chars().next().unwrap_or('?');
                errors.push(LexError::UnexpectedCharacter { character: bad, line, column });
            }
        }
    }

    // EOF sintético
    let eof_pos = source.len();
    tokens.push(SpannedToken {
        token: Token::Eof,
        span: Span { start: eof_pos, end: eof_pos },
        line,
        column: eof_pos.saturating_sub(line_start) + 1,
    });

    if errors.is_empty() { Ok(tokens) } else { Err(errors) }
}