use riscv::registers::{mcause, mpec, sie, mie};

pub fn init() {
    extern {
        fn __alltraps();
    }
    unsafe {
        // Set sscratch register to 0, indicating to exception vector that we are
        // presently executing in the kernel
        xscratch::write(0);
        // Set the exception vector address
        xtvec::write(__alltraps as usize, xtvec::TrapMode::Direct);
        // Enable IPI
        sie::set_ssoft();
        // Enable serial interrupt
        #[cfg(feature = "m_mode")]
        mie::set_mext();
        #[cfg(not(feature = "m_mode"))]
        sie::set_sext();
        // NOTE: In M-mode: mie.MSIE is set by BBL.
        //                  mie.MEIE can not be set in QEMU v3.0
        //                  (seems like a bug)
    }
    //use sbi::console_putchar;
    //console_putchar();
}
