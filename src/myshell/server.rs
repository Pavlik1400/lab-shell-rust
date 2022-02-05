use super::MyShell;
use argparse::Parse;
use nix::sys::wait::wait;
use nix::unistd::ForkResult::{Child, Parent};
use nix::unistd::{fork, getpid, getppid};
use std::net::{TcpListener, TcpStream};
use std::process;
use std::os::unix::io::{AsRawFd, RawFd};


const LOCALHOST: &'static str = "127.0.0.1";

impl MyShell {
    pub fn start_server(&mut self, port: String) -> i32 {
        let address = String::new() + LOCALHOST + ":" + &port;
        let listener = match TcpListener::bind(&address) {
            Ok(l) => l,
            Err(err) => {
                eprintln!("myshell: {:?}", err);
                return 1;
            }
        };
        println!("Start server at {}", address);
        loop {
            match listener.accept() {
                Ok((client_stream, client_addr)) => {
                    println!("Connection from {:?} accepted", client_addr);
                    unsafe {
                        let pid = fork();
                        match pid.expect("myshell: fork() failed") {
                            Child => {
                                println!(
                                    "Hello from child process with pid: {} and parent pid:{}",
                                    getpid(),
                                    getppid()
                                );
                                process::exit(MyShell::start_remote_interpreter(client_stream));
                            }
                            Parent { child } => {
                                println!(
                                    "Hello from parent process with pid: {} and child pid:{}",
                                    getpid(),
                                    child
                                );
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    return 1;
                }
            }
        }
        return 0;
    }

    pub fn start_remote_interpreter(mut client_stream: TcpStream) -> i32 {
        client_stream.as_raw_fd();
        return 0;
    }
}
