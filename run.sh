#!/usr/bin/env bash
cargo build --release
if [ $? -ne 0 ]; then
    echo "Failed to build kernel, check errors above."
    exit
fi
cargo mkboot
if [ $? -ne 0 ]; then
    echo "Failed to build bootable image, check errors above."
    exit
fi
qemu-system-x86_64 -drive format=raw,file=target/x86_64-bmos/release/boot-bios-bmos.img  -d cpu_reset -serial stdio -m 1G
