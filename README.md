# learn-rCore
Rust OS Learning

[rCore-Tutorial](https://github.com/rcore-os/rCore-Tutorial)
[rCore-Tutorial-deploy](https://rcore-os.github.io/rCore-Tutorial-deploy/)

# Env
```bash
qemu 6.0.0
```

### 查看可执行文件内存布局
```bash
rust-objdump target/riscv64imac-unknown-none-elf/debug/os -x --arch-name=riscv64   # all header
rust-objdump target/riscv64imac-unknown-none-elf/debug/os -h --arch-name=riscv64   # summaries of the headers for each section
rust-objdump target/riscv64imac-unknown-none-elf/debug/os -d --arch-name=riscv64   # assembler mnemonics for the machine instructions
rust-objdump target/riscv64imac-unknown-none-elf/debug/os -S --arch-name=riscv64   # source inlined with disassembly
```