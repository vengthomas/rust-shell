
//! 
//! note: This module encapsulates the third party lib used for
//! enriched user input (navigation in the input with arrow, shortcuts handling (ctrl c, selecting text, copy paste) etc...) 
//! 
//! 
use std::{env, error::Error, path::PathBuf};
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::{cli::interaction::{Interaction, UserInput}, command::builtin::get_working_directory};

/// Represents what an interaction via the terminal with the users contains.
/// 
/// 
pub struct TerminalInteraction { 
    rusty_lines_editor: DefaultEditor,
    // The path to the file where the history is saved
    history_path: PathBuf
}

impl TerminalInteraction {
    
    /// Attempts to create a new TerminalInteraction instance, returns an error if any error occurs during creation
    /// 
    pub fn try_new() -> Result<Self, Box<dyn std::error::Error>> {    

        // The creation of rusty_lines objects may fail 
        let mut rusty_lines_editor = DefaultEditor::new()?;

        let mut temp_path: PathBuf = env::temp_dir();
        // Creates a file in temporary folder (/tmp on linux for example) where the history will be saved
        temp_path.push("rust_shell_history.txt");
        let _ = rusty_lines_editor.load_history(&temp_path);

        Ok(TerminalInteraction {
            rusty_lines_editor,
            history_path: temp_path
        })
    }

}

impl Interaction for TerminalInteraction {

    /// Returns the input entered by the user on the stdin
    /// 
    /// Side effects: Prints the prompt string and modifies some attributes in the struct 
    fn receive_input(&mut self) -> Result<UserInput, Box<dyn Error>> {

        // side effect: also prints the prompt string
        let readline = self.rusty_lines_editor.readline(&self.prompt_string());
        match readline {
            // do nothing if the string is empty or is a bunch of spaces
            Ok(s) if s.trim().is_empty() => {
                Ok(UserInput::NoSpecialInput) 
            },
            Ok(line) => {
                // side effect: saves the line in the history
                self.rusty_lines_editor.add_history_entry(&line)?;
                self.save_history()?;

                Ok(UserInput::String(line))
            },
            Err(ReadlineError::Interrupted) => {
                Ok(UserInput::NoSpecialInput) // if it is ctrl c, just ignore it
            },
            Err(ReadlineError::Eof) => {
                Ok(UserInput::Eof)
            }
            Err(_) => Ok(UserInput::NoSpecialInput)  // if it is another error, just ignore it
        }

    }

    /// Save the previous inputs strings in the history, returns an error if any problem occurs during the saving
    /// 
    fn save_history(&mut self) -> Result<(), Box<dyn Error>> {
        self.rusty_lines_editor.save_history(&self.history_path)?;
        Ok(())
    }

    /// Returns the prefix on the left of the user input with the following format: `$ currentdirectory>`
    /// 
    fn prompt_string(&self) -> String {

        let working_dir = get_working_directory()
            .unwrap_or_else(|_| String::from("unknown"));

        // pretty colored shell prompt
        format!("$ \x1b[1;34m{}\x1b[0m> ", working_dir)
    }
}
