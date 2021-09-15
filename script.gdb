add-symbol-file init/kernel 
target remote | qemu-system-x86_64 -bios /usr/share/edk2-ovmf/x64/OVMF.fd -S -gdb stdio -drive format=raw,file=image
