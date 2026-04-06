use std::error::Error;

mod interaction;
mod terminal_interaction;

use crate::cli::interaction::{Interaction, UserInput};
use crate::cli::terminal_interaction::TerminalInteraction;
use crate::command::builtin::exit_shell;
use crate::command::jobs::JobsManager;
use crate::command_analysis::convert_to_command;


pub fn run_cli() {

    let mut terminal = TerminalInteraction::try_new().expect("error terminal interaction creation");
    let mut jobs_manager = JobsManager::new();

    println!(" ____            _     ____  _          _ _ ");
    println!("|  _ \\ _   _ ___| |_  / ___|| |__   ___| | |");
    println!("| |_) | | | / __| __| \\___ \\| '_ \\ / _ \\ | |");
    println!("|  _ <| |_| \\__ \\ |_   ___) | | | |  __/ | |");
    println!("|_| \\_\\\\__,_|___/\\__| |____/|_| |_|\\___|_|_|\n");

    loop {
        if let Err(err) = cli_loop_step(&mut terminal, &mut jobs_manager) {
            println!("{err}");
        }
    }
}

/// Processes a single step on a loop
pub fn cli_loop_step(terminal: &mut dyn Interaction, jobs_manager: &mut JobsManager) -> Result<(), Box<dyn Error>>{

    let user_input = terminal.receive_input()
        // Propagate the error by specifying it is a user input error
        .map_err(|e| Box::<dyn std::error::Error>::from(format!("Input error: {}", e)))?;

    match user_input {
        UserInput::String(input_string) => {
            
            let input_command = convert_to_command(&input_string)
                .map_err(|e| Box::<dyn std::error::Error>::from(format!("Parsing error: {}", e)))?;
            
            input_command.execute()
                .map_err(|e| Box::<dyn std::error::Error>::from(format!("Execution error: {}", e)))?; 

        },
        UserInput::NoSpecialInput => (), // If no special input, ignore it
        UserInput::Eof => {
            println!("exit");
            exit_shell(0)
        },
    }

    jobs_manager.clean_done_jobs();

    Ok(())
}
