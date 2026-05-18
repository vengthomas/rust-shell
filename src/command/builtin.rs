
//! Shell commands that needs a special treatment, 
//! for instance exit and cd
//!  

pub mod execution;


// utils for builtin commands
pub fn exit_shell(exit_code: i32) {
    std::process::exit(exit_code)
}

pub fn change_directory(to: &str) -> Result<(), Box<dyn std::error::Error>> {
    
    let path = std::path::Path::new(to);
    std::env::set_current_dir(path)?;

    Ok(())
}

pub fn get_working_directory() -> Result<String, Box<dyn std::error::Error>> {
    
    let path = std::env::current_dir()?;
    Ok(path.to_string_lossy().into_owned())
}


#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    // Mutex used to prevent concurrent executions from interfering with directory changes
    // Without this, concurrent tests could change the process working directory simultaneously, leading to incoherent states.
    // Tests run in parallel in Rust by default
    static CD_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn cd_root_sets_working_directory_to_root() {

        let _lock = CD_LOCK.lock().unwrap();

        change_directory("/").unwrap();
        
        let working_dir = get_working_directory().unwrap();
        assert_eq!("/", working_dir);

        // Automatically free the lock since it's dropped here
    }

    #[test]
    fn cd_home_sets_working_directory_to_home() {

        let _lock = CD_LOCK.lock().unwrap();

        let home = std::env::home_dir().unwrap();
        change_directory(home.to_str().unwrap()).unwrap();
        let working_dir = get_working_directory().unwrap();

        assert_eq!(home.to_str().unwrap(), working_dir);
        
    }
}