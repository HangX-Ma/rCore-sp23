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
KERNEL_ELF	:= target/$(TARGET)/$(MODE)/os
KERNEL_BIN	:= $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm

# Disassembly
DISASM ?= -x

# run
KERNEL_ENTRY_PA := 0x80200000

QEMU_ARGS		:= -machine virt \
					-nographic \
					-bios $(BOOTLOADER) \
					-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
GDB_ARGS		:= -ex 'file target/riscv64gc-unknown-none-elf/release/os' \
					-ex 'set arch riscv:rv64' \
					-ex 'target remote localhost:1234'

build: $(KERNEL_BIN)

clean:
	@cargo clean

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@cargo build --$(MODE)

disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)

run: build
	@qemu-system-riscv64 $(QEMU_ARGS)

gdbserver: build
	@qemu-system-riscv64 $(QEMU_ARGS) -s -S

gdbclient:
	@riscv64-unknown-elf-gdb $(GDB_ARGS)

.PHONY: build kernel clean run gdbserver gdbclient disasm disasm-vim