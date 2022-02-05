extern crate libc;
extern crate nix;
extern crate argparse;
extern crate lazy_static;

use argparse::{ArgumentParser, Store, StoreTrue};
use std::process;
use myshell::myshell::MyShell;

fn main() {
    let mut version = false;
    let mut script = String::new();
    let mut server = false;
    let mut port = String::new();
    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("MyShell - bash analog");
        ap.refer(&mut version)
            .add_option(&["-v", "--version"], StoreTrue, "Print program version");
        ap.refer(&mut script)
            .add_option(&["-s", "--script"], Store, "Path to the script to execute");
        ap.refer(&mut server)
            .add_option(&["--server"], StoreTrue, "Start remote server");
        ap.refer(&mut port)
            .add_option(&["-p", "--port"], Store, "Port of started remote server");
        ap.parse_args_or_exit();
    }

    if version {
        println!("Myshell, - bash, but worse, Rust port version 2.0.0");
        process::exit(0);
    }
    let mut shell = MyShell::new();
    if !script.is_empty() {
        if server || !port.is_empty() {
            eprintln!("myshell: Can't use script and server at the same time");
            process::exit(1);
        }
        process::exit(shell.run_script(script));
    } else if server {
        if port.is_empty() {
            eprintln!("Port number is required when starting server");
            process::exit(1);
        }
        process::exit(shell.start_server(port));
    } else if !port.is_empty() {
        eprintln!("--server is required when specifying port");
        process::exit(1);
    } else {
        process::exit(shell.start_int_shell());
    }
}
