.PHONY: docker

docker:
	docker run -it --mount type=bind,source=$(shell pwd),destination=/mnt panqinglin/rust_riscv