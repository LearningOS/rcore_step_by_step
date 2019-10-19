    .section .text.entry
    .globl _start
_start:
    # 1.1 enable paging
    # satp = (1 << 31) | PPN(boot_page_table_sv32)
    lui     t0, %hi(boot_page_table_sv32)
    li      t1, 0xC0000000 - 0x80000000
    sub     t0, t0, t1
    srli    t0, t0, 12
    li      t1, 1 << 31
    or      t0, t0, t1
    csrw    satp, t0
    sfence.vma

    # 1.2 update PC to 0xCxxxxxxx
    lui     t0, %hi(remove_identity_map)
    addi    t0, t0, %lo(remove_identity_map)
    jr      t0

    # 1.3 remove identity map
remove_identity_map:
    lui     t0, %hi(boot_page_table_sv32_top)
    sw      zero, (-4 * 511)(t0)
    sfence.vma

    # 2. setup stack pointer
    lui sp, %hi(bootstacktop)

    # 3. call rust_main
    call    rust_main

    .section .bss.stack
    .align 12  #PGSHIFT
    .global bootstack
bootstack:
    .space 4096 * 4
    .global bootstacktop
bootstacktop:

    .section .data
    .align 12
boot_page_table_sv32:
    .zero 4 * 513
    # 0x80400000 -> 0x80400000 (4M)
    .word (0x80400 << 10) | 0xcf # VRWXAD
    .zero 4 * 255
    # 0xC0400000 -> 0x80400000 (4M)
    .word (0x80400 << 10) | 0xcf # VRWXAD
    .zero 4 * 254
boot_page_table_sv32_top:
