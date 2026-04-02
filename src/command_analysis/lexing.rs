
// Use the RedirectionType enum for both the tokens (in the lexing) and the AST (in the Command enum)
use crate::command::RedirectionType;

#[derive(Clone)]
pub enum Token {
    Word(String),
    RedirectOp(RedirectionType),
    Pipe,
    Separator,
    And,
    Or,
}


/// Converts an input string into a vec of tokens
pub fn tokenize_input(input: &str) -> Vec<Token> { 

    let mut tokens: Vec<Token> = Vec::new();

    for word in input.split_whitespace() {
        tokens.push(match word {   
            "<" => Token::RedirectOp(RedirectionType::In), 
            ">" => Token::RedirectOp(RedirectionType::Out),
            ">>" => Token::RedirectOp(RedirectionType::Append),
            "2>" => Token::RedirectOp(RedirectionType::Err),
            "|" => Token::Pipe,
            ";" =>Token::Separator,
            "||" => Token::Or,
            "&&" => Token::And,
            _ => Token::Word(word.to_string())
        }); 
    }

    tokens
}