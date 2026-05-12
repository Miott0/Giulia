mod error;
mod parser;


pub use error::ParseError;
pub use parser::Parser;

use giulia_lexer::token::SpannedToken;
use giulia_ast::node::Program;

pub fn parse(tokens: Vec<SpannedToken>) -> Result<Program, Vec<ParseError>> {
    Parser::new(tokens).parse_program()    
}


