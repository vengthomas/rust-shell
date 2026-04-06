//!
//! Manages the built in commands execution 
//! 
//! 

use std::error::Error;

use crate::command::builtin::*;


/// Attempts to execute the command if the `cmd_path` is built-in command
/// Take io_context as a reference and not ownership because we do not want to transform it
/// 
/// Returns :
/// - Ok(Some(())) if `cmd_path` is a built-in command
/// - Ok(None) else
/// - Err(_) if an error occured during execution
///  
pub fn try_execute_builtin(cmd_path: &str, cmd_args: &[String]) -> Result<Option<()>, Box<dyn Error>> {
    
    match cmd_path {
        "exit" => exit_shell(0),
        // For now, cd takes no more arguments than the path
        "cd" => change_directory(cmd_args.first().ok_or("cd: missing arg")?)?,
        "pwd" => {
            let working_dir = get_working_directory()?;
            println!("{working_dir}");
        },
        _ => {
            return Ok(None);
        }
    }

    Ok(Some(()))
}