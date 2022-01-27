use super::MyShell;
use libc::{STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO};
use nix::libc::dup;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::io::Write;

impl MyShell {
    pub unsafe fn ioe_descriptors_to_files(descs: &[i32; 3]) -> (File, File, File) {
        let in_ = if descs[0] == STDIN_FILENO { dup(STDIN_FILENO) } else  { descs[0] };
        let out_ = if descs[1] == STDOUT_FILENO { dup(STDOUT_FILENO) } else  { descs[1] };
        let err_ = if descs[2] == STDERR_FILENO { dup(STDERR_FILENO) } else  { descs[2] };
        return (
            File::from_raw_fd(in_),
            File::from_raw_fd(out_),
            File::from_raw_fd(err_),
        );
    }

    pub fn writex(mut f: &File, message: &str) {
        write!(f, "{}", message).expect("Failed to write in the internal command");
    }
}
