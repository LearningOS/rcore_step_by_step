
#[inline(always)]
pub unsafe fn enable_and_wfi() {    // 使能中断并等待中断
    asm!("csrsi sstatus, 1 << 1; wfi" :::: "volatile");
}

#[inline(always)]
pub unsafe fn disable_and_store() -> usize {    // 禁用中断
    let sstatus: usize;
    asm!("csrci sstatus, 1 << 1" : "=r"(sstatus) ::: "volatile");
    sstatus & (1 << 1)
}
