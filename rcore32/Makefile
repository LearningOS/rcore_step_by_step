arch ?= riscv32-blog_os
board ?= none
mode ?= debug
LOG  ?= debug
graphic ?= off
smp  ?= 4

target := $(arch)
kernel := target/$(target)/$(mode)/rcore32
bin := target/$(target)/$(mode)/kernel.bin
bbl_path := $(PWD)/../bbl

usr_path := usr

### export environments ###
export ARCH = $(arch)
export SMP = $(smp)
export SFSIMG = $(usr_path)/try
export HELLO = $(usr_path)/hello_rust
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
export CC=$(cc)

.PHONY: all clean run build asm doc justrun debug kernel sfsimg install runnet

all: kernel

clean:
	@cargo clean

run: build justrun
	@echo $(SFSIMG)

justrun: build
	@qemu-system-riscv32 $(qemu_opts)

debug: $(kernel) $(bin)
	@qemu-system-$(arch) $(qemu_opts) -s -S &
	@sleep 1
	@$(gdb) $(kernel) -x src/tools/gdbinit

build: $(bin)

$(bin): kernel
	@mkdir -p target/$(target)/bbl && \
	cd target/$(target)/bbl && \
	$(bbl_path)/configure \
		--with-arch=rv32imac \
		--disable-fp-emulation \
		--host=riscv64-unknown-elf \
		--with-payload=$(abspath $(kernel)) && \
	make -j32 && \
	cp bbl $(abspath $@)

kernel:
	@-patch -p0 -N -b \
		$(shell rustc --print sysroot)/lib/rustlib/src/rust/src/libcore/sync/atomic.rs \
		src/tools/atomic.patch
	@cargo xbuild $(build_args)

docker:
	#sudo docker run -it --mount type=bind,source=/home/pan/Code/rcore_step_by_step/,destination=/mnt rust_riscv bash
	sudo docker run -it --mount type=bind,source=$(shell pwd)/..,destination=/mnt panqinglin/rust_riscv bash
