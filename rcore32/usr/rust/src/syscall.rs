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

pub fn sys_read(fd : usize, buf : &mut [u8]) -> i32 {
    sys_call(SyscallId::Read, fd, buf.as_ptr() as usize, buf.len(), 0)
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

enum SyscallId {
    Openat = 56,
    Close = 57,
    Read = 63,
    Write = 64,
    Exit = 93,
}
