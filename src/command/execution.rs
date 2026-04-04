//! Functions related with command execution
//! 
//! 

use std::process::{Child, Stdio};
use std::fs::OpenOptions;

use crate::command::builtin::execution::try_execute_builtin;
use crate::command::{IoContext, RedirectionType};
use crate::command::Command;

impl Command {

    /// Executes the command and waits for it to complete if necessary.
    /// 
    /// 
    pub fn execute(&self, io_context: IoContext)-> Result<(), ExecutionError> {

        // Execute the command and waiting the child process if any
        if let Some(mut child_process) = self.execute_recursive(io_context)? {
            child_process.wait()?;
        }

        Ok(())
    }

    /// Recursively executes the command depending on its type by propagating a transformed IO context.
    /// 
    /// Depending on the command variant, this function may executes a simple command, 
    /// Or recursively call functions for composed commands like redirections, pipes etc...
    /// 
    /// Returns either :
    /// - Ok(None) if there is no child process to wait (the case for the built-in commands)
    /// - Ok(Some(_)) if there is a child process executed
    /// - Err(_) if there is error during the command execution
    fn execute_recursive(&self, io_context: IoContext) -> Result<Option<Child>, ExecutionError>{
        // `io_context`: Passed by ownership because it will be transformed throught the recursive calls
        
        match self {
            Command::Simple{cmd_path, cmd_args} => {

                // Execute the built in command if it is 
                if let Some(()) = try_execute_builtin(cmd_path, cmd_args, &io_context).map_err(|_| ExecutionError::BuiltinExecError )? { // TODO more detail from builtin error
                    // Built-in functions are not executed in child processes, so return None
                    return Ok(None);
                }
                // If not treat it like any other simple command 
                Ok(Some(execute_simple_command(cmd_path, cmd_args, io_context)?))

            },
            Command::Redirection { kind, command, file } => {
                execute_redirection_command(kind, command, file, io_context)
            },
            Command::Pipe { left, right } => {
                execute_pipe_command(left, right, io_context)
            },
            Command::Separator { left, right } => {
                execute_separator_command(left, right, io_context)
            },
            Command::LogicalOr { left, right } => {
                execute_logical_op_command(left, right, io_context, true)
            },
            Command::LogicalAnd { left, right } => {
                execute_logical_op_command(left, right, io_context, false)
            },
            Command::Background { command } => {
                todo!()
            }
            
        }
    }


}

/// Executes a simple command by creating a child process with the io_context as stdin/stdout/stderr
/// This function does not executes built-in commands (such as pwd or cd)
/// 
/// Returns the child process executing the command
/// 
fn execute_simple_command(cmd_path: &str, cmd_args: &[String], io_context: IoContext) -> Result<Child, ExecutionError> {  

    let child = std::process::Command::new(cmd_path)
        .args(cmd_args)
        // If no io context, pass the parent process standard io 
        .stdin(io_context.stdin.unwrap_or(Stdio::inherit()))
        .stdout(io_context.stdout.unwrap_or(Stdio::inherit()))
        .stderr(io_context.stderr.unwrap_or(Stdio::inherit()))
        .spawn()?;

    Ok(child)

}

fn execute_redirection_command(kind: &RedirectionType, command: &Command, file_path: &str, io_context: IoContext) -> Result<Option<Child>, ExecutionError>  {

    // Select the options creation/read depending on the kind 
    let mut options = OpenOptions::new();
    match kind {
        RedirectionType::In => {
            options.read(true);
        },
        RedirectionType::Out | RedirectionType::Err => {
            options.truncate(true).create(true).write(true);
        },
        RedirectionType::Append => {
            options.write(true).create(true).append(true);
        },
    }
    let file = options.open(file_path)?;

    let mut new_io_context = io_context; 
    match kind {
        RedirectionType::In => new_io_context.stdin = Some(Stdio::from(file)),
        RedirectionType::Out | RedirectionType::Append => new_io_context.stdout = Some(Stdio::from(file)),
        RedirectionType::Err => new_io_context.stderr = Some(Stdio::from(file)),
    }
    
    let child_process = command.execute_recursive(new_io_context)?;

    Ok(child_process)
}


fn execute_pipe_command(left_cmd: &Command, right_cmd: &Command, mut io_context: IoContext) -> Result<Option<Child>, ExecutionError> {

    let new_io_context = IoContext {
        stdin: io_context.stdin.take(),
        stdout: Some(Stdio::piped()),
        stderr: io_context.stderr.take(),
    };

    let left = left_cmd.execute_recursive(new_io_context)?;

    let mut left_child_process = left.ok_or(ExecutionError::MissingChildProcess)?;

    let right_io_context = IoContext {
        stdin: Some(Stdio::from(
            left_child_process.stdout.take().ok_or(ExecutionError::IoContextError)?
        )),
        stdout: io_context.stdout.take(),
        stderr: io_context.stderr.take(),
    };

    let mut right_child_process = right_cmd.execute_recursive(right_io_context)?.ok_or(ExecutionError::MissingChildProcess)?;

    // Prevent the child from being zombie processes
    right_child_process.wait()?;
    left_child_process.wait()?;

    Ok(Some(right_child_process))
}

fn execute_separator_command(left_cmd: &Command, right_cmd: &Command, io_context: IoContext) -> Result<Option<Child>, ExecutionError> {

    let left = left_cmd.execute_recursive(io_context);
    
    match left {
        Ok(Some(mut child)) => {
            child.wait()?;
        },
        Err(err) => eprintln!("{err}"),
        Ok(None) => ()
    }

    let right_io_context = IoContext::default();
    let mut right = right_cmd.execute_recursive(right_io_context)?;

    if let Some(right_child) = &mut right {
        right_child.wait()?;
    }

    Ok(None)
}

/// Executes either the || or the && operator command depending on the `or` argument
fn execute_logical_op_command(left_cmd: &Command, right_cmd: &Command, io_context: IoContext, or: bool ) -> Result<Option<Child>, ExecutionError> {

    let left = left_cmd.execute_recursive(io_context);
    
    let is_left_success : bool = match left {
        Ok(Some(mut child)) => {
            let status = child.wait()?;
            if status.success() {
                true
            } else {
                false
            }
        },
        Err(err) => {
            eprintln!("{err}");
            false
        },
        Ok(None) => true // todo handle this case
    };

    // if it's the || operator, the left should be a failure to execute the next commands
    // if it's the && operator, the left should be a success to execute the next commands
    let should_run_right = match or {
        true => !is_left_success,
        false => is_left_success
    };

    if should_run_right {

        let right_io_context = IoContext::default();
        let mut right = right_cmd.execute_recursive(right_io_context)?;

        if let Some(right_child) = &mut right {
            right_child.wait()?;
        }

    }
    Ok(None)
}


#[derive(thiserror::Error, Debug)]
pub enum ExecutionError {

    #[error("Command execution error: {0}")]
    CommandError(#[from] std::io::Error),

    #[error("Error occured during built-in command execution")]
    BuiltinExecError,

    #[error("Execution error with IO")]
    IoContextError,

    #[error("Expected a child process")]
    MissingChildProcess
}