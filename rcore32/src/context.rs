use riscv::register::{
    sstatus::Sstatus as Xstatus,
    mcause::Mcause,
};

pub struct TrapFrame {
    pub x: [usize; 32], // general registers
    pub sstatus: Xstatus, // Supervisor Status Register
    pub sepc: usize, // Supervisor exception program counter, save the trap virtual address (here is used to save the process program entry addr?)
    pub stval: usize, // Supervisor trap value
    pub scause: Mcause, // scause register: record the cause of exception/interrupt/trap
}

impl TrapFrame {
    pub fn increase_sepc(self: &mut Self) {
        self.sepc = self.sepc + 4;
    }

    pub fn print_trapframe(self: &mut Self) {
        println!("print the trapfram:  
                    cause : {:#x} 
                    sepc : {:#x} 
                    stval : {:#x} ",
                    self.scause.bits(),
                    self.sepc ,
                    self.stval as usize);
    }
}
