use crate::proc::Process;
use std::fmt::Write;

pub fn print_to_console(process: Process, show_pgids: bool) {
    let mut buf = String::new();
    let prefixs: Vec<PrefixFragment> = Vec::new();
    dfs_write(&mut buf, &process, show_pgids, 0, 0, 0, &prefixs);
    println!("{}", buf)
}

#[derive(Clone)]
enum PrefixFragment {
    BlankSpace(usize),
    VerticalLine,
}

fn dfs_write(
    buf: &mut String,
    process: &Process,
    show_pgids: bool,
    depth: usize,
    total_child: usize,
    child_index: usize,
    prefixs: &Vec<PrefixFragment>,
) {
    let mut prefixs = prefixs.clone();
    let display_name = get_name(&process, show_pgids);
    if depth == 0 {
        write!(buf, "{}", display_name).unwrap();
        prefixs.push(PrefixFragment::BlankSpace(display_name.chars().count() + 1));
    } else {
        if child_index == 0 {
            let mut via = "┬";
            if total_child == 1 {
                via = "─";
            }
            write!(buf, "─{}─{}", via, display_name).unwrap();
        } else {
            let mut via = "├";
            if child_index + 1 == total_child {
                via = "└";
            }
            write!(
                buf,
                "\n{}{}─{}",
                generate_prefix(&prefixs),
                via,
                display_name
            )
            .unwrap();
        }
        if total_child == 1 {
            prefixs.push(PrefixFragment::BlankSpace(display_name.chars().count() + 3));
        } else {
            prefixs.push(PrefixFragment::VerticalLine);
            prefixs.push(PrefixFragment::BlankSpace(display_name.chars().count() + 2));
        }
    }

    let mut index = 0;
    let total_child = process.children().len();
    for child_process in process.children() {
        dfs_write(
            buf,
            child_process,
            show_pgids,
            depth + 1,
            total_child,
            index,
            &prefixs,
        );
        index += 1;
    }
}

fn generate_prefix(prefixs: &Vec<PrefixFragment>) -> String {
    let mut ret = String::new();
    for fragment in prefixs {
        write!(
            ret,
            "{}",
            match fragment {
                PrefixFragment::BlankSpace(i) => " ".repeat(i.to_owned()),
                PrefixFragment::VerticalLine => "│".to_string(),
            },
        )
        .unwrap();
    }
    ret
}

fn get_name(process: &Process, show_pgids: bool) -> String {
    if process.pid() == 0 {
        return process.name().to_string();
    }
    if show_pgids {
        format!("{}({})", process.name(), process.pid())
    } else {
        format!("{}", process.name())
    }
}
