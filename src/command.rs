
//! Module related with commands execution, treatment etc
//! 
//! 

pub mod execution;
pub mod builtin;
pub mod jobs;

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

impl ToString for RedirectionType {
    fn to_string(&self) -> String {
        match &self {
            RedirectionType::In => "<".to_string(),
            RedirectionType::Out => ">".to_string(),
            RedirectionType::Append => ">>".to_string(),
            RedirectionType::Err => "2>".to_string(),
        }
    }
}

impl ToString for Command {
    fn to_string(&self) -> String {
        match &self {
            Command::Simple { cmd_path, cmd_args } => {
                format!("{}{}", cmd_path, cmd_args.join(" "))
            },
            Command::Pipe { left, right } => {
                format!("{} | {}", left.to_string(), right.to_string())
            },
            Command::Redirection { kind, command, file } => {
                format!("{} {} {}", command.to_string(), kind.to_string(), file)
            },
            Command::Separator { left, right } => {
                format!("{}; {}", left.to_string(), right.to_string())
            },
            Command::LogicalOr { left, right } => {
                format!("{} || {}", left.to_string(), right.to_string())
            },
            Command::LogicalAnd { left, right } => {
                format!("{} && {}", left.to_string(), right.to_string())
            },
            Command::Background { command } => {
                format!("{} &",command.to_string())
            },
        }
    }
}
