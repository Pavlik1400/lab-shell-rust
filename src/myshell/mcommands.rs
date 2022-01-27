use super::MyShell;
use std::io::Write;

impl MyShell {
    pub fn merrno(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        // let (_, fout, ferr) = MyShell::ioe_descriptors_to_files(&ioe_descs);
        let (_, fout, ferr) = MyShell::ioe_descriptors_to_files(&ioe_descs);
        if command.len() == 2 {
            if command[1] == "-h" || command[1] == "--help" {
                write!(
                    fout,
                    "Get status code of last command\n Usage: \n    merrno [-h|--help]\n"
                )
                .expect("Error while writing to stdout fd");
                return 0;
            }
        }
        if command.len() >= 2 {
            write!(ferr, "merrno: too many arguments\n")
                .expect("Error while writing to stderr fd");
            return 1;
        }
        write!(fout, "{}", self.last_exit_code.to_string() + "\n").expect("Error while writing to stdout fd");
        return 0;
    }
    pub fn mpwd(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        // let (_, fout, ferr) = MyShell::ioe_descriptors_to_files(&ioe_descs);
        println!("mpwd called!");
        return 0;
    }
    pub fn mcd(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mcd called!");
        return 0;
    }
    pub fn execute_script(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("execute_script called!");
        return 0;
    }
    pub fn mecho(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mecho called!");
        return 0;
    }
    pub fn mexport(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mexport called!");
        return 0;
    }
    pub fn alias(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("alias called!");
        return 0;
    }
    pub fn mexit(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mexit called!");
        return 0;
    }

    pub fn set_local_variable(&mut self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("Set local variable called!");
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
