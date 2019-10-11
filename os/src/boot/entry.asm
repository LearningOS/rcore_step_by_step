    .section .text.entry
    .globl _start
_start:
    lui sp, %hi(bootstacktop)   # 将栈指针 sp 置为栈顶地址

    call rust_main

    .section .bss.stack
    .align 12  # PGSHIFT
    .global bootstack
bootstack:
    .space 4096 * 4		        # 开辟一块栈空间（4个页）
    .global bootstacktop
bootstacktop:
