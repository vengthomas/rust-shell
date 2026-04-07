
use std::fmt::Display;

use nix::errno::Errno;
use nix::sys::wait::{WaitPidFlag};
use nix::unistd::Pid;
use nix::{sys::{wait::{waitpid, WaitStatus}}};

use crate::command::Command;

#[derive(Default, Debug)]
pub struct JobsManager {
    background_jobs: Vec<Job>
}

#[derive(Debug)]
pub struct Job {
    job_number: usize,
    pid: i32,
    command: Command,
    state: State,
}

#[derive(Debug)]
pub enum State {
    Running,
    Stopped,
    Done,
    Killed
}

// TODO docs

impl JobsManager {

    pub fn new() -> Self {
        JobsManager::default()
    }

    /// side effect TODO DOC
    pub fn add_background_job(&mut self, command: &Command, pid: i32) {

        let job_number = self.last_job_number();

        let job = Job {
            job_number,
            pid,
            command: command.clone(), // TODO avoid clone ?
            state: State::Running
        };

        println!("[{}] {}", job_number, pid); //TODO move elsewhere

        self.background_jobs.push(job);
        
    }

    fn last_job_number(&self) -> usize {
        match self.background_jobs.last() {
            Some(latest_job) => latest_job.job_number+1,
            None => 1,
        }
    }

    /// Temporary function, should clean zombies with SIGCHLD signal handling
    /// non blocking polling to cleanup zombie processes from background jobs when they are done
    /// 
    /// side effect TODO DOC
    pub fn clean_done_jobs(&mut self) {

        // TODO while loop to clean all zombies once
        match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
            Ok(status) => match status {
                WaitStatus::Exited(pid, code) => {

                    let mut removed_job = self.remove_background_job(pid.as_raw()).unwrap(); // todo handle err
                    removed_job.state = State::Done;
                    println!("[{}]   {:?}\t\t{}", removed_job.job_number, removed_job.state, removed_job.command.to_string());

                }
                WaitStatus::StillAlive => (),
                /*WaitStatus::Signaled(pid, signal, _) => unimplemented!(),
                WaitStatus::Stopped(pid, signal) => unimplemented!(),
                WaitStatus::Continued(pid) => unimplemented!(),
                WaitStatus::PtraceEvent(pid, signal, _) => unimplemented!(),
                WaitStatus::PtraceSyscall(pid) => unimplemented!(),*/
                _ => unimplemented!()
            },
            Err(e) => {
                // do not signal it as a problem is there is no child to wait
                if e == Errno::ECHILD {
                    return;
                }
                eprintln!("waitpid failed: {}", e);
            }
        }
    }

    fn remove_background_job(&mut self, pid: i32) -> Result<Job, ()> { // Ok(removed_job) or err if not found

        if let Some(pos) = self.background_jobs.iter().position(|e| e.pid == pid) {
            let removed_job = self.background_jobs.remove(pos);
            Ok(removed_job)
        } else {
            Err(())
        }
    }
}
