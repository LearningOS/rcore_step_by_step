use crate::context::TrapFrame;
use crate::process::process;
use core::str::from_utf8;
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
            let proc = process();

            let inode = proc.lookup_inode(
                unsafe{ from_cstr(args[1] as *const u8) }
                ).unwrap(); // 根据传入的路径查找inode

            let fd = proc.get_free_fd();    // 获取一个未分配的fd 
            let file = FileHandle::new(inode);  // 新建一个文件handle
            proc.files.insert(fd, file);    // 记录当前进程打开的文件

            println!("to open fd {}", fd);
            return fd as isize;
        },
        SYS_CLOSE => {
        },
        SYS_READ => {
            let proc = process();
            let handle = proc.get_file_handle(args[0]).unwrap();

            let slice = unsafe { slice::from_raw_parts_mut(args[1] as *mut u8, args[2]) };
            let len = handle.read(slice).unwrap(); // 从文件中读取内容
            return len as isize;
        },
        SYS_WRITE => {
            print!("{}", args[0] as u8 as char);
            return 0;
        },
        _ => { 
            panic!("unknown syscall");
        },
    };
    return 0;
}

pub const SYS_OPENAT: usize = 56;   // 打开文件
pub const SYS_CLOSE: usize = 57;    // 关闭文件
pub const SYS_READ: usize = 63;
pub const SYS_WRITE: usize = 64;

//fn sys_openat(fd : usize, path: *const u8) -> isize {
//}
