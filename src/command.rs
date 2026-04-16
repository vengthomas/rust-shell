
//! Module related with commands execution, treatment etc
//! 
//! 

use std::fmt::Display;

pub mod execution;
pub mod builtin;
pub mod jobs;

/// Represents a command executable by a shell.
/// 
/// This enum represents the abstract syntax tree of a shell command created by the parsing module.
/// 
#[derive(PartialEq, Debug, Clone)]
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

impl Display for RedirectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            RedirectionType::In => write!(f, "<"),
            RedirectionType::Out => write!(f, ">"),
            RedirectionType::Append => write!(f, ">>"),
            RedirectionType::Err => write!(f, "2>"),
        }
    }
}

impl Display for Command {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        match &self {
            Command::Simple { cmd_path, cmd_args } => {
                write!(f, "{}{}", cmd_path, cmd_args.join(" "))
            },
            Command::Pipe { left, right } => {
                write!(f, "{} | {}", left.to_string(), right.to_string())
            },
            Command::Redirection { kind, command, file } => {
                write!(f, "{} {} {}", command.to_string(), kind.to_string(), file)
            },
            Command::Separator { left, right } => {
                write!(f, "{}; {}", left.to_string(), right.to_string())
            },
            Command::LogicalOr { left, right } => {
                write!(f, "{} || {}", left.to_string(), right.to_string())
            },
            Command::LogicalAnd { left, right } => {
                write!(f, "{} && {}", left.to_string(), right.to_string())
            },
            Command::Background { command } => {
                write!(f, "{} &",command.to_string())
            },
        }
    }
}
