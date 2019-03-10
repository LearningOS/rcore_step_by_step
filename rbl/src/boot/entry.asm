	.option norvc
	.section .text.entry,"ax",@progbits
	.globl reset_vector
reset_vector:
	j do_reset

trap_vector:

do_reset:
	li x1, 0
	li x2, 0
	li x3, 0
	li x4, 0
	li x5, 0
	li x6, 0
	li x7, 0
	li x8, 0
	li x9, 0
// save a0 and a1; arguments from previous boot loader stage:
//  li x10, 0
//  li x11, 0
	li x12, 0
	li x13, 0
	li x14, 0
	li x15, 0
	li x16, 0
	li x17, 0
	li x18, 0
	li x19, 0
	li x20, 0
	li x21, 0
	li x22, 0
	li x23, 0
	li x24, 0
	li x25, 0
	li x26, 0
	li x27, 0
	li x28, 0
	li x29, 0
	li x30, 0
	li x31, 0
	csrw mscratch, x0

	# write mtvec and make sure it sticks
	la t0, trap_vector
	csrw mtvec, t0
	csrr t1, mtvec
1:bne t0, t1, 1b

	la sp, stacks + 4096 - 128

	csrr a3, mhartid
	slli a2, a3, 12
	add sp, sp, a2

	# Boot on the first hart
	beqz a3, boot_main

	# set MSIE bit to receive IPI
	li a2, 32
	csrw mie, a2
	.bss
	.align 12
	.globl stacks
stacks:
	.skip 4096
