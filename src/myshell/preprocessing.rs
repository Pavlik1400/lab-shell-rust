use std::collections::HashMap;

use super::{pipeline, CommandType, MyShell};
use crate::string_utils::{find_all_start_end_symb, find_all_subshells};

use std::fs::File;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::process::{Command, Stdio};

use nix::libc::{strerror, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use nix::unistd::pipe;

impl MyShell {
    pub fn preprocess_comments(line: &str) -> &str {
        match line.find('#') {
            Some(idx) => &line[0..idx],
            None => line,
        }
    }

    pub fn split_command(line: &str) -> Result<Vec<&str>, &str> {
        let one_quote = find_all_start_end_symb(line, "'")?;
        let two_quote = find_all_start_end_symb(line, "\"")?;
        let subshells = find_all_subshells(line)?;

        let mut non_split_regions = one_quote;
        non_split_regions.extend(two_quote);
        non_split_regions.extend(subshells);

        let mut splitted: Vec<&str> = Vec::new();
        let mut next: usize = 0;
        let mut prev: usize = 0;
        for ci in 0..line.len() {
            let c = line.as_bytes()[ci] as char;
            let mut can_split = true;
            for (s, e) in &non_split_regions {
                if (*s <= ci) && (ci <= *e) {
                    can_split = false;
                }
            }
            if can_split && c == ' ' {
                if prev != next {
                    splitted.push(&line[prev..next].trim());
                }
                next += 1;
                prev = next;
                continue;
            }
            next += 1;
        }
        splitted.push(&line[prev..]);
        Ok(splitted)
    }

    pub fn preprocess_pipeline(commands: Vec<&str>) -> Result<pipeline, String> {
        let n_steps = commands.iter().filter(|&command| *command == "|").count();
        if n_steps == 1 {
            let mut subshell_comm: Vec<HashMap<usize, Vec<(usize, usize)>>> = Vec::new();
            subshell_comm.push(HashMap::new());
            subshell_comm[0].insert(1, Vec::new());
            return Ok(pipeline {
                steps: Vec::from(vec![commands]),
                ioe_descriptors: vec![(0, 1, 2)],
                types: Vec::from([CommandType::External]),
                subshell_comm,
            });
        }
        let mut steps: Vec<Vec<&str>> = Vec::new();
        steps.reserve(n_steps);

        let mut ioe_descriptors: Vec<(i32, i32, i32)> = Vec::new();
        ioe_descriptors.reserve(n_steps);

        let types: Vec<CommandType> = vec![CommandType::External; n_steps];

        let mut subshell_comm: Vec<HashMap<usize, Vec<(usize, usize)>>> = Vec::new();
        subshell_comm.reserve(n_steps);

        steps.push(Vec::new());
        for command in &commands {
            if *command == "|" {
                steps.push(Vec::new());
            }
            let last_step_len = steps.len();
            steps[last_step_len - 1].push(*command);
        }

        let mut pfds: (i32, i32) = (0, 1);
        let mut pfds_prev: (i32, i32) = (0, 1);
        for i in 0..n_steps {
            if i != n_steps - 1 {
                pfds = match pipe() {
                    Ok(fds) => fds,
                    Err(err) => return Err(err.to_string()),
                };
            }
            if i == 0 {
                ioe_descriptors.push((STDIN_FILENO, pfds.0, STDERR_FILENO));
            } else if i == n_steps - 1 {
                ioe_descriptors.push((pfds_prev.0, STDOUT_FILENO, STDERR_FILENO));
            } else {
                ioe_descriptors.push((pfds_prev.0, pfds.1, STDERR_FILENO));
            }
            pfds_prev = pfds;
        }

        Ok(pipeline {
            steps,
            ioe_descriptors,
            types,
            subshell_comm,
        })
    }
    pub fn preprocess_subshells(mut p: pipeline) -> Result<pipeline, &str> {
        let n_steps = p.steps.len();
        for step_i in 0..n_steps {
            let command = &p.steps[step_i];
            // let subshells: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
            for i in 0..command.len() {
                let subshells = find_all_subshells(command[i])?;
                if !subshells.is_empty() {
                    p.subshell_comm[step_i].insert(i, subshells);
                }
            }
        }
        Ok(p)
    }
}
