//! Functions related with command execution
//! 
//! 

use std::{error::Error, ffi::{CStr, CString}};

use crate::command::{Command, RedirectionType};
use nix::{fcntl::{OFlag,open}, sys::{stat::Mode, wait::{WaitStatus, waitpid}}, unistd::{ForkResult, dup2_stderr, dup2_stdin, dup2_stdout, execvp, fork, pipe}};

impl Command {

    /// Executes the command
    /// 
    /// 
    pub fn execute(&self)-> Result<(), Box<dyn Error>> {  // todo properly handle err

        // TODO fork only if it's not a builtin, or pipe etc..
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                waitpid(child, None)?;  
            }
            Ok(ForkResult::Child) => {
                self.execute_recursive()?;
            }
            Err(_) => println!("Fork failed"),
        }

        Ok(())
    }
    
    fn execute_recursive(&self) -> Result<(), Box<dyn Error>>{
        
        match self {
            Command::Simple{cmd_path, cmd_args} => {

                // TODO builtin command

                execute_simple_command(cmd_path, cmd_args)?;
                Ok(())
            },
            Command::Redirection { kind, command, file } => {
                execute_redirection_command(kind, command, file)?;
                Ok(())
            },
            Command::Pipe { left, right } => {
                execute_pipe_command(left, right)?;
                Ok(())
            },
            Command::Separator { left, right } => {
                execute_separator_command(left, right)?;
                Ok(())
            },
            Command::LogicalOr { left, right } => {
                execute_logical_command(left, right, false)?;
                Ok(())
            },
            Command::LogicalAnd { left, right } => {
                execute_logical_command(left, right, true)?;
                Ok(())
            },
            Command::Background { command } => {
                todo!()
            }
            
        }
    }

}

/// Executes a simple command by REPLACING the current process.
/// This function does not executes built-in commands (such as pwd or cd)
fn execute_simple_command(cmd_path: &str, cmd_args: &[String]) -> Result<(), Box<dyn Error>> {  

    // TODO less chaotic conversions 
    let cmd = CString::new(cmd_path)?;

    let mut args: Vec<CString> = Vec::with_capacity(cmd_args.len() + 1);
    args.push(cmd.clone()); // argv[0]

    for arg in cmd_args {
        args.push(CString::new(arg.as_str())?);
    }

    // convert to &[&CStr]
    let argv: Vec<&CStr> = args.iter().map(|c| c.as_c_str()).collect();

    execvp(&cmd, &argv)?;

    Ok(())   
}

fn execute_redirection_command(kind: &RedirectionType, command: &Command, file_path: &str) -> Result<(), Box<dyn Error>> {

    // Select the options creation/read depending on the kind 
    let open_flags = match kind {
        RedirectionType::In => OFlag::O_RDONLY,
        RedirectionType::Out | RedirectionType::Err => OFlag::O_TRUNC | OFlag::O_CREAT | OFlag::O_WRONLY,
        RedirectionType::Append => OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_APPEND
    };

    let fd = open(file_path, open_flags, Mode::from_bits(0o644).unwrap())?; // TODO maybe avoid unwrap

    match kind {
        RedirectionType::In => dup2_stdin(fd)?,
        RedirectionType::Out |  RedirectionType::Append => dup2_stdout(fd)?,
        RedirectionType::Err => dup2_stderr(fd)?
    };
    
    // Then execute the command with redirected input or output
    command.execute_recursive()?;

    Ok(())
}

fn execute_pipe_command(left_cmd: &Command, right_cmd: &Command) -> Result<(), Box<dyn Error>> {

    let (pipe_fd_read, pipe_fd_write) = pipe()?;

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            
            dup2_stdout(pipe_fd_write)?;

            left_cmd.execute_recursive()?;
            waitpid(child, None)?;  
        }
        Ok(ForkResult::Child) => {
            dup2_stdin(pipe_fd_read)?;
            // TODO make sure to wait for the parent for dup2  
            right_cmd.execute_recursive()?;
        }
        Err(_) => println!("Pipe fork failed"),
    }

    Ok(())
}

fn execute_separator_command(left_cmd: &Command, right_cmd: &Command) -> Result<(), Box<dyn Error>> {
    
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            waitpid(child, None)?;
            right_cmd.execute_recursive()?;
        }
        Ok(ForkResult::Child) => {
            if let Err(err) = left_cmd.execute_recursive() {
                eprintln!("Child error: {:?}", err);
                std::process::exit(1); 
            }
            std::process::exit(0);
        }
        Err(_) => return Err("Pipe in separator failed".into()),
    }

    Ok(())
}

/// Executes the right command depending on if the left command is an exit success
/// used for && and || commands
fn execute_logical_command(left_cmd: &Command, right_cmd: &Command, continue_on_success: bool) -> Result<(), Box<dyn Error>> {
    
    // TODO without fork
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            if let Ok(WaitStatus::Exited(_, status_code)) = waitpid(child, None) {
                if status_code == 0 && continue_on_success ||
                   status_code != 0 && !continue_on_success {
                    right_cmd.execute_recursive()?;
                }
            }
        }
        Ok(ForkResult::Child) => {
            if let Err(err) = left_cmd.execute_recursive() {
                eprintln!("Child error: {:?}", err);
                std::process::exit(1); 
            }
            std::process::exit(0);
        }
        Err(_) => return Err("Pipe in separator failed".into()),
    }

    Ok(())
}

/*impl Command {

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
                execute_background_command(command, io_context)
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
            status.success()
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

fn execute_background_command(command: &Command, io_context: IoContext) -> Result<Option<Child>, ExecutionError>  {
    command.execute_recursive(io_context)?;
    Ok(None)    // TODO wait without blocking to avoid zombie processes
}*/

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