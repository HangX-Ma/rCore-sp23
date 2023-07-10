# os/Makefile

# Board
BOARD			?= qemu
SBI				?= rustsbi
BOOTLOADER		:= ../bootloader/$(SBI)-$(BOARD).bin

# Binutils
OBJDUMP		:= rust-objdump --arch-name=riscv64
OBJCOPY		:= rust-objcopy --binary-architecture=riscv64

# build
TARGET		:= riscv64gc-unknown-none-elf
MODE		:= release
# MODE		:= debug
KERNEL_ELF	:= target/$(TARGET)/$(MODE)/os
KERNEL_BIN	:= $(KERNEL_ELF).bin
DISASM_TMP	:= target/$(TARGET)/$(MODE)/asm
FS_IMG		:= ../user/target/$(TARGET)/$(MODE)/fs.img
APPS		:= ../user/src/bin/*

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# Disassembly
DISASM ?= -x
DUMP_DTB ?= false

BASE ?= 1
TEST ?= 0

ifeq ($(DUMP_DTB), true)
	DUMP_DTB_ARG := ,dumpdtb=dump.dtb
endif

# run
KERNEL_ENTRY_PA := 0x80200000

QEMU_ARGS		:= -machine virt$(DUMP_DTB_ARG)\
					-nographic \
					-bios $(BOOTLOADER) \
					-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)\
					-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
					-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

GDB_ARGS		:= -ex 'file $(KERNEL_ELF)' \
					-ex 'set arch riscv:rv64' \
					-ex 'target remote localhost:1234'

build: $(KERNEL_BIN) fs-img

clean:
	@cargo clean

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

fs-img: $(APPS)
	@cd ../user && make build TEST=$(TEST) BASE=$(BASE)
	@rm -f $(FS_IMG)
	@cd ../easy-fs-fuse && cargo run --release -- -s ../user/build/app/ -t ../user/target/riscv64gc-unknown-none-elf/release/

kernel:
	@echo Running platform: $(BOARD)
	@cp src/linker-$(BOARD).ld src/linker.ld
	@cargo build $(MODE_ARG)
	@rm src/linker.ld

disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)

run: run-inner

run-inner: build
	@qemu-system-riscv64 $(QEMU_ARGS)

debug: build
	@tmux new-session -d \
		"qemu-system-riscv64 $(QEMU_ARGS) -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

gdbserver: build
	@qemu-system-riscv64 $(QEMU_ARGS) -s -S

gdbclient:
	@riscv64-unknown-elf-gdb $(GDB_ARGS)

.PHONY: build kernel clean run gdbserver gdbclient disasm disasm-vim fs-img