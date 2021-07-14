use regex::Regex;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{PathBuf};
use std::{fs};
use log::{debug, warn};

use crate::proc::{ProcProvider, Process, ProcessStat};

pub const LINUX_PROC_DIR: &'static str = "/proc";

pub const ROOT_PROC_NAME: &'static str = "?";

lazy_static! {
    static ref STAT_RE: Regex = Regex::new(r"(\w) (\d+) (\d+) (\d+) (\d+) (-?\d+) (\d+) (\d+) (\d+) (\d+) (\d+) (\d+) (\d+) (\d+) (\d+) (\d+) (\d+) (\d+) .*").unwrap();
}

#[derive(Copy, Clone)]
pub struct LinuxProvider {
    proc_path: &'static str,
}

impl LinuxProvider {
    pub fn new() -> Self {
        LinuxProvider { proc_path: LINUX_PROC_DIR }
    }

    #[allow(dead_code)]
    pub fn change_proc_path(&mut self, proc_path: &'static str) -> &mut Self {
        self.proc_path = proc_path;
        self
    }
}

impl ProcProvider for LinuxProvider {
    fn get_process_tree(self, pid: u32) -> Option<Process> {
        // 1. get all process in /proc
        let mut procs_stat: Vec<ProcessStat> = Vec::new();

        match fs::read_dir(self.proc_path) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        // Here, `entry` is a `DirEntry`.
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_dir() && is_process_dir(entry.file_name()) {
                                // Now let's show our entry's file type!
                                let stat_path = entry.path().join("stat");

                                if let Some(stat) = read_proc_stat(stat_path.clone()) {
                                    procs_stat.push(stat)
                                } else {
                                    warn!("stat {} disappear.", stat_path.clone().display());
                                }
                            }
                        }
                    }
                }
            }
            Err(error) => panic!("Couldn't read {}, error is {}", self.proc_path, error),
        };

        debug_procs_stat(&procs_stat);

        // 2. find root_proc by given pid and children
        let mut pgid_pid_map: HashMap<u32, Vec<u32>> = HashMap::default();
        let mut process_map: HashMap<u32, Process> = HashMap::default();

        for proc_stat in procs_stat.iter() {
            let process = Process {
                name: proc_stat.name.clone(),
                stat: proc_stat.to_owned(),
                children: vec![],
            };

            process_map.insert(proc_stat.pid, process);

            if !pgid_pid_map.contains_key(&proc_stat.ppid) {
                pgid_pid_map.insert(proc_stat.ppid, Vec::new());
            }
            pgid_pid_map
                .get_mut(&proc_stat.ppid)
                .unwrap()
                .push(proc_stat.pid);
        }

        let children = build_process_tree(0, &mut process_map, pgid_pid_map);
        process_map.insert(
            0,
            Process {
                name: ROOT_PROC_NAME.to_string(),
                stat: ProcessStat::default(),
                children: children,
            },
        );

        process_map.get(&pid).and_then(|p| Some(p.to_owned()))
    }
}

fn debug_procs_stat(procs_stat: &[ProcessStat]) {
    for proc_stat in procs_stat {
        debug!("pid: {}, ppid: {}, name: {}", proc_stat.pid, proc_stat.ppid, proc_stat.name.to_string())
    }
}

/// build_process_tree recusive set children
fn build_process_tree(
    ppid: u32,
    process_map: &mut HashMap<u32, Process>,
    pgid_pid_map: HashMap<u32, Vec<u32>>,
) -> Vec<Process> {
    let mut process_list: Vec<Process> = Vec::new();

    if pgid_pid_map.contains_key(&ppid) {
        for pid in pgid_pid_map.get(&ppid).unwrap() {
            let children = build_process_tree(pid.to_owned(), process_map, pgid_pid_map.clone());
            let process = process_map.get_mut(&pid).unwrap();
            process.set_children(children);
            process_list.push(process.to_owned());
        }
    }

    process_list
}

fn read_proc_stat(stat_path: PathBuf) -> Option<ProcessStat> {
    let contents = fs::read_to_string(stat_path.clone());
    if contents.is_err() {
        warn!(
            "read file {:?} error: {:?}",
            stat_path.as_path(),
            contents.err()
        );
        return None;
    }
    let contents = contents.unwrap();

    let fields: Vec<&str> = split_keep(&Regex::new(r"\(.*\)").unwrap(), contents.as_str());
    if fields.len() != 3 {
        return None;
    }

    let mut process_stat = ProcessStat::default();
    process_stat.name = fields[1]
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .to_string();
    process_stat.pid = fields[0].trim().parse::<u32>().unwrap();
    if let None = parse_stat_str(&mut process_stat, fields[2]) {
        None
    } else {
        Some(process_stat)
    }
}

fn parse_stat_str(stat: &mut ProcessStat, line: &str) -> Option<()> {
    let caps = STAT_RE.captures(line.trim())?;

    stat.state = caps.get(1)?.as_str().as_bytes()[0] as char;
    stat.ppid = caps.get(2)?.as_str().parse::<u32>().unwrap();
    stat.pgrp = caps.get(3)?.as_str().parse::<u32>().unwrap();
    stat.session = caps.get(4)?.as_str().parse::<u32>().unwrap();
    stat.tty_nr = caps.get(5)?.as_str().parse::<u32>().unwrap();
    stat.tpgid = caps.get(6)?.as_str().parse::<i32>().unwrap();
    stat.flags = caps.get(7)?.as_str().parse::<u64>().unwrap();
    stat.minflt = caps.get(8)?.as_str().parse::<u64>().unwrap();
    stat.cminflt = caps.get(9)?.as_str().parse::<u64>().unwrap();
    stat.majflt = caps.get(10)?.as_str().parse::<u64>().unwrap();
    stat.cmajflt = caps.get(11)?.as_str().parse::<u64>().unwrap();
    stat.utime = caps.get(12)?.as_str().parse::<u64>().unwrap();
    stat.cutime = caps.get(13)?.as_str().parse::<u64>().unwrap();
    stat.stime = caps.get(14)?.as_str().parse::<u64>().unwrap();
    stat.cstime = caps.get(15)?.as_str().parse::<u64>().unwrap();
    stat.priority = caps.get(16)?.as_str().parse::<u64>().unwrap();
    stat.nice = caps.get(17)?.as_str().parse::<u64>().unwrap();
    stat.nthreads = caps.get(18)?.as_str().parse::<u64>().unwrap();

    Some(())
}

fn is_process_dir(filename: OsString) -> bool {
    let filename = filename.to_str();
    match filename {
        Some(filename) => {
            if let Err(_) = filename.parse::<u32>() {
                false
            } else {
                true
            }
        }
        None => false,
    }
}

fn split_keep<'a>(r: &Regex, text: &'a str) -> Vec<&'a str> {
    let mut result = Vec::new();
    let mut last = 0;
    for (index, matched) in text.match_indices(r) {
        if last != index {
            result.push(&text[last..index]);
        }
        result.push(matched);
        last = index + matched.len();
    }
    if last < text.len() {
        result.push(&text[last..]);
    }
    result
}

impl Process {
    /// Set the process's children.
    fn set_children(&mut self, children: Vec<Process>) {
        self.children = children;
    }
}

impl Default for Process {
    fn default() -> Self {
        Process {
            name: String::new(),
            stat: ProcessStat::default(),
            children: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_linux_provider() -> LinuxProvider {
        LinuxProvider::new()
    }

    #[test]
    fn test_it_work() {
        let mut p = new_linux_provider();
        p.change_proc_path("/Users/rry/Code/go/src/github.com/rrylee/rspstree/src/proc/testdata");
        let process  = p.get_process_tree(
            0,
        );

        let process = process.unwrap();
        assert_eq!(process.name, ROOT_PROC_NAME);
        assert_eq!(process.children.len(), 2);
        assert_eq!(process.children[0].children.len(), 1);
        assert_eq!(process.children[1].children.len(), 0);

        let process = p.get_process_tree(
            1,
        );
        let process = process.unwrap();
        assert_eq!(process.name, "systemd");
        assert_eq!(process.children.len(), 1);
    }

    #[test]
    fn test_split() {
        let stat_str = "2 (kthreadd) S 0 0 0 0 -1 2129984 0 0 0 0 0 391 0 0 20 0 1 0 7 0 0 18446744073709551615 0 0 0 0 0 0 0 2147483647 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0";
        let fields: Vec<&str> = split_keep(&Regex::new(r"\(.*\)").unwrap(), stat_str);

        assert_eq!(fields.len(), 3);
    }

    #[test]
    fn test_parse_stat_str() {
        let stat_str = " S 0 0 0 0 -1 2129984 0 0 0 0 0 391 0 0 20 0 1 0 7 0 0 18446744073709551615 0 0 0 0 0 0 0 2147483647 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0";
        let mut process_stat = ProcessStat::default();
        parse_stat_str(&mut process_stat, stat_str);

        assert_eq!(process_stat.state, 'S');
        assert_eq!(process_stat.ppid, 0);
        assert_eq!(process_stat.pgrp, 0);
        assert_eq!(process_stat.session, 0);
        assert_eq!(process_stat.tty_nr, 0);
        assert_eq!(process_stat.tpgid, -1);
        assert_eq!(process_stat.flags, 2129984);
        assert_eq!(process_stat.minflt, 0);
        assert_eq!(process_stat.cminflt, 0);
        assert_eq!(process_stat.majflt, 0);
        assert_eq!(process_stat.cmajflt, 0);
        assert_eq!(process_stat.utime, 0);
        assert_eq!(process_stat.cutime, 391);
        assert_eq!(process_stat.stime, 0);
        assert_eq!(process_stat.cstime, 0);
        assert_eq!(process_stat.priority, 20);
        assert_eq!(process_stat.nice, 0);
        assert_eq!(process_stat.nthreads, 1);
    }
}
