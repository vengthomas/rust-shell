
//! Module related with commands execution, treatment etc
//! 
//! 

use std::{process::Stdio};

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

/// Struct containing what stdin should be and where stdout and stderr should go.
/// It may be used to specify redirections and pipe destinations, and be used for testing
pub struct IoContext {
    // pass Stdio as Option to make consuming the ownership easier, 
    // None represents a IO that will be inherited from parent during execution 
    pub stdin: Option<Stdio>, 
    pub stdout: Option<Stdio>,
    pub stderr: Option<Stdio>,
}

impl IoContext {
    
    pub fn new() -> Self {
        IoContext { 
            stdin: None, 
            stdout: None, 
            stderr: None 
        }
    }
}

impl Default for IoContext {

    fn default() -> Self {
        Self::new()
    }
}