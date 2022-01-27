mod mcommands;
mod preprocessing;
mod utils;

use lazy_static::lazy_static;
use nix::libc::{strerror, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;
use std::fs::File;
use std::os::unix::prelude::FromRawFd;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::{collections::HashMap, process};

lazy_static! {
    pub static ref REDIRECTION_KEYS: Vec<&'static str> = vec!["2>", "&>", ">&", "<", ">"];
    pub static ref REDIRECTIONS: HashMap<&'static str, Vec<i32>> = {
        let mut m = HashMap::new();
        m.insert(">", vec![STDOUT_FILENO]);
        m.insert("2>", vec![STDERR_FILENO]);
        m.insert("&>", vec![STDOUT_FILENO, STDERR_FILENO]);
        m.insert("<", vec![STDIN_FILENO]);
        m.insert(">&", vec![STDOUT_FILENO, STDERR_FILENO]);
        m
    };
}

#[derive(Clone, Debug, PartialEq)]
enum CommandType {
    Internal,
    External,
    LocalVar,
}

pub struct MyShell {
    time_to_exit: bool,
    aliases: HashMap<String, String>,
    local_vars: HashMap<String, String>,
    pub exec_path: String,
    pub last_exit_code: i32,
    special_symbols: Vec<char>,
    internal_cmds: Vec<&'static str>,
}

pub struct Pipeline {
    steps: Vec<Vec<String>>,
    ioe_descriptors: Vec<[i32; 3]>,
    types: Vec<CommandType>,
    subshell_comm: Vec<HashMap<usize, Vec<(usize, usize)>>>,
}

impl MyShell {
    pub fn new() -> MyShell {
        let time_to_exit = false;
        let aliases = HashMap::new();
        let local_vars = HashMap::new();

        let exec_path = std::env::current_exe()
            .unwrap_or_else(|error| {
                eprintln!(
                    "myshell: Error: could not determine path to the executable: {}",
                    error
                );
                process::exit(1);
            })
            .into_os_string()
            .into_string()
            .unwrap();
        let last_exit_code = 0;
        let special_symbols = vec!['$', ' ', '\'', '"'];

        let internal_cmds: Vec<&'static str> = vec![
            "merrno", "mpwd", "mcd", ".", "mecho", "mexport", "alias", "mexit",
        ];
        // add exec_path to path if needed
        match env::var("PATH") {
            Ok(val) => {
                let new_path = val + ":" + &exec_path;
                env::set_var("PATH", new_path);
            }
            Err(_) => {
                eprintln!("myshell: Error: PATH is unset");
                process::exit(1);
            }
        }
        MyShell {
            time_to_exit,
            aliases,
            local_vars,
            exec_path,
            last_exit_code,
            special_symbols,
            internal_cmds,
        }
    }

    pub fn start_int_shell(&mut self) -> i32 {
        // `()` can be used when no completer is required
        let mut rl = Editor::<()>::new();

        let home_path = env::var("HOME").unwrap_or_else(|_| {
            eprintln!("Error: HOME variable is unset");
            process::exit(1);
        });
        let history_filename = home_path + "/.myshell_history";

        if rl.load_history(&history_filename).is_err() {
            std::fs::File::create(&history_filename).unwrap_or_else(|err| {
                eprintln!("Error: could not create history file: {}", err);
                process::exit(1);
            });
        }
        while !self.time_to_exit {
            // pwd
            let curdir = env::current_dir()
                .unwrap_or_else(|err| {
                    println!("Error: could not read current directory: {}", err);
                    process::exit(1);
                })
                .into_os_string()
                .into_string()
                .unwrap();

            // read input
            let readline = rl.readline(&(curdir + " $ "));
            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str());
                    let mut line = String::from(line.trim());
                    if line.is_empty() {
                        continue;
                    }
                    self.last_exit_code = self.interpret_line(&mut line);
                }
                Err(ReadlineError::Interrupted) => {
                    self.last_exit_code = 0;
                    break;
                }
                Err(ReadlineError::Eof) => {
                    self.last_exit_code = 0;
                    break;
                }
                Err(err) => {
                    eprintln!("myshell: Error: {:?}", err);
                    self.last_exit_code = 1;
                    break;
                }
            }
        }
        rl.save_history(&history_filename).unwrap_or_else(|err| {
            println!("Warning: could not save history file: {}", err);
        });
        self.last_exit_code
    }

    fn interpret_line(&mut self, line: &mut str) -> i32 {
        let line = MyShell::preprocess_comments(line);
        let line = line.trim();

        if line.is_empty() {
            return 0;
        }

        let line = match MyShell::split_command(line) {
            Ok(splitted) => splitted,
            Err(err) => {
                eprintln!("myshell: {}", err);
                return 1;
            }
        };

        let line = match MyShell::preprocess_pipeline(line) {
            Ok(l) => l,
            Err(err) => {
                let err: i32 = err.parse().unwrap();
                unsafe {
                    eprintln!("myshell: {:?}", strerror(err));
                }
                return 1;
            }
        };

        let line = match MyShell::preprocess_subshells(line) {
            Ok(l) => l,
            Err(err) => {
                eprint!("myshell: {}", err);
                return 1;
            }
        };

        let mut line = match MyShell::preprocess_redirections(line) {
            Ok(l) => l,
            Err(err) => {
                eprintln!("myshell: {}", err);
                return 1;
            }
        };
        for step in &line.steps {
            if step.is_empty() {
                eprintln!("myshell: syntax error");
                return 1;
            }
        }

        for i in 0..line.steps.len() {
            // TODO: add variable substitution & subshell search
            line.steps[i] =
                match MyShell::insert_myshell(MyShell::expand_globs(Ok(line.steps[i].clone()))) {
                    Ok(val) => val,
                    Err(err) => {
                        eprintln!("myshell: {}", err);
                        return 1;
                    }
                }
        }
        let line = self.mark_command_types(line);
        self.execute_pipeline(line)
    }

    pub fn execute_pipeline(&mut self, mut p: Pipeline) -> i32 {
        let path = match env::var("PATH") {
            Ok(val) => val,
            Err(err) => {
                eprintln!("myshell: {}", err.to_string());
                return 1;
            }
        };
        let mut path: Vec<&str> = path.split(":").collect();
        path.push("");

        let n_steps = p.steps.len();
        let mut childs: Vec<Child> = Vec::new();
        let mut statuses: Vec<i32> = vec![0; n_steps];

        #[cfg(debug_assertions)]
        {
            // for i in 0..n_steps {
            //     if !p.subshell_comm[i].is_empty() {
            //         println!("Found subshell in part {}: ", i);
            //         for (key, val) in &p.subshell_comm[i] {
            //             print!("    {}: [", key);
            //             for (s, e) in val {
            //                 println!("{{{}, {}}}, ", *s, *e);
            //             }
            //             println!("]")
            //         }
            //     }
            // }
            println!("n_steps = {}", n_steps);
            println!("command types: {:?}", p.types);
            for i in 0..n_steps {
                println!(
                    "step: {}: {:?}; ioe descriptors: {:?}",
                    i, p.steps[i], p.ioe_descriptors[i],
                );
            }
        }
        // run all external first to make sure that write to pipe from internal later is not blocking execution
        for step_i in 0..n_steps {
            if p.types[step_i] == CommandType::External {
                let command = &mut p.steps[step_i];
                let mut background = false;
                if command.last().unwrap() == "&" {
                    background = true;
                    command.pop();
                }
                // TODO: Add subshell processing
                let mut found_binary = false;
                for &subpath in &path {
                    let bin_path = String::from(subpath) + "/" + &command[0];
                    if Path::new(&bin_path).exists() {
                        found_binary = true;
                        let descs = &p.ioe_descriptors[step_i];
                        let (in_, out_, err_) = unsafe {
                            (
                                if descs[0] != STDIN_FILENO {
                                    Stdio::from_raw_fd(descs[0])
                                } else {
                                    Stdio::inherit()
                                },
                                if descs[1] != STDOUT_FILENO {
                                    Stdio::from_raw_fd(descs[1])
                                } else {
                                    Stdio::inherit()
                                },
                                if descs[2] != STDERR_FILENO {
                                    Stdio::from_raw_fd(descs[2])
                                } else {
                                    Stdio::inherit()
                                },
                            )
                        };
                        let child = match Command::new(bin_path)
                            .args(&command[1..])
                            .stdin(in_)
                            .stdout(out_)
                            .stderr(err_)
                            .spawn()
                        {
                            Ok(c) => c,
                            Err(err) => {
                                eprintln!("myshell: {}", err.to_string());
                                process::exit(1);
                            }
                        };
                        if !background {
                            childs.push(child);
                        }

                        break;
                    }
                }
                if !found_binary {
                    eprintln!("myshell: command not found: {}", &command[0]);
                    statuses[step_i] = 127;
                }
            }
        }
        // now run all internal
        for step_i in 0..n_steps {
            let command = &mut p.steps[step_i];
            // TODO: subshell
            if p.types[step_i] == CommandType::Internal {
                if command.last().unwrap() == "&" {
                    command.pop();
                }
                let status = self.call_mcommand(command, p.ioe_descriptors[step_i]);
                statuses[step_i] = status;
            } else if p.types[step_i] == CommandType::LocalVar {
                let status = self.set_local_variable(command, p.ioe_descriptors[step_i]);
                statuses[step_i] = status;
            }
        }

        for mut child in childs {
            child.wait().expect("Could not wait for child");
        }

        // check if everybody  finished successfully
        for step_i in 0..n_steps {
            if statuses[step_i] != 0 {
                return statuses[step_i];
            }
        }
        return 0;
    }
}
