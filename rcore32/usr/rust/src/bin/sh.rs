#![no_std]
#![no_main]
#![feature(alloc)]

extern crate alloc;
#[macro_use]
extern crate rcore32_user;

use alloc::vec::Vec;
use core::ptr;

use rcore32_user::io::get_line;
use rcore32_user::syscall::{sys_exec};

// IMPORTANT: Must define main() like this
#[no_mangle]
pub fn main() -> i32 {
    println!("Rust user shell");
    let mut history = Vec::new();

    loop {
        //print!("{}> ", sys_getcwd());
        print!(">> ");
        let cmd = get_line(&mut history);
        // split cmd, make argc & argv
        // to-do: handle quotes
        let cmd = cmd.replace(' ', "\0") + "\0";
        let cmds: Vec<&str> = cmd.split('\0').collect();
        let mut ptrs: Vec<usize> = cmd
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(|s| s.as_ptr() as usize)
            .collect();
        if ptrs.is_empty() {
            continue;
        }

        sys_exec(ptrs[0] as *const u8);

        //if cmds.len() == 3 {
            //// handle cd
            //if cmds[0] == "cd" {
                //sys_chdir(cmds[1]);
                //continue;
            //}
        //}

        //if sys_access(cmds[0]) == 0 {
            //ptrs.push(0); // indicate the end of argv

            //let pid = sys_vfork();
            //assert!(pid >= 0);
            //if pid == 0 {
                //return sys_exec(
                    //ptrs[0] as *const u8,
                    //ptrs.as_ptr() as *const *const u8,
                    //ptr::null(),
                //);
            //} else {
                //let mut code: i32 = 0;
                //sys_wait(pid as usize, &mut code);
                //println!("\n[Process exited with code {}]", code);
            //}
        //} else {
            //println!("\n[Command {} not found]", cmds[0]);
        //}
    }
}
