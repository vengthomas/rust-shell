
use nix::errno::Errno;
use nix::sys::wait::{WaitPidFlag};
use nix::unistd::Pid;
use nix::{sys::{wait::{waitpid, WaitStatus}}};

use std::fmt;

use crate::command::Command;

/// Struct containing the background jobs
/// 
/// Invariants: 
/// - The last background job has always the highest value
/// 
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

impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] {:?}\t{} &", self.job_number, self.state, self.command)
    }
}

/// Represents a job execution state
#[derive(Debug)]
pub enum State {
    Running,
    Stopped,
    Done,
    Killed
}

impl JobsManager {

    /// Adds a new background job to the manager
    /// 
    /// Side effect: prints in the console the newly added job
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

    /// Returns the active background jobs as a read-only list
    pub fn active_jobs(&self) -> &Vec<Job> {
        &self.background_jobs
    }

    /// Returns the next job number based on the last job in `background_jobs`.
    ///
    /// Precondition: `background_jobs` is sorted by insertion order
    fn last_job_number(&self) -> usize {
        match self.background_jobs.last() {
            Some(latest_job) => latest_job.job_number+1,
            None => 1,
        }
    }

    // Temporary function, should clean zombies with SIGCHLD signal handling
    /// Cleans up the zombie processes with non-blocking polling.
    /// Update the background jobs list if the zombie is a background job
    /// 
    /// Side effect: prints to stdout the removed job(s?)
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

    /// Removes in the list the job with corresponding `pid` and 
    /// 
    /// Returns either:
    /// - Ok(removed_job) if the removal is successful
    /// - Err(()) if job with `pid` was not found
    fn remove_background_job(&mut self, pid: i32) -> Result<Job, ()> { 

        if let Some(pos) = self.background_jobs.iter().position(|e| e.pid == pid) {
            let removed_job = self.background_jobs.remove(pos);
            Ok(removed_job)
        } else {
            Err(()) 
        }
    }
}
