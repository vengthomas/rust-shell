//! 
//! This module processes the input text (entered by the user) and produces an AST
//! 

pub mod lexing;
pub mod parsing;

use crate::command::Command;

use self::lexing::tokenize_input;
use self::parsing::parse;

/// Converts a string representing a command into a Command structure
/// For example "ls /home" gives SimpleCommand("ls", ["/home"])
pub fn convert_to_command(input: &str) -> Result<Command, Box<dyn std::error::Error>>  {
    
    // Turns the input in a vec of Strings 
    let input_tokens = tokenize_input(input);
    // Turns the tokens into a command structure
    let command = parse(&input_tokens)?;

    Ok(command)
}