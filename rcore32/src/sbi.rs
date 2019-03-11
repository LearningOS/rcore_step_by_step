//! Port from sbi.h

#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret;
    unsafe {
        asm!("ecall"
            : "={x10}" (ret)
            : "{x10}" (arg0), "{x11}" (arg1), "{x12}" (arg2), "{x17}" (which)
            : "memory"
            : "volatile");
    }
    ret
}

pub fn console_putchar(ch: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, ch, 0, 0);
}

pub fn console_getchar() -> usize {
    sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0)
}

pub fn set_timer(stime_value: u64) {
    sbi_call(SBI_SET_TIMER, stime_value as usize, (stime_value >> 32) as usize, 0);
}

pub fn clear_ipi() {
    sbi_call(SBI_CLEAR_IPI, 0, 0, 0);
}

pub fn send_ipi(hart_mask: *const usize) {
    sbi_call(SBI_SEND_IPI, hart_mask as usize, 0, 0);
}

pub fn other_ecall() {
    sbi_call(9, 0, 0, 0);
}

/// sbi: 设置时钟
const SBI_SET_TIMER: usize = 0;
/// sbi: 输出字符
const SBI_CONSOLE_PUTCHAR: usize = 1;
/// sbi: 输入字符
const SBI_CONSOLE_GETCHAR: usize = 2;
/// sbi: 停止机器运行
const SBI_SHUTDOWN: usize = 8;
const SBI_CLEAR_IPI: usize = 3;
const SBI_SEND_IPI: usize = 4;
