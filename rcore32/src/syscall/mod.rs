use crate::context::TrapFrame;
use crate::process::{ process, thread, CPU};
use crate::process;
use core::slice;
use crate::fs::file_handle::FileHandle;

pub unsafe fn from_cstr(s: *const u8) -> &'static str {
    use core::{slice, str};
    let len = (0usize..).find(|&i| *s.add(i) == 0).unwrap();
    str::from_utf8(slice::from_raw_parts(s, len)).unwrap()
}

pub fn syscall(id : usize, args : [usize;3], tf : &mut TrapFrame) -> isize{
    match id {
        SYS_OPENAT => {
            return sys_openat(args[1] as *const u8);
        },
        SYS_CLOSE => {
        },
        SYS_READ => {
            return sys_read(args[0], args[1] as *mut u8, args[2]);
        },
        SYS_WRITE => {
            print!("{}", args[0] as u8 as char);
            return 0;
        },
        SYS_EXIT => {
            sys_exit(args[0]);
        },
        SYS_FORK => {
            sys_fork(tf);
        },
        _ => { 
            panic!("unknown syscall id {}", id);
        },
    };
    return 0;
}

pub const SYS_OPENAT: usize = 56;   // 打开文件
pub const SYS_CLOSE: usize = 57;    // 关闭文件
pub const SYS_READ: usize = 63;
pub const SYS_WRITE: usize = 64;
pub const SYS_EXIT: usize = 93;
pub const SYS_FORK: usize = 220;

fn sys_openat(path: *const u8) -> isize {
    let proc = process();

    let inode = proc.lookup_inode(
        unsafe{ from_cstr(path) }
        ).unwrap(); // 根据传入的路径查找inode

    let fd = proc.get_free_fd();    // 获取一个未分配的fd 
    let file = FileHandle::new(inode);  // 新建一个文件handle
    proc.files.insert(fd, file);    // 记录当前进程打开的文件

    println!("to open fd {}", fd);
    return fd as isize;
}

fn sys_read(fd : usize, base : *mut u8, len : usize) -> isize {
    let proc = process();
    let handle = proc.get_file_handle(fd).unwrap();

    let slice = unsafe { slice::from_raw_parts_mut(base, len) };
    let real_len = handle.read(slice).unwrap(); // 从文件中读取内容
    return real_len as isize;
}

fn sys_exit(code : usize) {
    println!("exit!");
    process::exit(code);
}

fn sys_fork(tf : &mut TrapFrame) -> isize {
    let new_thread = unsafe{ thread().fork(tf) };
    CPU.add_thread(new_thread);
    return 0;
}
