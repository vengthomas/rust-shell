

use nix::sys::wait::{WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use nix::{sys::{wait::{waitpid}}};

use crate::command::Command;

pub struct JobsManager {
    jobs: Vec<Job>
}

pub struct Job {
    job_number: usize,
    pid: i32,
    command: Command,
    state: State,
}

pub enum State {
    STOPPED,
    RUNNING,
    DONE
}

impl JobsManager {

    pub fn new() -> Self {
        JobsManager { jobs: Vec::new() }
    }

    /// Temporary function, should clean zombies with SIGSHLD signal handling
    /// non blocking polling to cleanup zombie processes from background jobs when they are done
    pub fn clean_done_jobs(&self) {
        match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(_, _)) => (),
            Ok(WaitStatus::Signaled(_, _, _)) => (),
            Ok(WaitStatus::StillAlive) => (),
            Err(_) => (),
            _ => (),
        }
    }
}
