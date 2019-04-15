out_dir ?= build
out_img ?= rcore32.img

cargo_args := --target riscv32-blog_os.json
rust_src_dir := rust/src/bin
rust_bin_path := rust/target/riscv32-blog_os/debug
rust_bins := $(patsubst $(rust_src_dir)/%.rs, $(rust_bin_path)/%, $(wildcard $(rust_src_dir)/*.rs)) 

.PHONY: all clean build rust 

all : rust build

rust : 
	@echo Building rust user program
	@cd rust && cargo xbuild $(cargo_args)
	@rm -rf $(out_dir)/rust && mkdir -p $(out_dir)/rust
	@echo $(out_dir)
	@cp -r $(rust_bins) $(out_dir)/rust

build : $(out_img)

$(out_img) :
	@rcore-fs-fuse $@ $(out_dir) zip

rcore-fs-fuse:
ifeq ($(shell which rcore-fs-fuse),)
	@echo Installing rcore-fs-fuse
	@cargo install rcore-fs-fuse --git https://github.com/rcore-os/rcore-fs --rev c611248
endif

clean :
	@rm -rf $(out_dir)
	@cd rust && cargo clean
