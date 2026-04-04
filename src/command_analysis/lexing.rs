
// Use the RedirectionType enum for both the tokens (in the lexing) and the AST (in the Command enum)
use crate::command::RedirectionType;

use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
pub enum Token {

    #[regex(r"<|>>|2>|>", |lex| match lex.slice() {
        "<" => Some(RedirectionType::In),
        ">>" => Some(RedirectionType::Append),
        "2>"  => Some(RedirectionType::Err),
        ">"  => Some(RedirectionType::Out),
        _ => unreachable!(),
    })]
    RedirectOp(RedirectionType),

    #[token(";")]
    Separator,

    #[token("&&")]
    And,

    #[token("&")]
    BackgroundOp, // the background operator

    #[token("||")]
    Or,

    #[token("|")]
    Pipe,

    // Any other string that doesn't matches the previous patterns
    #[regex(r"[^ \t\n\f;|&<>]+", |lex| lex.slice().to_string())]
    Word(String),
}


/// Converts an input string into a vec of tokens
pub fn tokenize_input(input: &str) -> Vec<Token> { 

    let mut tokens: Vec<Token> = Vec::new();

    let mut lexer = Token::lexer(input);
    while let Some(result_token) = lexer.next() {
        //println!("{:?} {:?}", result_token, lexer.slice());
        tokens.push(result_token.unwrap()); // TODO do not unwrap and handle error
    }

    tokens
}


#[derive(thiserror::Error, Debug)]
pub enum LexingError {

    #[error("Unexpected token: {0}")]
    UnknownToken(String),
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_simple_command_with_pipe() {

        let input = "ls / | cat".to_string();
        let result = tokenize_input(&input);

        let expected = vec![Token::Word("ls".into()), Token::Word("/".into()), Token::Pipe, Token::Word("cat".into())];

        assert_eq!(expected, result);
    }

    #[test]
    fn redirection_types_are_correct() {

        let input = "< > >> 2>".to_string();
        let result = tokenize_input(&input);

        let expected = vec![
            Token::RedirectOp(RedirectionType::In), 
            Token::RedirectOp(RedirectionType::Out), 
            Token::RedirectOp(RedirectionType::Append),
            Token::RedirectOp(RedirectionType::Err)
        ];

        assert_eq!(expected, result);
    }

    #[test]
    fn test_logical_ops_and_separator() {

        let input = "|| ; &&".to_string();
        let result = tokenize_input(&input);

        let expected = vec![Token::Or, Token::Separator, Token::And];

        assert_eq!(expected, result);
    }

    #[test]
    fn test_without_spaces_around_pipe() {

        let input = "ls|cat".to_string();
        let result = tokenize_input(&input);

        let expected = vec![Token::Word("ls".into()), Token::Pipe, Token::Word("cat".into())];

        assert_eq!(expected, result);
    }

    // Tests some symbols that should'nt stick each others, 
    // for instance -l and / should be separated into two words
    /*#[test]
    fn specific_symbols_should_not_stick() {

        let input = "-l/|".to_string();
        let result = tokenize_input(&input);

        let expected = vec![Token::Word("-l".into()), Token::Word("/".into()), Token::Pipe];

        assert_eq!(expected, result);
    }*/
}