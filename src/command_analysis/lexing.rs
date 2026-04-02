
// Use the RedirectionType enum for both the tokens (in the lexing) and the AST (in the Command enum)
use crate::command::RedirectionType;

#[derive(Clone, PartialEq, Debug)]
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

    /*#[test]
    fn test_without_spaces_between_pipe() {

        let input = "ls -l /|cat".to_string();
        let result = tokenize_input(&input);

        let expected = vec![Token::Word("ls".into()), Token::Word("/".into()), Token::Pipe, Token::Word("cat".into())];

        assert_eq!(expected, result);
    }*/
}