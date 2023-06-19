# os/Makefile

OBJCOPY		:= rust-objcopy
# build
TARGET		:= riscv64gc-unknown-none-elf
MODE		:= release
KERNEL_ELF	:= target/$(TARGET)/$(MODE)/os
KERNEL_BIN	:=  target/$(TARGET)/$(MODE)/os.bin

# run
KERNEL_ENTRY_PA := 0x80200000
BOARD			?= qemu
SBI				?= rustsbi
BOOTLOADER		:= ../bootloader/$(SBI)-$(BOARD).bin
GDB_ENABLE		?= false
QEMU_ARGS		:= -machine virt \
					-nographic \
					-bios $(BOOTLOADER) \
					-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
GDB_ARGS		:= -ex 'file target/riscv64gc-unknown-none-elf/release/os' \
					-ex 'set arch riscv:rv64' \
					-ex 'target remote localhost:1234'

ifeq ($(GDB_ENABLE),true)
QEMU_ARGS := $(QEMU_ARGS) -s -S
endif

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@cargo build --$(MODE)

run: $(KERNEL_BIN)
	qemu-system-riscv64 $(QEMU_ARGS)

gdb:
	riscv64-unknown-elf-gdb $(GDB_ARGS)

.PHONY: build kernel clean run gdb