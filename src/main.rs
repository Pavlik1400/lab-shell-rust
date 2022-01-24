extern crate argparse;
use argparse::{ArgumentParser, Store, StoreTrue};
use std::process;
use lab3_myshell::myshell;
use myshell::MyShell;

fn main() {
    let mut version = false;
    let mut script = String::from("");
    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("MyShell - bash analog");
        ap.refer(&mut version)
            .add_option(&["-v", "--version"], StoreTrue, "Print program version");
        ap.refer(&mut script)
            .add_option(&["--script"], Store, "Path to the script to execute");
        ap.parse_args_or_exit();
    }

    if version {
        println!("Myshell, - bash, but worse, Rust port version 2.0.0");
        process::exit(0);
    }
    let mut shell = MyShell::new();
    if script.is_empty() {
        process::exit(shell.start_int_shell());
    } else {
        println!("Path to the script: {}", script);
    }
}
