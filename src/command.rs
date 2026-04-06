
//! Module related with commands execution, treatment etc
//! 
//! 

pub mod execution;
pub mod builtin;

/// Represents a command executable by a shell.
/// 
/// This enum represents the abstract syntax tree of a shell command created by the parsing module.
/// 
#[derive(PartialEq, Debug)]
pub enum Command {
    Simple {
        cmd_path: String,
        cmd_args: Vec<String>,
    },
    Pipe {
        left: Box<Command>,
        right: Box<Command>,
    },
    Redirection {
        kind: RedirectionType,
        command: Box<Command>,
        file: String,
    },
    Separator { // ;
        left: Box<Command>,
        right: Box<Command>,
    },    
    LogicalOr { // ||
        left: Box<Command>,
        right: Box<Command>,
    },
    LogicalAnd { // &&
        left: Box<Command>,
        right: Box<Command>,
    },
    // A command executed in background job
    Background { // cmd &
        command: Box<Command>
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum RedirectionType {
    In,       // <
    Out,      // >
    Append,   // >>
    Err,      // 2>
}
