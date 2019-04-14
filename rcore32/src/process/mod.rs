mod interrupt;
mod thread_pool;
mod timer;
mod structs;
mod scheduler;
mod processor;

use self::structs::{ Thread, Process};
use self::thread_pool::ThreadPool;
use self::scheduler::RRScheduler;
use self::processor::Processor;
use alloc::{boxed::Box, vec::Vec, string::String, sync::Arc};

static CPU : Processor = Processor::new();

extern "C" {
    fn _user_program_start();
    fn _user_program_end();
}

pub fn init() {
    println!("+------ now to initialize process ------+");
    let scheduler = RRScheduler::new(50);
    let thread_pool = ThreadPool::new(100, scheduler);
    unsafe{
        CPU.init(Thread::new_init(), Box::new(thread_pool));
    }
    //let thread0 = unsafe{ Thread::new_kernel(hello_thread, 0) };
    //CPU.add_thread(thread0);
    //let thread1 = unsafe{ Thread::new_kernel(hello_thread, 1) };
    //CPU.add_thread(thread1);
    //let thread2 = unsafe{ Thread::new_kernel(hello_thread, 2) };
    //CPU.add_thread(thread2);
    //let thread3 = unsafe{ Thread::new_kernel(hello_thread, 3) };
    //CPU.add_thread(thread3);
    //let thread4 = unsafe{ Thread::new_kernel(hello_thread, 4) };
    //CPU.add_thread(thread4);
    //println!("the user img from {:#x} to {:#x}", 
             //_user_img_start as usize, _user_img_end as usize);

    let data = unsafe{
        ::core::slice::from_raw_parts(
            _user_program_start as *const u8,
            _user_program_end as usize - _user_program_start as usize,
        )
    };
    let user = unsafe{ Thread::new_user(data) };

    let shell_thread = unsafe{ Thread::new_kernel(hello_thread, 4) };
    //CPU.add_thread(shell_thread);
    CPU.add_thread(user);
    CPU.run();
}

#[no_mangle]
pub extern "C" fn hello_thread(_arg : usize) -> ! {
    loop{
        println!("i wake up!");
        sleep(100);
    }
}

pub fn process() -> &'static mut Box<Process> {
    use core::mem::transmute;
    let process: &mut Thread = CPU.context();
    process.proc.as_mut().unwrap()
}



pub fn sleep(time : usize) {
    CPU.sleep(time);
}

pub struct KernelStack(usize);
const STACK_SIZE : usize = 0x8000;

impl KernelStack {
    pub fn new() -> Self{
        use alloc::alloc::{alloc, Layout};
        let bottom =
            unsafe { alloc(Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap()) } as usize;
        KernelStack(bottom)
    }

    fn top(&self) -> usize {
        self.0 + STACK_SIZE
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        use alloc::alloc::{dealloc, Layout};
        unsafe {
            dealloc(
                self.0 as _,
                Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap(),
            );
        }
    }
}



pub type ExitCode = usize;
pub type Tid = usize;
pub type Pid = usize;

pub fn tick() {
    //CPU.tick();
}

pub extern "C" fn shell(_arg: usize) -> ! {
    let mut history = Vec::new();

    loop {
        print!(">> ");
        let cmd = get_line(&mut history);
        if cmd == "" {
            continue;
        }
        let name = cmd.trim().split(' ').next().unwrap();
        //if let Ok(file) = ROOT_INODE.lookup(name) {
            //let data = file.read_as_vec().unwrap();
            //let _pid = processor()
                //.manager()
                //.add(Thread::new_user(data.as_slice(), cmd.split(' ')));
        //// TODO: wait until process exits, or use user land shell completely
        ////unsafe { thread::JoinHandle::<()>::_of(pid) }.join().unwrap();
        //} else {
            println!("Program {} not exist", name);
        //}
    }
}

const BEL: u8 = 0x07u8;
const BS: u8 = 0x08u8;
const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const ESC: u8 = 0x1bu8;
const DEL: u8 = 0x7fu8;

fn get_line(history: &mut Vec<Vec<u8>>) -> String {
    let mut cursor = 0;
    let mut line_vec = Vec::with_capacity(512);
    let mut history_index = history.len();
    loop {
        match get_char() {
            BS | DEL => {
                // Backspace
                if cursor > 0 {
                    cursor -= 1;
                    line_vec.remove(cursor);

                    put_char(BS);
                    for byte in &line_vec[cursor..] {
                        put_char(*byte);
                    }
                    put_char(b' ');
                    for _i in cursor..line_vec.len() {
                        put_char(ESC);
                        put_char(b'[');
                        put_char(b'D');
                    }
                    put_char(ESC);
                    put_char(b'[');
                    put_char(b'D');
                } else {
                    put_char(BEL);
                }
            }
            CR | LF => {
                // Return
                put_char(CR);
                put_char(LF);
                break;
            }
            ESC => {
                match get_char() {
                    b'[' => {
                        match get_char() {
                            b'D' => {
                                // Left arrow
                                if cursor > 0 {
                                    cursor -= 1;
                                    put_char(ESC);
                                    put_char(b'[');
                                    put_char(b'D');
                                } else {
                                    put_char(BEL);
                                }
                            }
                            b'C' => {
                                // Right arrow
                                if cursor < line_vec.len() {
                                    cursor += 1;
                                    put_char(ESC);
                                    put_char(b'[');
                                    put_char(b'C');
                                } else {
                                    put_char(BEL);
                                }
                            }
                            direction @ b'A' | direction @ b'B' => {
                                if direction == b'A' && history_index > 0 {
                                    // Up arrow
                                    history_index -= 1;
                                } else if direction == b'B' && history.len() > 0 // usize underflow
                                    && history_index < history.len() - 1
                                {
                                    // Down arrow
                                    history_index += 1;
                                } else {
                                    put_char(BEL);
                                    continue;
                                }

                                for _ in 0..line_vec.len() {
                                    put_char(ESC);
                                    put_char(b'[');
                                    put_char(b'D');
                                }
                                for _ in 0..line_vec.len() {
                                    put_char(b' ');
                                }
                                for _ in 0..line_vec.len() {
                                    put_char(ESC);
                                    put_char(b'[');
                                    put_char(b'D');
                                }
                                line_vec = history[history_index].clone();
                                cursor = line_vec.len();
                                for byte in &line_vec {
                                    put_char(*byte);
                                }
                            }
                            _ => {
                                put_char(BEL);
                            }
                        }
                    }
                    _ => {
                        put_char(BEL);
                    }
                }
            }
            byte if byte.is_ascii_graphic() || byte == b' ' => {
                line_vec.insert(cursor, byte);
                for byte in &line_vec[cursor..] {
                    put_char(*byte);
                }
                cursor += 1;
                for _i in cursor..line_vec.len() {
                    put_char(ESC);
                    put_char(b'[');
                    put_char(b'D');
                }
            }
            _ => {
                // unrecognized characters
                put_char(BEL);
            }
        }
    }

    if line_vec.len() > 0 {
        history.push(line_vec.clone());
    }
    String::from_utf8(line_vec).unwrap_or_default()
}

use crate::sbi::console_getchar;
fn get_char() -> u8 {
    //crate::fs::STDIN.pop() as u8
    console_getchar() as u8
}

fn put_char(ch: u8) {
    print!("{}", ch as char);
}
