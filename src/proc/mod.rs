pub mod linux_proc;

use serde::{Deserialize, Serialize};

pub trait ProcProvider {
    fn get_process_tree(self, pid: u32) -> Option<Process>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Process {
    name: String,
    stat: ProcessStat,
    children: Vec<Process>,
}

impl Process {
    /// Get a reference to the process's name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Get a reference to the process's children.
    pub fn children(&self) -> &[Process] {
        self.children.as_slice()
    }

    pub fn pid(&self) -> u32 {
        self.stat.pid
    }

    /// Get a reference to the process stat's ppid.
    pub fn ppid(&self) -> u32 {
        self.stat.ppid
    }
}

// ProcessStat
// https://man7.org/linux/man-pages/man5/proc.5.html
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessStat {
    pub pid: u32,
    pub name: String,
    pub state: char,
    pub ppid: u32,    // The PID of the parent of this process.
    pub pgrp: u32,    // The process group ID of the process.
    pub session: u32, // The session ID of the process.
    pub tty_nr: u32,  // The controlling terminal of the process
    pub tpgid: i32, // The ID of the foreground process group of the controlling terminal of the process.
    pub flags: u64, // The kernel flags word of the process.
    // The number of minor faults the process has made
    //  which have not required loading a memory page from
    //  disk.
    pub minflt: u64,
    // The number of minor faults that the process's
    // waited-for children have made.
    pub cminflt: u64,
    // The number of major faults the process has made
    // which have required loading a memory page from
    // disk.
    pub majflt: u64,
    // The number of major faults that the process's
    // waited-for children have made.
    pub cmajflt: u64,
    // Amount of time that this process has been scheduled in user mode
    pub utime: u64,
    pub cutime: u64,
    // Amount of time that this process has been scheduled in system mode
    pub stime: u64,
    pub cstime: u64,
    pub priority: u64, // priority
    pub nice: u64,     // the nice value
    pub nthreads: u64,
}

impl Default for ProcessStat {
    fn default() -> Self {
        ProcessStat {
            pid: 0,
            name: String::new(),
            state: char::default(),
            ppid: 0,
            pgrp: 0,
            session: 0,
            tty_nr: 0,
            tpgid: 0,
            flags: 0,
            minflt: 0,
            cminflt: 0,
            majflt: 0,
            cmajflt: 0,
            utime: 0,
            cutime: 0,
            stime: 0,
            cstime: 0,
            priority: 0,
            nice: 0,
            nthreads: 0,
        }
    }
}