use std::collections::HashMap;
use std::collections::btree_map::ValuesMut;
use std::env;

use super::{CommandType, MyShell, Pipeline, REDIRECTIONS, REDIRECTION_KEYS, SPECIAL_SYMBOLS};
use crate::string_utils::{find_all_start_end_symb, find_all_subshells};

use glob::glob;
use nix::libc::{close, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use nix::unistd::pipe;
use std::fs::File;
use std::os::unix::io::IntoRawFd;

impl MyShell {
    pub fn preprocess_comments(line: &str) -> &str {
        match line.find('#') {
            Some(idx) => &line[0..idx],
            None => line,
        }
    }
    pub fn split_command(line: &str) -> Result<Vec<String>, &str> {
        let one_quote = find_all_start_end_symb(line, "'")?;
        let two_quote = find_all_start_end_symb(line, "\"")?;
        let subshells = find_all_subshells(line)?;

        let mut non_split_regions = one_quote;
        non_split_regions.extend(two_quote);
        non_split_regions.extend(subshells);

        let mut splitted: Vec<String> = Vec::new();
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
                    splitted.push(line[prev..next].trim().to_string());
                }
                next += 1;
                prev = next;
                continue;
            }
            next += 1;
        }
        splitted.push(line[prev..].trim().to_string());
        Ok(splitted)
    }
    pub fn preprocess_pipeline(commands: Vec<String>) -> Result<Pipeline, String> {
        let n_steps = commands.iter().filter(|&command| *command == "|").count() + 1;
        if n_steps == 1 {
            let mut subshell_comm: Vec<HashMap<usize, Vec<(usize, usize)>>> = Vec::new();
            subshell_comm.push(HashMap::new());
            subshell_comm[0].insert(1, Vec::new());
            return Ok(Pipeline {
                steps: Vec::from(vec![commands]),
                ioe_descriptors: vec![[0, 1, 2]],
                types: Vec::from([CommandType::External]),
                subshell_comm,
            });
        }
        let mut steps: Vec<Vec<String>> = Vec::new();
        steps.reserve(n_steps);

        let mut ioe_descriptors: Vec<[i32; 3]> = Vec::new();
        ioe_descriptors.reserve(n_steps);

        let types: Vec<CommandType> = vec![CommandType::External; n_steps];

        let mut subshell_comm: Vec<HashMap<usize, Vec<(usize, usize)>>> = Vec::new();
        subshell_comm.reserve(n_steps);

        steps.push(Vec::new());
        for command in &commands {
            if *command == "|" {
                steps.push(Vec::new());
                continue;
            }
            let last_step_len = steps.len();
            steps[last_step_len - 1].push((*command).clone());
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
                ioe_descriptors.push([STDIN_FILENO, pfds.1, STDERR_FILENO]);
            } else if i == n_steps - 1 {
                ioe_descriptors.push([pfds_prev.0, STDOUT_FILENO, STDERR_FILENO]);
            } else {
                ioe_descriptors.push([pfds_prev.0, pfds.1, STDERR_FILENO]);
            }
            pfds_prev = pfds;
        }

        Ok(Pipeline {
            steps,
            ioe_descriptors,
            types,
            subshell_comm,
        })
    }
    pub fn preprocess_subshells(mut p: Pipeline) -> Result<Pipeline, String> {
        let n_steps = p.steps.len();
        for step_i in 0..n_steps {
            let command = &p.steps[step_i];
            // let subshells: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
            for i in 0..command.len() {
                let subshells = find_all_subshells(&command[i])?;
                if !subshells.is_empty() {
                    p.subshell_comm[step_i].insert(i, subshells);
                }
            }
        }
        Ok(p)
    }
    pub fn preprocess_redirections(mut p: Pipeline) -> Result<Pipeline, String> {
        for step_i in 0..p.steps.len() {
            let command = &mut p.steps[step_i];
            // for &redirection in REDIRECTION_KEYS.iter() {
            for red_i in 0..REDIRECTION_KEYS.len() {
                let mut redirection = REDIRECTION_KEYS[red_i];
                if command.contains(&redirection.to_string()) {
                    // command > file 2>&1
                    if command.contains(&">".to_string()) && command.contains(&"2>&1".to_string()) {
                        // syntax error
                        if command[command.len() - 1] != "2>&1" || command[command.len() - 3] != ">"
                        {
                            return Err("syntax error".to_string());
                        }
                        redirection = "&>";
                        let redir_index = command.len() - 3;
                        command[redir_index] = redirection.to_string();
                        command.pop();
                    }
                    let io_indecies = REDIRECTIONS.get(redirection).unwrap();
                    let filename = command.last().unwrap();

                    let fd: i32;
                    if redirection == "<" {
                        fd = match File::open(filename) {
                            Ok(f) => f.into_raw_fd(),
                            Err(err) => return Err(err.to_string()),
                        }
                    } else {
                        fd = match File::create(filename) {
                            Ok(f) => f.into_raw_fd(),
                            Err(err) => return Err(err.to_string()),
                        }
                    }
                    for &index in io_indecies {
                        let old_desc = p.ioe_descriptors[step_i][index as usize];
                        if old_desc > 2 {
                            unsafe {
                                if close(old_desc) == -1 {
                                    return Err(
                                        "file descriptor close was unsuccsessful".to_string()
                                    );
                                }
                            };
                        }
                        p.ioe_descriptors[step_i][index as usize] = fd;
                    }
                    if command.len() < 3 {
                        return Err("parse error".to_string());
                    }
                    // pop 2 last entries
                    command.pop();
                    command.pop();
                    break;
                }
            }
        }

        Ok(p)
    }
    pub fn substitute_vars_rem_parenth(
        &self,
        command: Result<Vec<String>, String>,
    ) -> Result<Vec<String>, String> {
        let command = command?;
        let mut result: Vec<String> = Vec::new();
        let mut new_token: String = String::new();
        let mut substitue: bool = true;
        let mut from: usize;
        let mut to: usize;

        for token in &command {
            new_token.clear();
            let token_chars: Vec<char> = token.chars().collect();

            let mut i: usize = 0;
            while i < token_chars.len() {
                // escape symbol
                if token_chars[i] == '\\' {
                    if i != token.len() - 1 {
                        new_token.push(token_chars[i + 1]);
                        i += 2;
                        continue;
                    }
                    i += 1;
                // dont substitue variables between '
                } else if token_chars[i] == '\'' {
                    substitue = !substitue;
                // variables substitution
                } else if token_chars[i] == '$'
                    && substitue
                    && (i != token_chars.len() - 1 && token_chars[i + 1] != '(')
                {
                    from = i;
                    to = token_chars.len();
                    for &s in SPECIAL_SYMBOLS.iter() {
                        if let Some(val) = token[from+1..].find(s) {
                            to = val;
                        }
                    } 
                    let varname = &token[from+1..to];
                    if self.local_vars.contains_key(varname) {
                        new_token += self.local_vars.get(varname).unwrap();
                    } else if !env::var(varname).is_err() {
                        new_token += &env::var(varname).unwrap();
                    }
                    i += varname.len();
                } else if token_chars[i] != '"' {
                    new_token.push(token_chars[i]);
                }
                i += 1;
            }
            result.push(new_token.clone());
        }

        Ok(result)
    }

    pub fn expand_globs(command: Result<Vec<String>, String>) -> Result<Vec<String>, String> {
        let mut command = command?;
        let mut result: Vec<String> = vec![command[0].clone()];

        for i in 1..command.len() {
            let entries = match glob(&command[i]) {
                Ok(matches) => matches,
                Err(err) => return Err(err.to_string()),
            };
            let mut matched = false;
            for entry in entries {
                matched = true;
                match entry {
                    Ok(path) => result.push(match path.to_str() {
                        Some(path) => path.to_string(),
                        None => return Err("glob exeption".to_string()),
                    }),
                    Err(err) => return Err(err.to_string()),
                }
            }
            if !matched {
                result.push(command[i].clone());
            }
        }
        command = result;
        Ok(command)
    }

    pub fn insert_myshell(command: Result<Vec<String>, String>) -> Result<Vec<String>, String> {
        let mut command = command?;
        if command.len() == 1 && command[0].ends_with(".msh") {
            command.insert(0, "myshell".to_string());
        }
        Ok(command)
    }

    pub fn substitute_aliases(
        &self,
        command: Result<Vec<String>, String>,
    ) -> Result<Vec<String>, String> {
        unimplemented!(); // TODO: implement
    }

    pub fn mark_command_types(&self, mut p: Pipeline) -> Pipeline {
        for i in 0..p.steps.len() {
            let command = &p.steps[i];
            p.types[i] = if self.internal_cmds.contains(&command[0].as_str()) {
                CommandType::Internal
            } else if command.len() == 1 && command[0].contains('=') {
                CommandType::LocalVar
            } else {
                CommandType::External
            }
        }
        p
    }

}
