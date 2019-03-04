target := riscv_32-xy_os
bbl_path := $(abspath riscv-pk)
mode := debug
kernel := target/$(target)/$(mode)/xy_os
bin := target/$(target)/$(mode)/kernel.bin

all: $(bin)

$(bin): src
	@cargo xbuild --target riscv_32-xy_os.json
	mkdir -p target/$(target)/bbl && \
	cd target/$(target)/bbl && \
	$(bbl_path)/configure \
		--with-arch=rv32imac \
		--disable-fp-emulation \
		--host=riscv64-unknown-elf \
		--with-payload=$(abspath $(kernel)) && \
	make -j32 && \
	cp bbl $(abspath $@)

qemu:
	qemu-system-riscv32 -kernel target/riscv_32-xy_os/debug/kernel.bin -nographic -machine virt