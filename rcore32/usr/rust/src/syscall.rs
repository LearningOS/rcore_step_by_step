#[inline(always)]
fn sys_call(
    syscall_id: SyscallId,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
) -> i32 {
    let id = syscall_id as usize;
    let mut ret: i32;
    unsafe {
        #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
        asm!("ecall"
            : "={x10}" (ret)
            : "{x17}" (id), "{x10}" (arg0), "{x11}" (arg1), "{x12}" (arg2), "{x13}" (arg3)
            : "memory"
            : "volatile");
    }
    ret
}

pub fn sys_write(ch : u8) -> i32 {
    sys_call(SyscallId::Write, ch as usize, 0, 0, 0)
}

pub fn sys_read(fd : usize, base : *const u8, len : usize) -> i32 {
    sys_call(SyscallId::Read, fd, base as usize , len , 0)
}

pub fn sys_open(path: &str, flags: usize) -> i32 {
    // UNSAFE: append '\0' to the string
    use core::mem::replace;
    let end = unsafe { &mut *(path.as_ptr().offset(path.len() as isize) as *mut u8) };
    let backup = replace(end, 0);
    const AT_FDCWD: isize = -100;
    let ret = sys_call(
        SyscallId::Openat,
        AT_FDCWD as usize,
        path.as_ptr() as usize,
        0,
        0
    );
    *end = backup;
    ret
}

pub fn sys_close(fd: usize) -> i32 {
    sys_call(SyscallId::Close, fd, 0, 0, 0)
}


pub fn sys_exit(code: usize) -> ! {
    sys_call(SyscallId::Exit, code, 0, 0, 0);
    loop{}
}

pub fn sys_fork() -> i32 {
    sys_call(SyscallId::Fork, 0, 0, 0, 0)
}

pub fn sys_exec(path : *const u8) {
    sys_call(SyscallId::Exec, path as usize, 0, 0, 0);
}

pub fn sys_getpid() -> i32{
    sys_call(SyscallId::GetPid, 0, 0, 0, 0)
}

pub fn sys_sleep(time : usize) -> i32{
    sys_call(SyscallId::Sleep, time, 0, 0, 0)
}

enum SyscallId {
    Sleep = 35,
    Openat = 56,
    Close = 57,
    Read = 63,
    Write = 64,
    Exit = 93,
    GetPid = 172,
    Fork = 220,
    Exec = 221,
}
