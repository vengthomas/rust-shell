
//!
//! Module related with the interactions between with the user. 
//! 

use std::{error::Error};

/// Represents the contract that an interaction with the user should respect
pub trait Interaction {
    fn receive_input(&mut self) -> Result<UserInput, Box<dyn Error>>;
    fn save_history(&mut self) -> Result<(), Box<dyn Error>>;
    fn prompt_string(&self) -> String;
}

/// Represents what a user input could be, it could be just a string, or an action 
pub enum UserInput {
    String(String),
    Eof,          // ctrl d
    NoSpecialInput // a generic variant when no special action should happen
}

// TODO custom errors
/*pub enum InteractionError {
    InputError;
}*/