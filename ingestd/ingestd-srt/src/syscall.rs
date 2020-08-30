#[derive(Debug, Default)]
#[repr(C)]
pub struct CloneArgs {
    pub flags: u64,
    pub pidfd: u64,
    pub child_tid: u64,
    pub parent_tid: u64,
    pub exit_signal: u64,
    pub stack: u64,
    pub stack_size: u64,
    pub tls: u64,
    pub set_tid: u64,
    pub set_tid_size: u64
}

pub unsafe fn clone3(clone_args: *const CloneArgs) -> libc::c_long {
    libc::syscall(435, clone_args, std::mem::size_of::<CloneArgs>())
}

pub unsafe fn pidfd_send_signal(pidfd: libc::c_int, sig: libc::c_int, siginfo: *const libc::siginfo_t, flags: libc::c_uint) -> libc::c_int {
    libc::syscall(424, pidfd, sig, siginfo, flags) as i32
}
