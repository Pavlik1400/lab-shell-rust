use super::MyShell;
use std::collections::btree_map::ValuesMut;
use std::fs;
use std::io::{Write, BufReader, BufRead};
use std::path::Path;
use std::{env, fs::File, os::unix::prelude::FromRawFd, process};

impl MyShell {
    pub fn merrno(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        let (_, fout, ferr) = unsafe { MyShell::ioe_descriptors_to_files(&ioe_descs) };
        if command.len() == 2 {
            if command[1] == "-h" || command[1] == "--help" {
                MyShell::writex(
                    &fout,
                    "Get status code of last command\n Usage: \n    merrno [-h|--help]\n",
                );
                return 0;
            }
        }
        if command.len() >= 2 {
            MyShell::writex(&ferr, "merrno: too many arguments\n");
            return 1;
        }
        MyShell::writex(&fout, &(self.last_exit_code.to_string() + "\n"));
        return 0;
    }
    pub fn mpwd(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        let (_, fout, ferr) = unsafe { MyShell::ioe_descriptors_to_files(&ioe_descs) };
        if command.len() == 2 {
            if command[1] == "-h" || command[1] == "--help" {
                MyShell::writex(
                    &fout,
                    "Get current directory\n Usage: \n    mpwd [-h|--help]\n",
                );
                return 0;
            }
        }
        if command.len() >= 2 {
            MyShell::writex(&ferr, "mpwd: too many arguments\n");
            return 1;
        }
        let curdir = env::current_dir()
            .unwrap_or_else(|err| {
                println!("mpwd: could not read current directory: {}", err);
                process::exit(1);
            })
            .into_os_string()
            .into_string()
            .unwrap();
        // TODO: explore why
        println!("{}", curdir);
        // MyShell::writex(&fout, &curdir);
        return 0;
    }
    pub fn mcd(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        let (_, fout, ferr) = unsafe { MyShell::ioe_descriptors_to_files(&ioe_descs) };
        if command.len() > 2 {
            MyShell::writex(&ferr, "mcd: too many arguments\n");
            return 1;
        }
        let mut cd_path: String = String::from("");
        if command.len() == 1 || command[1] == "~" {
            cd_path = match env::var("HOME") {
                Ok(val) => val,
                Err(err) => {
                    MyShell::writex(&ferr, &format!("mcd: {}\n", err.to_string()));
                    return 2;
                }
            }
        }
        if command.len() == 2 && command[1] != "~" {
            if command[1] == "-h" || command[1] == "--help" {
                MyShell::writex(
                    &fout,
                    "Change directory\n Usage: \n    mcd <directory=~> [-h|--help]\n",
                );
            }
            cd_path = command[1].clone();
        }
        match env::set_current_dir(&cd_path) {
            Ok(_) => return 0,
            Err(err) => {
                MyShell::writex(&ferr, &format!("mcd: {}\n", err.to_string()));
                return 3;
            }
        }
    }
    pub fn execute_script(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        let (_, _, ferr) = unsafe { MyShell::ioe_descriptors_to_files(&ioe_descs) };
        if command.len() != 2 {
            MyShell::writex(&ferr, ".: bad number of arguments");
            return 1;
        }

        let file = match File::open(&command[1]) {
            Ok(f) => f,
            Err(err) => {
                MyShell::writex(&ferr, &format!(".: {}", err.to_string()));
                return 2;
            }
        };
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let mut line = match line {
                Ok(l) => l,
                Err(err) => {
                    eprintln!("myshell: {}", err.to_string());
                    return 1;
                }
            };
            self.last_exit_code = self.interpret_line(&mut line);
            if self.time_to_exit {
                break;
            }
        }
        return self.last_exit_code;
    }
    pub fn mecho(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        let (_, fout, ferr) = unsafe { MyShell::ioe_descriptors_to_files(&ioe_descs) };
        if command.len() == 2 {
            if command[1] == "-h" || command[1] == "--help" {
                MyShell::writex(&fout, "Print text and substite variables\n    Usage: mecho [-h|--help] [text|$<var_name>] ...\n");
                return 0;
            }
        }
        let mut output: String = String::new();
        for i in 1..command.len() {
            if i != 1 {
                output += " ";
            }
            output += &command[i];
        }
        output += "\n";
        MyShell::writex(&fout, &output);
        return 0;
    }
    pub fn mexport(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        let (_, _, ferr) = unsafe { MyShell::ioe_descriptors_to_files(&ioe_descs) };
        if command.len() != 2 {
            MyShell::writex(&ferr, "mexport: bad number of arguments\n");
            return 1;
        }
        let splitted: Vec<&str> = command[1].split("=").collect();
        if splitted.len() != 2 {
            MyShell::writex(&ferr, "mexport: syntax error\n");
            return 2;
        }
        env::set_var(splitted[0], splitted[1]);
        return 1;
    }
    pub fn alias(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("alias called!");
        return 0;
    }
    pub fn mexit(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        let (_, fout, ferr) = unsafe { MyShell::ioe_descriptors_to_files(&ioe_descs) };
        let mut status: i32 = 0;
        if command.len() == 2 {
            if command[1] == "-h" || command[1] == "--help" {
                MyShell::writex(
                    &fout,
                    "Close current session\nUsage: \n    mexit <code=0> [-h|--help]\n",
                );
                return 0;
            }
            status = match command[1].parse() {
                Ok(val) => val,
                Err(_) => {
                    MyShell::writex(&ferr, "mexit: exit status is not a number\n");
                    return 1;
                }
            };
        }
        if command.len() > 2 {
            MyShell::writex(&ferr, "mexit: too many arguments\n");
            return 2;
        }
        self.time_to_exit = true;
        return status;
    }
    pub fn set_local_variable(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        let (_, fout, ferr) = unsafe { MyShell::ioe_descriptors_to_files(&ioe_descs) };
        let splitted: Vec<&str> = command[0].split("=").collect();
        if splitted.len() != 2 {
            MyShell::writex(&fout, "myshell: syntax error\n");
            return 1;
        }
        self.local_vars.insert(splitted[0].to_string(), splitted[1].to_string());
        return 0;
    }
    pub fn call_mcommand(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        // TODO:: look awful
        if command[0] == "merrno" {
            return self.merrno(command, ioe_descs);
        } else if command[0] == "mpwd" {
            return self.mpwd(command, ioe_descs);
        } else if command[0] == "mcd" {
            return self.mcd(command, ioe_descs);
        } else if command[0] == "." {
            return self.execute_script(command, ioe_descs);
        } else if command[0] == "mecho" {
            return self.mecho(command, ioe_descs);
        } else if command[0] == "mexport" {
            return self.mexport(command, ioe_descs);
        } else if command[0] == "alias" {
            return self.alias(command, ioe_descs);
        } else if command[0] == "mexit" {
            return self.mexit(command, ioe_descs);
        }
        return 0;
    }
}
