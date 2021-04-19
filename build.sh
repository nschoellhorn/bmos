#!/usr/bin/env bash

jq_bin=$(which jq)

if [[ $? -ne 0 ]]; then
    echo "`jq` not found. Please add it to your path."
    exit
fi

echo "=== Building Kernel"
cargo build

echo "=== Building bootable image"
bootloader_src_dir=$(cargo metadata --format-version 1 | $jq_bin '.packages | .[] | select(.name == "bootloader") | .manifest_path' -r | sed 's|/Cargo.toml||')

workdir=$(pwd)

cd $bootloader_src_dir
cargo builder --kernel-manifest $workdir/Cargo.toml --kernel-binary $workdir/target/x86_64-tos/debug/tos2 --target-dir $workdir/target --out-dir $workdir --target $workdir/x86_64-tos.json
