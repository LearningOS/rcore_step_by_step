target := riscv32-xy_os
bbl_path := $(abspath riscv-pk)
mode := debug
kernel := target/$(target)/$(mode)/xy_os
bin := target/$(target)/$(mode)/kernel.bin
usr_path := usr

export SFSIMG = $(usr_path)/hello_rust

.PHONY: all clean run build asm qemu kernel

all: kernel

$(bin): kernel
	mkdir -p target/$(target)/bbl && \
	cd target/$(target)/bbl && \
	$(bbl_path)/configure \
		--with-arch=rv32imac \
		--disable-fp-emulation \
		--host=riscv64-unknown-elf \
		--with-payload=$(abspath $(kernel)) && \
	make -j32 && \
	cp bbl $(abspath $@)

build: $(bin)

run: build qemu

kernel: 
	@cargo xbuild --target riscv32-xy_os.json

asm:
	@riscv64-unknown-elf-objdump -d $(kernel) | less

header:
	@riscv64-unknown-elf-objdump -h $(kernel)

qemu:
	qemu-system-riscv32 -kernel target/riscv32-xy_os/debug/kernel.bin -nographic -machine virt

clean:
	@cargo clean