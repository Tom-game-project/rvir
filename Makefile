# test rules 
#

GDB=gdb
GDB_SETUP=setup.gdb

DEMO=output.elf

DEMO_SRC= \
	test.S \
    test_program.S \

KERNEL_LD=test.ld


$(DEMO): $(DEMO_SRC)
	riscv64-unknown-elf-gcc -g -nostdlib -Ttext=0x80000000 $(DEMO_SRC) -T $(KERNEL_LD) -o $(DEMO)

demo_run: $(DEMO)
	qemu-system-riscv64 -machine virt -bios none -nographic -kernel $(DEMO) -s -S

demo: $(DEMO)
	# ターミナル1: ビルドしてQEMUをGDB待機モードで起動
	zellij run -f --name "QEMU (RISC-V)" -- make demo_run
	$(GDB) -q -x $(GDB_SETUP) $(DEMO)

