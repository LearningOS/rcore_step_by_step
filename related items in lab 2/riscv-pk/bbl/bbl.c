// See LICENSE for license details.

#include "bbl.h"
#include "mtrap.h"
#include "atomic.h"
#include "vm.h"
#include "bits.h"
#include "config.h"
#include "fdt.h"
#include <string.h>

extern char _payload_start, _payload_end; /* internal payload */
static const void* entry_point;
long disabled_hart_mask;

static uintptr_t dtb_output()
{
  /*
   * Place DTB after the payload, either the internal payload or a
   * preloaded external payload specified in device-tree, if present.
   *
   * Note: linux kernel calls __va(dtb) to get the device-tree virtual
   * address. The kernel's virtual mapping begins at its load address,
   * thus mandating device-tree is in physical memory after the kernel.
   */
  uintptr_t end = kernel_end ? (uintptr_t)kernel_end : (uintptr_t)&_payload_end;
  return (end + MEGAPAGE_SIZE - 1) / MEGAPAGE_SIZE * MEGAPAGE_SIZE;
}

static void filter_dtb(uintptr_t source)
{
  uintptr_t dest = dtb_output();
  uint32_t size = fdt_size(source);
  memcpy((void*)dest, (void*)source, size);

  // Remove information from the chained FDT
  filter_harts(dest, &disabled_hart_mask);
  filter_plic(dest);
  filter_compat(dest, "riscv,clint0");
  filter_compat(dest, "riscv,debug-013");
}

// static alloc root page table
static pte_t root_table[1 << RISCV_PGLEVEL_BITS] __attribute__((aligned(RISCV_PGSIZE)));

// only used for Sv48 systems, to map 0xFFFF_FFFF_C000_0000 to 0x8000_0000.
static pte_t p3_table[1 << RISCV_PGLEVEL_BITS] __attribute__((aligned(RISCV_PGSIZE)));

static void setup_page_table_sv32()
{
  // map kernel 0x80000000 -> 0xC0000000..
  int i_end = dtb_output();
  for(unsigned int i = 0x80000000; i <= i_end; i += MEGAPAGE_SIZE) {
    root_table[(i + 0x40000000) / MEGAPAGE_SIZE] = pte_create(i >> 12, PTE_R | PTE_W | PTE_X);
  }
}

static void setup_page_table_sv39()
{
  // map kernel [0o777] 0x80000000 -> 0xFFFFFFFF_C0000000 (size = 1G)
  root_table[0777] = pte_create(0x80000, PTE_R | PTE_W | PTE_X);
  // map recursive [0o774] (V), [0o775] (VRW), [0o776] (VRW)
  uintptr_t root_table_ppn = (uintptr_t)root_table >> RISCV_PGSHIFT;
  root_table[0774] = pte_create(root_table_ppn, 0);
  root_table[0775] = pte_create(root_table_ppn, PTE_R | PTE_W);
}

static void setup_page_table_sv48()
{
  // map kernel [0o777] 0x80000000 -> 0xFFFFFFFF_C0000000 (size = 1G)
	uintptr_t p3_table_ppn = (uintptr_t) p3_table >> RISCV_PGSHIFT;
  root_table[0777] = pte_create(p3_table_ppn, 0);
	p3_table[0777] = pte_create(0x80000, PTE_R | PTE_W | PTE_X);
  // map recursive [0o774] (V), [0o775] (VRW), [0o776] (VRW)
  uintptr_t root_table_ppn = (uintptr_t) root_table >> RISCV_PGSHIFT;
  root_table[0774] = pte_create(root_table_ppn, 0);
  root_table[0775] = pte_create(root_table_ppn, PTE_R | PTE_W);
}

static void enable_paging() {
  uintptr_t root_table_ppn = (uintptr_t)root_table >> RISCV_PGSHIFT;
  write_csr(sptbr, root_table_ppn | SATP_MODE_CHOICE);
  flush_tlb();
}

void boot_other_hart(uintptr_t unused __attribute__((unused)))
{
  const void* entry;
  do {
    entry = entry_point;
    mb();
  } while (!entry);

  long hartid = read_csr(mhartid);
  if ((1 << hartid) & disabled_hart_mask) {
    while (1) {
      __asm__ volatile("wfi");
#ifdef __riscv_div
      __asm__ volatile("div x0, x0, x0");
#endif
    }
  }

#ifdef BBL_BOOT_MACHINE
  asm (".pushsection .rodata\n"
       "bbl_functions:\n"
       "  .word mcall_trap\n"
       "  .word illegal_insn_trap\n"
       "  .word mcall_console_putchar\n"
       "  .word mcall_console_getchar\n"
       ".popsection\n");
  extern void* bbl_functions;
  enter_machine_mode(entry, hartid, dtb_output(), ~disabled_hart_mask & hart_mask, (uintptr_t)&bbl_functions);
#else /* Run bbl in supervisor mode */
  enable_paging();
#if __riscv_xlen == 64
    uintptr_t dtb = dtb_output() + 0xffffffff40000000;
#else
    uintptr_t dtb = dtb_output() + 0x40000000;
#endif
  enter_supervisor_mode(entry, hartid, dtb, ~disabled_hart_mask & hart_mask);
#endif
}

void boot_loader(uintptr_t dtb)
{
  filter_dtb(dtb);
#ifdef PK_ENABLE_LOGO
  print_logo();
#endif
#ifdef PK_PRINT_DEVICE_TREE
  fdt_print(dtb_output());
#endif
  mb();
  /* Use optional FDT preloaded external payload if present */
  entry_point = kernel_start ? kernel_start : &_payload_start;
#ifndef BBL_BOOT_MACHINE
#if __riscv_xlen == 64
#ifdef BBL_SV39
  setup_page_table_sv39();
#else
  setup_page_table_sv48();
#endif
  entry_point += 0xffffffff40000000;
#else
  setup_page_table_sv32();
  entry_point += 0x40000000;
#endif
#endif
  boot_other_hart(0);
}
