arch ?= riscv32
board ?= none
mode ?= debug
LOG  ?= debug
graphic ?= off

target := $(arch)
bootloader := target/$(target)/$(mode)/rbl
bin := target/$(target)/$(mode)/rbl

### export environments ###
export ARCH = $(arch)
# crate 'process' use this to set interrupt (MIE or SIE)

### qemu options ###
qemu_opts := \
	-machine virt \
	-kernel $(bin)

ifdef d
qemu_opts += -d $(d)
endif

ifeq ($(graphic), off)
qemu_opts += -nographic
else
qemu_opts += -serial stdio
endif

### build args ###
build_args := --target $(target).json --features "$(features)"

ifeq ($(mode), release)
build_args += --release
endif


### prefix ###
prefix := riscv64-unknown-elf-

ld := $(prefix)ld
cc := $(prefix)gcc
as := $(prefix)as
gdb := $(prefix)gdb
obj := $(prefix)objdump
export CC=$(cc)

.PHONY: all clean run build asm justrun debug bootloader cargo_run

all: bootloader

clean:
	@cargo clean

run: build justrun

justrun: build
	@qemu-system-riscv32 $(qemu_opts)

debug: $(bootloader) $(bin)
	@qemu-system-$(arch) $(qemu_opts) -s -S &
	@sleep 1
	@$(gdb) $(bootloader) -x ../tools/gdbinit

build: bootloader

objdump: bootloader
	$(obj) -d $(bin) > rbl.txt

bootloader:
	@cargo xbuild $(build_args)

cargo_run: bootloader
	@cargo run
