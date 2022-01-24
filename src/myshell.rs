mod preprocessing;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;
use std::{collections::HashMap, process};

pub struct MyShell {
    time_to_exit: bool,
    aliases: HashMap<String, String>,
    local_vars: HashMap<String, String>,
    pub exec_path: String,
    pub last_exit_code: i32,
    special_symbols: Vec<char>,
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
        loop {
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

    fn interpret_line(&self, line: &mut str) -> i32 {
        println!("Before everything:            {:?}", line);
        let line = MyShell::preprocess_comments(line);
        println!("After comments preprocessing: {:?}", line);
        let line = line.trim();
        println!("After trim:                   {:?}", line);

        if line.is_empty() {
            return 0;
        }

        let line = match MyShell::split_command(line) {
            Ok(splitted) => splitted,
            Err(err) => {
                eprintln!("{}", err);
                return 1;
            }
        };
        println!("After split:                  {:?}", line);

        return 0;
    }
}