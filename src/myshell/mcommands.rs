use super::MyShell;
use std::io::Write;

impl MyShell {
    pub fn merrno(&self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        let (_, fout, ferr) = MyShell::ioe_descriptors_to_files(&ioe_descs);
        if command.len() == 2 {
            if command[1] == "-h" || command[1] == "--help" {
                write!(
                    &fout,
                    "Get status code of last command\n Usage: \n    merrno [-h|--help]\n"
                )
                .expect("Error while writing to stdout fd");
                return 0;
            }
        }
        if command.len() >= 2 {
            write!(&ferr, "merrno: too many arguments\n")
                .expect("Error while writing to stderr fd");
            return 1;
        }
        write!(&fout, "{}", self.last_exit_code.to_string() + "\n").expect("Error while writing to stdout fd");
        return 0;
    }
    pub fn mpwd(&self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mpwd called!");
        return 0;
    }
    pub fn mcd(&self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mcd called!");
        return 0;
    }
    pub fn execute_script(&self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("execute_script called!");
        return 0;
    }
    pub fn mecho(&self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mecho called!");
        return 0;
    }
    pub fn mexport(&self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mexport called!");
        return 0;
    }
    pub fn alias(&self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("alias called!");
        return 0;
    }
    pub fn mexit(&self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mexit called!");
        return 0;
    }

    pub fn set_local_variable(&self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("Set local variable called!");
        return 0;
    }
}
