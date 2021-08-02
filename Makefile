
bin=target/x86_64-os/debug/bootimage-os-x86.bin
img=disk.img
	
# Rebuild MOROS if the features list changed
image:
	qemu-img create $(img) 32M
	cargo bootimage
	dd conv=notrunc if=$(bin) of=$(img)

run:
	qemu-system-x86_64 -drive id=boot,format=raw,file=disk.img -vga cirrus
debug:
	make image
	make run

clean:
	cargo clean
