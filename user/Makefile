TARGET 		:= riscv64gc-unknown-none-elf
MODE 		:= release
# MODE := debug
APP_DIR 	:= src/bin
TARGET_DIR 	:= target/$(TARGET)/$(MODE)
BUILD_DIR 	:= build

TEST ?= 0
ifeq ($(TEST), 0)
	APPS :=  $(filter-out $(wildcard $(APP_DIR)/test*.rs), $(wildcard $(APP_DIR)/*.rs))
else
	APPS :=  $(wildcard $(APP_DIR)/test$(TEST)*.rs)
endif
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
BINS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))

OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

PY := python3

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

elf:
	@cargo build $(MODE_ARG)

binary: elf
	@$(foreach elf, $(ELFS),\
		$(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf));)

build: binary

.PHONY: elf binary build clean