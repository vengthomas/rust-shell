//! Functions related with command execution
//! 
//! 

use std::{error::Error, ffi::{CStr, CString}, io::{stdin, stdout, stderr}};

use crate::command::{Command, RedirectionType, builtin::execution::try_execute_builtin};
use nix::{fcntl::{OFlag,open}, sys::{stat::Mode, wait::{WaitStatus, waitpid}}, unistd::{ForkResult, dup, dup2_stderr, dup2_stdin, dup2_stdout, execvp, fork, pipe}};

impl Command {

    /// Executes the command
    /// 
    /// 
    pub fn execute(&self)-> Result<(), Box<dyn Error>> {  // todo properly handle err

        // Save the original io fds because it they could be redirected 
        let saved_stdin = dup(stdin())?;
        let saved_stdout = dup(stdout())?;
        let saved_stderr = dup(stderr())?;

        self.execute_recursive()?;

        // Then restore the fds
        dup2_stdin(saved_stdin)?;
        dup2_stdout(saved_stdout)?;
        dup2_stderr(saved_stderr)?;

        Ok(())
    }
    
    fn execute_recursive(&self) -> Result<(), Box<dyn Error>>{
        
        match self {
            Command::Simple{cmd_path, cmd_args} => {

                if let Ok(Some(())) = try_execute_builtin(cmd_path, cmd_args) { // TODO more detail from builtin error
                    return Ok(());
                }

                // Executes the command in subprocess if it's not a builtin
                execute_simple_command(cmd_path, cmd_args)?;
            },
            Command::Redirection { kind, command, file } => {
                execute_redirection_command(kind, command, file)?;
            },
            Command::Pipe { left, right } => {
                execute_pipe_command(left, right)?;
            },
            Command::Separator { left, right } => {
                execute_separator_command(left, right)?;
            },
            Command::LogicalOr { left, right } => {
                execute_logical_command(left, right, false)?;
            },
            Command::LogicalAnd { left, right } => {
                execute_logical_command(left, right, true)?;
            },
            Command::Background { command } => {
                execute_background_command(command)?;
            }
            
        };
        
        Ok(())
    }
    

}

/// Executes a simple command in a subprocess.
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


    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            waitpid(child, None)?;  
        }
        Ok(ForkResult::Child) => {
            execvp(&cmd, &argv)?;
        }
        Err(_) => println!("Fork failed"),
    }

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

fn execute_background_command(command: &Command) -> Result<(), Box<dyn Error>> {
    
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            // TODO add the pid to jobs
        }
        Ok(ForkResult::Child) => {
            match command.execute_recursive() {
                Ok(_) => std::process::exit(0),
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    std::process::exit(1);
                }
            }
        }
        Err(_) => return Err("Pipe in separator failed".into()),
    }

    Ok(()) 
}

// #[derive(thiserror::Error, Debug)]
// pub enum ExecutionError {

//     #[error("Command execution error: {0}")]
//     CommandError(#[from] std::io::Error),

//     #[error("Error occured during built-in command execution")]
//     BuiltinExecError,

//     #[error("Execution error with IO")]
//     IoContextError,

//     #[error("Expected a child process")]
//     MissingChildProcess
// }