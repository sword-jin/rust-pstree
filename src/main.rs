#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use clap::{App, AppSettings, Arg};
use env_logger::{Builder, Target};
use log::{debug, warn};
use log::{Level, LevelFilter, Record};
use std::io::Write;

use crate::proc::ProcProvider;

mod print;
mod proc;

const DEFAULT_PID: u32 = 1;

fn main() {
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.format(|buf, record| {
        writeln!(
            buf,
            "{}{}\x1b[0m",
            get_logging_prefix(record),
            record.args()
        )
    });

    let matches = App::new("rstree")
        .setting(AppSettings::TrailingVarArg)
        .version(env!("CARGO_PKG_VERSION"))
        .author("rrylee <rrylee1994@gmail.com>")
        .about("rebuild pstree using rust")
        .arg(
            Arg::from("<showPgids> -p, --show-pgids 'show process group ids; implies -c'")
                .long_about("Show PIDs.  PIDs are shown as decimal numbers in parentheses after each process name.  -p implicitly disables compaction.")
                .required(false)
                .takes_value(false)
        )
        .arg(
            Arg::from("<PID> 'start at this PID; default is 1 (init)'")
                .required(false)
        )
        .arg(
            Arg::from("<verbose> -v, --verbose 'Enable verbose mode.'")
                .required(false)
                .takes_value(false)
        )
        .get_matches();

    if matches.is_present("verbose") {
        builder.filter_level(LevelFilter::Debug);
    }
    builder.init();

    let show_pgids = matches.is_present("showPgids");
    debug!("show_pgids: {}", show_pgids);

    let pid = match matches.value_of("PID") {
        Some(pid) => {
            if let Ok(pid) = pid.parse::<u32>() {
                pid
            } else {
                std::process::exit(1);
            }
        }
        None => DEFAULT_PID,
    };

    if !cfg!(target_os = "linux") {
        warn!("not support your os");
        std::process::exit(1);
    }

    let process_tree = proc::linux_proc::LinuxProvider::new()
        // .change_proc_path("/Users/rry/Code/go/src/github.com/rrylee/rust-pstree/tests/proc")
        .get_process_tree(pid)
        .unwrap();
    print::print_to_console(process_tree, show_pgids)
}

fn get_logging_prefix(record: &Record) -> String {
    match record.level() {
        Level::Trace => get_logging_prefix_for_level(Level::Trace),
        Level::Debug => get_logging_prefix_for_level(Level::Debug),
        Level::Info => get_logging_prefix_for_level(Level::Info),
        Level::Warn => get_logging_prefix_for_level(Level::Warn),
        Level::Error => get_logging_prefix_for_level(Level::Error),
    }
}

fn get_logging_prefix_for_level(level: Level) -> String {
    match level {
        Level::Trace => "🐭\x1b[0;33m ",
        Level::Debug => "🐱\x1b[0;36m ",
        Level::Info => "🦔 ",
        Level::Warn => "😺\x1b[1;33m ",
        Level::Error => "🙀\x1b[0;31m ",
    }
    .to_owned()
}
