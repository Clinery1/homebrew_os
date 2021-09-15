all: init/kernel
release: init/kernel_release

# Kernel build
init/kernel: src/** target/fs.img
	cargo xbuild --target ./triplets/os.json
	cp ./target/os/debug/homebrew_os init/kernel

# hacky, but I dont know how to use makefiles
init/kernel_release: src/** target/fs.img
	cargo xbuild --target ./triplets/os.json --release
	cp ./target/os/debug/homebrew_os init/kernel

target/fs.img:
	mkdir -p target
	fallocate target/fs.img -l 128M
	mkfs.vfat target/fs.img
