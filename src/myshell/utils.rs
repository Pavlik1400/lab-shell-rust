use super::MyShell;
use nix::libc::dup2;
use std::fs::File;
use std::os::unix::io::FromRawFd;

impl MyShell {
    pub unsafe fn ioe_descriptors_to_files(descs: &[i32; 3]) -> (File, File, File) {
        return (
            File::from_raw_fd(descs[0]),
            File::from_raw_fd(descs[1]),
            File::from_raw_fd(descs[2]),
        );
    }
}
