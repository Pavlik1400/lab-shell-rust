use super::MyShell;

impl MyShell {
    pub fn merrno(command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("merrno called!");
        return 0;
    }
    pub fn mpwd(command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mpwd called!");
        return 0;
    }
    pub fn mcd(command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mcd called!");
        return 0;
    }
    pub fn execute_script(command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("execute_script called!");
        return 0;
    }
    pub fn mecho(command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mecho called!");
        return 0;
    }
    pub fn mexport(command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mexport called!");
        return 0;
    }
    pub fn alias(command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("alias called!");
        return 0;
    }
    pub fn mexit(command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("mexit called!");
        return 0;
    }

    pub fn set_local_variable(&self, command: &Vec<String>, ioe_descs: [i32; 3]) -> i32 {
        println!("Set local variable called!");
        return 0;
    }
}
